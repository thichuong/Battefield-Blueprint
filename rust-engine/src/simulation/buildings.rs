use crate::components::*;
use crate::config::BuildingStatsConfig;
use crate::resources::TechTreeState;
use crate::GameEngine;

pub fn calculate_building_construction_step(
    build_progress: f32,
    under_attack_timer: f32,
    is_worker_near: bool,
    dt: f32,
) -> f32 {
    let mut build_speed = if is_worker_near {
        crate::config::BUILD_SPEED_WORKER
    } else {
        crate::config::BUILD_SPEED_BASE
    };
    if under_attack_timer > 0.0 {
        build_speed *= crate::config::BUILDING_UNDER_ATTACK_SPEED_MULTIPLIER;
    }
    (build_progress + build_speed * dt).min(1.0)
}

pub fn update_buildings(engine: &mut GameEngine, dt: f32) {
    let mut new_spawned_units = Vec::new();
    let mut new_turret_projectiles = Vec::new();
    let factions = engine.team_factions();
    let team_techs: [TechTreeState; 8] =
        std::array::from_fn(|i| engine.team_tech(Team::from_index(i)).clone());

    let worker_build_range_sq = crate::config::WORKER_BUILD_RANGE * crate::config::WORKER_BUILD_RANGE;

    // Chỉ thu thập vị trí của Engineer khi có ít nhất 1 công trình chưa xây xong
    let has_incomplete_building = engine.buildings.iter().any(|b| b.build_progress < 1.0);
    let engineer_pos: Vec<(Team, f32, f32)> = if has_incomplete_building {
        engine
            .units
            .iter()
            .filter(|u| u.unit_type == UnitType::Engineer)
            .map(|u| (u.team, u.x, u.y))
            .collect()
    } else {
        Vec::new()
    };

    let num_buildings = engine.buildings.len();
    for b_idx in 0..num_buildings {
        let (b_team, b_type, b_x, b_y) = {
            let b = &engine.buildings[b_idx];
            (b.team, b.building_type, b.x, b.y)
        };
        let b_team_tech = &team_techs[b_team.index() % 8];
        let my_faction = factions[b_team.index() % 8];

        let b_cfg = engine.config.get_building(b_type);
        let range = b_cfg.map(|c| c.turret_range).unwrap_or(0.0);
        let base_damage = b_cfg.map(|c| c.turret_damage).unwrap_or(0.0);

        let target_pos = if range > 0.0 && base_damage > 0.0 && engine.buildings[b_idx].current_cooldown <= 0.0 {
            if let Some((target_id, is_target_building, _dist)) = engine
                .spatial_grid
                .find_nearest_hostile(glam::Vec2::new(b_x, b_y), range, my_faction)
            {
                if is_target_building {
                    if let Some(&tb_idx) = engine.spatial_grid.building_id_to_index.get(&target_id) {
                        if tb_idx < engine.buildings.len() {
                            let tb = &engine.buildings[tb_idx];
                            if tb.id == target_id {
                                Some((tb.x, tb.y))
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    if let Some(&t_idx) = engine.spatial_grid.unit_id_to_index.get(&target_id) {
                        if t_idx < engine.units.len() {
                            let tu = &engine.units[t_idx];
                            if tu.id == target_id {
                                Some((tu.x, tu.y))
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
            } else {
                None
            }
        } else {
            None
        };

        let b = &mut engine.buildings[b_idx];
        if b.under_attack_timer > 0.0 {
            b.under_attack_timer = (b.under_attack_timer - dt).max(0.0);
        }

        if b.build_progress < 1.0 {
            let is_worker_near = engineer_pos.iter().any(|(t, x, y)| {
                *t == b.team
                    && ((*x - b.x).powi(2) + (*y - b.y).powi(2)) <= worker_build_range_sq
            });
            let next_progress = calculate_building_construction_step(
                b.build_progress,
                b.under_attack_timer,
                is_worker_near,
                dt,
            );
            let added_hp = b.max_hp * (next_progress - b.build_progress).max(0.0);
            b.build_progress = next_progress;
            b.hp = (b.hp + added_hp).min(b.max_hp * b.build_progress);
        } else {
            // Huấn luyện lính
            if !b.training_queue.is_empty() {
                b.training_timer += dt;
                let mut required_time = engine.config.get_unit_training_time(b.training_queue[0]);
                if b_team_tech.hive_proliferation {
                    required_time *= 0.65;
                }
                if b.training_timer >= required_time {
                    let u_type = b.training_queue.remove(0);
                    b.training_timer = 0.0;
                    new_spawned_units.push((
                        b.team,
                        u_type,
                        b.x + 10.0,
                        b.y + 40.0,
                        b.rally_x,
                        b.rally_y,
                    ));
                }
            }

            // Công trình phòng thủ / tấn công tự động: Sử dụng SpatialGrid để tìm mục tiêu O(1)
            let b_cfg = engine.config.get_building(b_type);
            let range = b_cfg.map(|c| c.turret_range).unwrap_or(0.0);
            let base_damage = b_cfg.map(|c| c.turret_damage).unwrap_or(0.0);

            if range > 0.0 && base_damage > 0.0 {
                if b.current_cooldown > 0.0 {
                    b.current_cooldown -= dt;
                } else if let Some((ex, ey)) = target_pos {
                    let base_cd = b_cfg.map(|c| c.turret_cooldown).unwrap_or(1.0).max(0.1);
                    let base_dmg_type = b_cfg.map(|c| c.turret_damage_type).unwrap_or(b.damage_type);
                    let base_speed = b_cfg
                        .map(|c| c.turret_projectile_speed)
                        .unwrap_or(900.0)
                        .max(100.0);

                    let has_overload = b_team_tech.overload_strike;
                    b.current_cooldown = if has_overload { base_cd * 0.6 } else { base_cd };

                    let has_dark_rockets = b_team_tech.dark_matter_rockets;
                    let spawn_y = b.y - (b.height * 0.25).min(20.0);
                    let is_laser = !has_dark_rockets && base_dmg_type == DamageType::Energy;
                    let angle = (ey - spawn_y).atan2(ex - b.x);
                    let visual = if has_dark_rockets {
                        ProjectileVisual::Rocket
                    } else if is_laser {
                        ProjectileVisual::Laser
                    } else {
                        ProjectileVisual::Bullet
                    };

                    new_turret_projectiles.push(Projectile {
                        id: 0,
                        team: b.team,
                        x: b.x,
                        y: spawn_y,
                        prev_x: b.x,
                        prev_y: spawn_y,
                        target_x: ex,
                        target_y: ey,
                        speed: if has_dark_rockets {
                            base_speed * (750.0 / 900.0)
                        } else {
                            base_speed
                        },
                        damage: if has_dark_rockets {
                            base_damage * 1.4
                        } else {
                            base_damage
                        },
                        damage_type: if has_dark_rockets {
                            DamageType::Explosive
                        } else {
                            base_dmg_type
                        },
                        aoe_radius: if has_dark_rockets { 40.0 } else { 0.0 },
                        is_laser,
                        lifetime: if has_dark_rockets {
                            1.2
                        } else if is_laser {
                            0.3
                        } else {
                            1.0
                        },
                        is_melee: false,
                        building_damage_mult: 1.0,
                        visual,
                        angle,
                    });
                }
            }
        }
    }

    for (team, u_type, sx, sy, rx, ry) in new_spawned_units {
        let u_id = engine.alloc_id();
        let mut unit = engine.config.create_unit(u_id, team, u_type, sx, sy);
        unit.target_x = rx;
        unit.target_y = ry;
        unit.facing_right = rx >= sx;

        // Nếu là Engineer, tự động tìm mỏ quặng gần nhất để khai thác ngay lập tức
        if u_type == UnitType::Engineer {
            let nearest_mineral = engine
                .minerals
                .iter()
                .filter(|m| m.amount > 0.0)
                .min_by(|a, b| {
                    let da = (a.x - sx).powi(2) + (a.y - sy).powi(2);
                    let db = (b.x - sx).powi(2) + (b.y - sy).powi(2);
                    da.partial_cmp(&db).unwrap()
                })
                .map(|m| m.id);

            if let Some(target_mineral_id) = nearest_mineral {
                unit.order = UnitOrder::Harvest {
                    target_mineral_id,
                    phase: HarvestPhase::MovingToNode,
                };
            } else {
                unit.order = UnitOrder::MoveTo { x: rx, y: ry };
            }
        } else {
            unit.order = UnitOrder::MoveTo { x: rx, y: ry };
        }

        // Áp dụng nâng cấp công nghệ (Tech upgrades cho team)
        let team_tech = engine.team_tech(team);
        if u_type == UnitType::ShockTrooper && team_tech.energy_shielding {
            unit.hp += 30.0;
            unit.max_hp += 30.0;
        }
        if team_tech.exo_boosters || team_tech.hive_proliferation {
            unit.speed *= 1.25;
        }
        if team_tech.titan_alloy_plating {
            unit.resistances.physical = (unit.resistances.physical + 0.25).min(0.80);
        }
        if team_tech.golden_bell_shield {
            unit.resistances.energy = (unit.resistances.energy + 0.25).min(0.80);
            unit.resistances.arcane = (unit.resistances.arcane + 0.30).min(0.80);
        }
        if team_tech.primeval_scales
            && (u_type == UnitType::ShockTrooper
                || u_type == UnitType::Vanguard
                || u_type == UnitType::BehemothTitan
                || u_type == UnitType::AcidBaneling)
        {
            unit.resistances.physical = (unit.resistances.physical + 0.25).min(0.80);
            unit.resistances.explosive = (unit.resistances.explosive + 0.25).min(0.80);
        }

        engine.units.push(unit);
    }

    for mut p in new_turret_projectiles {
        p.id = engine.alloc_id();
        engine.projectiles.push(p);
    }

    // Hiệu ứng khói lửa bốc lên từ các công trình bị hư hại nặng (< 45% HP)
    if !engine.is_headless && fastrand::f32() < 0.3 {
        let damaged_pos: Vec<(f32, f32)> = engine
            .buildings
            .iter()
            .filter(|b| b.hp < b.max_hp * 0.45 && b.build_progress >= 1.0)
            .map(|b| {
                (
                    b.x + (fastrand::f32() - 0.5) * b.width * 0.5,
                    b.y - b.height * 0.2,
                )
            })
            .collect();
        for (rx, ry) in damaged_pos {
            engine.spawn_particles(rx, ry, 1, "#f97316", 30.0, 2.2, 0.45);
            engine.spawn_particles(rx, ry, 1, "#475569", 22.0, 3.0, 0.65);
        }
    }

    // Sinh vụ nổ lớn và rung chấn màn hình khi công trình bị sập
    if !engine.is_headless {
        let dead_buildings: Vec<(f32, f32)> = engine
            .buildings
            .iter()
            .filter(|b| b.hp <= 0.0)
            .map(|b| (b.x, b.y))
            .collect();
        for (x, y) in dead_buildings {
            engine.spawn_particles(x, y, 24, "#ea580c", 170.0, 3.8, 0.55);
            engine.spawn_particles(x, y, 14, "#94a3b8", 110.0, 2.8, 0.5);
        }
    }

    // Xóa nhà sập
    engine.buildings.retain(|b| b.hp > 0.0);
}

pub fn process_building_combat(
    b: &mut Building,
    b_cfg: Option<&BuildingStatsConfig>,
    player_tech: &TechTreeState,
    enemy_tech: &TechTreeState,
    enemy_unit_targets: &[(Team, f32, f32)],
    enemy_building_targets: &[(Team, f32, f32)],
    dt: f32,
) -> Option<Projectile> {
    let b_tech = if b.team == Team::Player {
        player_tech
    } else {
        enemy_tech
    };
    process_building_combat_with_factions(
        b,
        b_cfg,
        b_tech,
        None,
        enemy_unit_targets,
        enemy_building_targets,
        dt,
    )
}

pub fn process_building_combat_with_factions(
    b: &mut Building,
    b_cfg: Option<&BuildingStatsConfig>,
    b_tech: &TechTreeState,
    factions: Option<&[u8; 8]>,
    enemy_unit_targets: &[(Team, f32, f32)],
    enemy_building_targets: &[(Team, f32, f32)],
    dt: f32,
) -> Option<Projectile> {
    let range = b_cfg.map(|c| c.turret_range).unwrap_or(0.0);
    let base_damage = b_cfg.map(|c| c.turret_damage).unwrap_or(0.0);

    if range <= 0.0 || base_damage <= 0.0 {
        return None;
    }

    if b.current_cooldown > 0.0 {
        b.current_cooldown -= dt;
        return None;
    }

    let base_cd = b_cfg.map(|c| c.turret_cooldown).unwrap_or(1.0).max(0.1);
    let base_dmg_type = b_cfg.map(|c| c.turret_damage_type).unwrap_or(b.damage_type);
    let base_speed = b_cfg
        .map(|c| c.turret_projectile_speed)
        .unwrap_or(900.0)
        .max(100.0);

    let is_hostile = |t: Team| {
        if let Some(factions) = factions {
            let my_faction = factions[b.team.index() % 8];
            factions[t.index() % 8] != my_faction
        } else {
            t != b.team
        }
    };

    // Ưu tiên ngắm lính đối phương gần nhất, nếu không có mới ngắm công trình đối phương
    let target_pos = enemy_unit_targets
        .iter()
        .find(|(t, x, y)| {
            is_hostile(*t) && ((*x - b.x).powi(2) + (*y - b.y).powi(2)).sqrt() <= range
        })
        .or_else(|| {
            enemy_building_targets.iter().find(|(t, x, y)| {
                is_hostile(*t) && ((*x - b.x).powi(2) + (*y - b.y).powi(2)).sqrt() <= range
            })
        });

    if let Some((_, ex, ey)) = target_pos {
        let has_overload = b_tech.overload_strike;
        b.current_cooldown = if has_overload { base_cd * 0.6 } else { base_cd };

        let has_dark_rockets = b_tech.dark_matter_rockets;

        let spawn_y = b.y - (b.height * 0.25).min(20.0);
        let is_laser = !has_dark_rockets && base_dmg_type == DamageType::Energy;
        let angle = (*ey - spawn_y).atan2(*ex - b.x);
        let visual = if has_dark_rockets {
            ProjectileVisual::Rocket
        } else if is_laser {
            ProjectileVisual::Laser
        } else {
            ProjectileVisual::Bullet
        };

        Some(Projectile {
            id: 0,
            team: b.team,
            x: b.x,
            y: spawn_y,
            prev_x: b.x,
            prev_y: spawn_y,
            target_x: *ex,
            target_y: *ey,
            speed: if has_dark_rockets {
                base_speed * (750.0 / 900.0)
            } else {
                base_speed
            },
            damage: if has_dark_rockets {
                base_damage * 1.4
            } else {
                base_damage
            },
            damage_type: if has_dark_rockets {
                DamageType::Explosive
            } else {
                base_dmg_type
            },
            aoe_radius: if has_dark_rockets { 40.0 } else { 0.0 },
            is_laser,
            lifetime: if has_dark_rockets {
                1.2
            } else if is_laser {
                0.3
            } else {
                1.0
            },
            is_melee: false,
            building_damage_mult: 1.0,
            visual,
            angle,
        })
    } else {
        None
    }
}
