use crate::components::*;
use crate::GameEngine;

pub fn calculate_effective_damage(raw_dmg: f32, dmg_type: DamageType, res: &Resistances) -> f32 {
    let r = match dmg_type {
        DamageType::Physical => res.physical,
        DamageType::Energy => res.energy,
        DamageType::Explosive => res.explosive,
        DamageType::Arcane => res.arcane,
    };
    let factor = (1.0 - r.clamp(-0.5, 0.85)).max(0.1);
    (raw_dmg * factor).max(1.0)
}

/// Trừ sát thương vào Khiên trước (nếu có), phần còn lại trừ vào Máu
pub fn apply_damage_to_unit(unit: &mut Unit, taken: f32) -> f32 {
    let mut remaining = taken;
    if unit.shield > 0.0 {
        if unit.shield >= remaining {
            unit.shield -= remaining;
            remaining = 0.0;
        } else {
            remaining -= unit.shield;
            unit.shield = 0.0;
        }
    }
    if remaining > 0.0 {
        unit.hp -= remaining;
    }
    unit.hit_flash_timer = 0.08;
    taken
}

pub fn update_magic_circles(engine: &mut GameEngine, dt: f32) {
    let factions = engine.team_factions();
    engine
        .spatial_grid
        .rebuild_with_factions(&engine.units, &engine.buildings, &factions);
    let has_corrosive: [bool; 8] =
        std::array::from_fn(|i| engine.team_tech(Team::from_index(i)).corrosive_curse);
    let mut new_texts = Vec::new();
    let default_dps = engine
        .config
        .get_unit(UnitType::VoidWarlock)
        .map(|u| u.percent_max_hp_dps)
        .filter(|&p| p > 0.0)
        .unwrap_or(crate::config::VOID_WARLOCK_DEFAULT_MAX_HP_DPS);

    for circle in engine.magic_circles.iter_mut() {
        circle.lifetime -= dt;
        circle.tick_timer += dt;

        let should_tick_text = circle.tick_timer >= 0.5;
        if should_tick_text {
            circle.tick_timer = 0.0;
        }

        let c_team_idx = circle.team.index() % 8;
        let c_faction = factions[c_team_idx];
        let circle_dps = if circle.damage_pct_per_sec > 0.0 {
            circle.damage_pct_per_sec
        } else {
            default_dps
        };
        let dmg_type = circle.damage_type;

        // 1. Sát thương lên lính & công trình địch qua SpatialGrid trong bán kính circle.radius
        let mut affected_units = Vec::new();
        let mut affected_buildings = Vec::new();

        engine.spatial_grid.for_each_hostile_in_radius(
            glam::Vec2::new(circle.x, circle.y),
            circle.radius,
            c_faction,
            |entity, dist| {
                if entity.is_building {
                    affected_buildings.push((entity.index, dist));
                } else {
                    affected_units.push((entity.index, dist));
                }
            },
        );

        for (u_idx, _dist) in affected_units {
            if u_idx < engine.units.len() {
                let u = &mut engine.units[u_idx];
                // Sát thương theo % máu tối đa mỗi giây từ config (mặc định 10% max HP/s)
                let raw_dmg = circle_dps * u.max_hp * dt;
                // Sát thương tính qua hệ số Kháng sát thương như các đòn đánh thường
                let taken = calculate_effective_damage(raw_dmg, dmg_type, &u.resistances);
                apply_damage_to_unit(u, taken);

                // Hiệu ứng đòn đánh: Lời nguyền ăn mòn (Corrosive Curse) bào mòn kháng kẻ địch khi dính sát thương
                if has_corrosive[c_team_idx] && should_tick_text {
                    u.resistances.physical = (u.resistances.physical - 0.04).max(0.0);
                    u.resistances.energy = (u.resistances.energy - 0.04).max(0.0);
                    u.resistances.explosive = (u.resistances.explosive - 0.04).max(0.0);
                    u.resistances.arcane = (u.resistances.arcane - 0.04).max(0.0);
                }

                if should_tick_text && !engine.is_headless {
                    let tick_raw = circle_dps * u.max_hp * 0.5;
                    let tick_taken = calculate_effective_damage(tick_raw, dmg_type, &u.resistances);
                    new_texts.push(FloatingText {
                        id: 0,
                        text: format!("-{:.0}", tick_taken),
                        x: u.x,
                        y: u.y - 12.0,
                        color: dmg_type.color_code().to_string(),
                        lifetime: 0.6,
                        max_lifetime: 0.6,
                    });
                }
            }
        }

        for (b_idx, _dist) in affected_buildings {
            if b_idx < engine.buildings.len() {
                let b = &mut engine.buildings[b_idx];
                let b_rad = b.width.max(b.height) * 0.5;
                let raw_dmg = circle_dps * b.max_hp * dt;
                let taken = calculate_effective_damage(raw_dmg, dmg_type, &b.resistances);
                b.hp -= taken;
                // Hiệu ứng bị tấn công như các đơn vị khác (làm chậm xây dựng, báo động đỏ)
                b.under_attack_timer = crate::config::BUILDING_UNDER_ATTACK_DURATION;

                if should_tick_text && !engine.is_headless {
                    let tick_raw = circle_dps * b.max_hp * 0.5;
                    let tick_taken = calculate_effective_damage(tick_raw, dmg_type, &b.resistances);
                    new_texts.push(FloatingText {
                        id: 0,
                        text: format!("-{:.0}", tick_taken),
                        x: b.x,
                        y: b.y - b_rad,
                        color: dmg_type.color_code().to_string(),
                        lifetime: 0.7,
                        max_lifetime: 0.7,
                    });
                }
            }
        }
    }

    for mut ft in new_texts {
        ft.id = engine.alloc_id();
        engine.floating_texts.push(ft);
    }

    engine.magic_circles.retain(|c| c.lifetime > 0.0);
}

pub fn update_projectiles(engine: &mut GameEngine, dt: f32) {
    let factions = engine.team_factions();
    let has_corrosive: [bool; 8] =
        std::array::from_fn(|i| engine.team_tech(Team::from_index(i)).corrosive_curse);
    let mut new_texts = Vec::new();
    let mut explosions = Vec::new();

    for p in engine.projectiles.iter_mut() {
        p.lifetime -= dt;
        p.prev_x = p.x;
        p.prev_y = p.y;
        let dx = p.target_x - p.x;
        let dy = p.target_y - p.y;
        let dist = (dx * dx + dy * dy).sqrt();
        let is_laser = p.is_laser || p.visual == ProjectileVisual::Laser;

        if !is_laser && dist > 10.0 {
            let step = (p.speed * dt).min(dist);
            p.x += (dx / dist) * step;
            p.y += (dy / dist) * step;
        }

        if p.lifetime <= 0.0 || (!is_laser && dist <= 18.0) {
            let is_aoe = p.aoe_radius > 0.0;
            let aoe_range = if is_aoe { p.aoe_radius.max(30.0) } else { 24.0 };
            let p_team_idx = p.team.index() % 8;
            let p_faction = factions[p_team_idx];

            // Sát thương lính & công trình địch bằng SpatialGrid (O(1) vùng nổ thay vì O(U+B))
            let mut hit_units = Vec::new();
            let mut hit_buildings = Vec::new();

            engine.spatial_grid.for_each_hostile_in_radius(
                glam::Vec2::new(p.target_x, p.target_y),
                aoe_range,
                p_faction,
                |entity, dist| {
                    if entity.is_building {
                        hit_buildings.push((entity.index, dist));
                    } else {
                        hit_units.push((entity.index, dist));
                    }
                },
            );

            // Sát thương lính địch (hấp thụ vào Khiên trước nếu có)
            for (u_idx, d) in hit_units {
                if u_idx < engine.units.len() {
                    let u = &mut engine.units[u_idx];
                    let falloff = if is_aoe {
                        (1.0 - (d / aoe_range) * 0.35).max(0.65)
                    } else {
                        1.0
                    };
                    let taken = calculate_effective_damage(
                        p.damage * falloff,
                        p.damage_type,
                        &u.resistances,
                    );
                    apply_damage_to_unit(u, taken);

                    if has_corrosive[p_team_idx] {
                        u.resistances.physical = (u.resistances.physical - 0.04).max(0.0);
                        u.resistances.energy = (u.resistances.energy - 0.04).max(0.0);
                        u.resistances.explosive = (u.resistances.explosive - 0.04).max(0.0);
                        u.resistances.arcane = (u.resistances.arcane - 0.04).max(0.0);
                    }

                    if !engine.is_headless {
                        new_texts.push(FloatingText {
                            id: 0,
                            text: format!("-{:.0}", taken),
                            x: u.x,
                            y: u.y - 12.0,
                            color: p.damage_type.color_code().to_string(),
                            lifetime: 0.7,
                            max_lifetime: 0.7,
                        });
                    }

                    if !is_aoe {
                        break;
                    }
                }
            }

            // Sát thương công trình địch (áp dụng building_damage_mult, ví dụ AcidBaneling x2)
            for (b_idx, _d) in hit_buildings {
                if b_idx < engine.buildings.len() {
                    let b = &mut engine.buildings[b_idx];
                    let b_rad = b.width.max(b.height) * 0.5;
                    let raw_dmg = p.damage * p.building_damage_mult;
                    let taken = calculate_effective_damage(raw_dmg, p.damage_type, &b.resistances);
                    b.hp -= taken;
                    b.under_attack_timer = crate::config::BUILDING_UNDER_ATTACK_DURATION;
                    b.hit_flash_timer = 0.08;

                    if !engine.is_headless {
                        new_texts.push(FloatingText {
                            id: 0,
                            text: format!("-{:.0}", taken),
                            x: b.x,
                            y: b.y - b_rad,
                            color: p.damage_type.color_code().to_string(),
                            lifetime: 0.8,
                            max_lifetime: 0.8,
                        });
                    }

                    if !is_aoe {
                        break;
                    }
                }
            }

            if !engine.is_headless && !p.is_melee {
                explosions.push((p.target_x, p.target_y, p.visual, is_aoe));
            }

            p.lifetime = -1.0;
        }
    }

    // Sinh hiệu ứng hạt khi đạn nổ trúng đích
    for (ex, ey, visual, is_aoe) in explosions {
        match visual {
            ProjectileVisual::Rocket => {
                engine.spawn_particles(ex, ey, 14, "#f97316", 150.0, 3.2, 0.4);
                engine.spawn_particles(ex, ey, 6, "#ef4444", 110.0, 2.5, 0.3);
            }
            ProjectileVisual::FlyingSword => {
                engine.spawn_particles(ex, ey, 6, "#38bdf8", 100.0, 2.0, 0.3);
            }
            _ => {
                if is_aoe {
                    engine.spawn_particles(ex, ey, 10, "#fbbf24", 130.0, 2.8, 0.35);
                } else {
                    engine.spawn_particles(ex, ey, 4, "#facc15", 85.0, 1.8, 0.25);
                }
            }
        }
    }

    for mut ft in new_texts {
        ft.id = engine.alloc_id();
        engine.floating_texts.push(ft);
    }

    engine.projectiles.retain(|p| p.lifetime > 0.0);
}

pub fn update_floating_texts(engine: &mut GameEngine, dt: f32) {
    for ft in engine.floating_texts.iter_mut() {
        ft.lifetime -= dt;
        ft.y -= 25.0 * dt;
    }
    engine.floating_texts.retain(|ft| ft.lifetime > 0.0);
}
