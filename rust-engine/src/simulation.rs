use crate::bridge::*;
use crate::components::*;
use crate::map::{MAP_HEIGHT, MAP_WIDTH};
use crate::GameEngine;

pub fn step(engine: &mut GameEngine, dt: f32) {
    // 1. Xử lý các lệnh từ React
    process_commands(engine);

    if engine.is_paused || engine.game_over {
        return;
    }

    let dt = dt.min(0.05); // Tránh lag giật thời gian lớn
    engine.game_time += dt;

    // 2. Di chuyển Camera bằng phím WASD / Mũi tên và Edge Scrolling
    update_camera(engine, dt);

    // 3. Cập nhật Kinh tế tự nhiên & Đo tốc độ thu nhập
    update_economy(engine, dt);

    // 4. Cập nhật Tiến trình Nghiên cứu Cây Công Nghệ
    update_tech_research(engine, dt);

    // 5. Cập nhật Đơn vị Lính (Di chuyển, Khai mỏ, Tấn công mép nhà, Attack-Move)
    update_units(engine, dt);

    // 6. Cập nhật Công trình (Xây dựng, Huấn luyện lính, Tháp pháo bắn)
    update_buildings(engine, dt);

    // 7. Cập nhật Đạn & Tia Laser (sát thương đơn vị & công trình)
    update_projectiles(engine, dt);

    // 8. Cập nhật Hiệu ứng Chữ nổi (Floating numbers)
    update_floating_texts(engine, dt);

    // 9. Cập nhật Fog of War
    update_fog(engine);

    // 10. Cập nhật AI Đối thủ (kinh tế + điều quân càn quét nhà)
    engine.ai_think_timer += dt;
    if engine.ai_think_timer >= 0.5 {
        engine.ai_think_timer = 0.0;
        crate::ai::update_ai(engine);
    }

    // 11. Kiểm tra Điều kiện Thắng / Thua
    check_game_over(engine);

    // 12. Bắn sự kiện định kỳ đồng bộ về React
    send_sync_event(engine);
}

fn process_commands(engine: &mut GameEngine) {
    while let Some(cmd) = engine.pending_commands.pop_front() {
        match cmd {
            ReactToEngineCommand::StartGame { .. } => {
                engine.is_paused = false;
                engine.game_over = false;
            }
            ReactToEngineCommand::SetPaused { paused } => {
                engine.is_paused = paused;
            }
            ReactToEngineCommand::UpgradeTech { tech_id } => {
                try_start_research(engine, &tech_id);
            }
            ReactToEngineCommand::UpdateSettings {
                sound_volume,
                camera_speed,
                auto_pause,
                show_grid,
            } => {
                engine.settings.sound_volume = sound_volume;
                engine.settings.camera_speed = camera_speed;
                engine.settings.auto_pause_on_tech = auto_pause;
                engine.settings.show_grid = show_grid;
            }
            ReactToEngineCommand::RestartGame => {
                let ctx = engine.context.clone();
                let canvas = engine.canvas.clone();
                *engine = GameEngine::new(canvas, ctx);
            }
        }
    }
}

fn try_start_research(engine: &mut GameEngine, tech_id: &str) {
    if engine.player_tech.current_research.is_some() {
        return;
    }

    let (cost_minerals, cost_energy, time) = match tech_id {
        // Nhánh Khoa học
        "focused_laser" => (250.0, 200.0, 20.0),
        "energy_shielding" => (200.0, 150.0, 16.0),
        "dark_matter_rockets" => (240.0, 160.0, 18.0),
        "titan_alloy_plating" => (280.0, 180.0, 22.0),

        // Nhánh Ma thuật
        "celestial_flying_swords" => (220.0, 160.0, 18.0),
        "golden_bell_shield" => (260.0, 200.0, 22.0),
        "corrosive_curse" => (190.0, 150.0, 16.0),
        "void_cataclysm" => (290.0, 240.0, 25.0),

        // Nhánh Quái vật
        "primeval_scales" => (220.0, 120.0, 18.0),
        "behemoth_mutation" => (320.0, 180.0, 26.0),
        "acid_banelings" => (160.0, 90.0, 14.0),
        "hive_proliferation" => (260.0, 160.0, 20.0),

        // Kinh tế / Phụ trợ
        "optical_drills" => (100.0, 50.0, 10.0),
        "overcharged_reactors" => (150.0, 100.0, 15.0),
        "fortified_blueprints" => (200.0, 150.0, 20.0),
        "magnetic_rounds" => (200.0, 150.0, 15.0),
        "exo_boosters" => (250.0, 200.0, 20.0),
        "superheated_blades" => (350.0, 300.0, 30.0),
        "overload_strike" => (400.0, 350.0, 35.0),
        _ => return,
    };

    if engine.player_eco.minerals >= cost_minerals && engine.player_eco.energy >= cost_energy {
        engine.player_eco.minerals -= cost_minerals;
        engine.player_eco.energy -= cost_energy;
        engine.player_tech.current_research = Some(tech_id.to_string());
        engine.player_tech.research_progress = 0.0;
        engine.player_tech.research_total_time = time;
        engine.outgoing_events.push(EngineToReactEvent::PlaySound {
            sound: "research_start".to_string(),
        });
    }
}

fn update_tech_research(engine: &mut GameEngine, dt: f32) {
    if let Some(ref tech_id) = engine.player_tech.current_research.clone() {
        engine.player_tech.research_progress += dt / engine.player_tech.research_total_time;
        if engine.player_tech.research_progress >= 1.0 {
            match tech_id.as_str() {
                // Khoa học
                "focused_laser" => engine.player_tech.focused_laser = true,
                "energy_shielding" => engine.player_tech.energy_shielding = true,
                "dark_matter_rockets" => engine.player_tech.dark_matter_rockets = true,
                "titan_alloy_plating" => {
                    engine.player_tech.titan_alloy_plating = true;
                    // Tăng thêm máu cho các công trình hiện hữu
                    for b in engine.buildings.iter_mut().filter(|b| b.team == Team::Player) {
                        b.max_hp *= 1.25;
                        b.hp *= 1.25;
                        b.resistances.physical = (b.resistances.physical + 0.20).min(0.80);
                    }
                }

                // Ma thuật
                "celestial_flying_swords" => engine.player_tech.celestial_flying_swords = true,
                "golden_bell_shield" => {
                    engine.player_tech.golden_bell_shield = true;
                    for u in engine.units.iter_mut().filter(|u| u.team == Team::Player) {
                        u.resistances.energy = (u.resistances.energy + 0.25).min(0.80);
                        u.resistances.arcane = (u.resistances.arcane + 0.30).min(0.80);
                    }
                }
                "corrosive_curse" => engine.player_tech.corrosive_curse = true,
                "void_cataclysm" => engine.player_tech.void_cataclysm = true,

                // Quái vật
                "primeval_scales" => {
                    engine.player_tech.primeval_scales = true;
                    for u in engine.units.iter_mut().filter(|u| u.team == Team::Player) {
                        u.resistances.physical = (u.resistances.physical + 0.25).min(0.80);
                        u.resistances.explosive = (u.resistances.explosive + 0.25).min(0.80);
                    }
                }
                "behemoth_mutation" => engine.player_tech.behemoth_mutation = true,
                "acid_banelings" => engine.player_tech.acid_banelings = true,
                "hive_proliferation" => {
                    engine.player_tech.hive_proliferation = true;
                    for u in engine.units.iter_mut().filter(|u| u.team == Team::Player) {
                        u.speed *= 1.25;
                    }
                }

                // Kinh tế / Phụ trợ
                "optical_drills" => engine.player_tech.optical_drills = true,
                "overcharged_reactors" => engine.player_tech.overcharged_reactors = true,
                "fortified_blueprints" => engine.player_tech.fortified_blueprints = true,
                "magnetic_rounds" => engine.player_tech.magnetic_rounds = true,
                "exo_boosters" => engine.player_tech.exo_boosters = true,
                "superheated_blades" => engine.player_tech.superheated_blades = true,
                "overload_strike" => engine.player_tech.overload_strike = true,
                _ => {}
            }
            engine.player_tech.current_research = None;
            engine.player_tech.research_progress = 0.0;
            engine.outgoing_events.push(EngineToReactEvent::PlaySound {
                sound: "research_complete".to_string(),
            });
        }
    }
}

fn update_camera(engine: &mut GameEngine, dt: f32) {
    let speed = 650.0 * engine.settings.camera_speed * dt / engine.camera.zoom;

    // Bàn phím điều khiển
    if engine.keys_down.contains("KeyW") || engine.keys_down.contains("ArrowUp") {
        engine.camera.y -= speed;
    }
    if engine.keys_down.contains("KeyS") || engine.keys_down.contains("ArrowDown") {
        engine.camera.y += speed;
    }
    if engine.keys_down.contains("ArrowLeft")
        || (engine.keys_down.contains("KeyA") && !engine.is_attack_move_mode)
    {
        engine.camera.x -= speed;
    }
    if engine.keys_down.contains("KeyD") || engine.keys_down.contains("ArrowRight") {
        engine.camera.x += speed;
    }

    // Cuộn mép màn hình (Edge-Scrolling) khi không nhấn giữ kéo chuột chọn
    if !engine.is_mouse_down && engine.camera.viewport_w > 0.0 && engine.camera.viewport_h > 0.0 {
        let edge_margin = 18.0;
        let edge_spd = speed * 1.1;
        if engine.mouse_screen_x >= 0.0 && engine.mouse_screen_x <= edge_margin {
            engine.camera.x -= edge_spd;
        } else if engine.mouse_screen_x >= engine.camera.viewport_w - edge_margin
            && engine.mouse_screen_x <= engine.camera.viewport_w
        {
            engine.camera.x += edge_spd;
        }
        if engine.mouse_screen_y >= 0.0 && engine.mouse_screen_y <= edge_margin {
            engine.camera.y -= edge_spd;
        } else if engine.mouse_screen_y >= engine.camera.viewport_h - edge_margin
            && engine.mouse_screen_y <= engine.camera.viewport_h
        {
            engine.camera.y += edge_spd;
        }
    }

    let min_x = (engine.camera.viewport_w * 0.5 / engine.camera.zoom).min(MAP_WIDTH * 0.5);
    let max_x = (MAP_WIDTH - engine.camera.viewport_w * 0.5 / engine.camera.zoom).max(min_x);
    let min_y = (engine.camera.viewport_h * 0.5 / engine.camera.zoom).min(MAP_HEIGHT * 0.5);
    let max_y = (MAP_HEIGHT - engine.camera.viewport_h * 0.5 / engine.camera.zoom).max(min_y);

    engine.camera.x = engine.camera.x.clamp(min_x, max_x);
    engine.camera.y = engine.camera.y.clamp(min_y, max_y);
}

fn update_economy(engine: &mut GameEngine, dt: f32) {
    let mut p_reactors = 0;
    let mut e_reactors = 0;
    let mut p_supply_depots = 0;
    let mut e_supply_depots = 0;

    for b in &engine.buildings {
        if b.build_progress >= 1.0 {
            match (b.team, b.building_type) {
                (Team::Player, BuildingType::PowerReactor) => p_reactors += 1,
                (Team::Enemy, BuildingType::PowerReactor) => e_reactors += 1,
                (Team::Player, BuildingType::SupplyDepot) => p_supply_depots += 1,
                (Team::Enemy, BuildingType::SupplyDepot) => e_supply_depots += 1,
                _ => {}
            }
        }
    }

    let p_mult = if engine.player_tech.overcharged_reactors {
        1.5
    } else {
        1.0
    };
    let player_energy_rate = (2.0 + p_reactors as f32 * 8.0) * p_mult;
    let enemy_energy_rate = 2.0 + e_reactors as f32 * 8.0;

    engine.player_eco.energy += player_energy_rate * dt;
    engine.enemy_eco.energy += enemy_energy_rate * dt;
    engine.player_eco.energy_per_sec = player_energy_rate;

    engine.player_eco.max_supply = 10 + p_supply_depots * 8;
    engine.enemy_eco.max_supply = 10 + e_supply_depots * 8;

    engine.player_eco.supply = engine
        .units
        .iter()
        .filter(|u| u.team == Team::Player)
        .count() as u32;
    engine.enemy_eco.supply = engine
        .units
        .iter()
        .filter(|u| u.team == Team::Enemy)
        .count() as u32;

    // Tính tốc độ thu nhập khoáng sản
    engine.income_calc_timer += dt;
    if engine.income_calc_timer >= 1.0 {
        engine.player_eco.minerals_per_sec =
            engine.minerals_mined_accumulator / engine.income_calc_timer;
        engine.minerals_mined_accumulator = 0.0;
        engine.income_calc_timer = 0.0;
    }
}

fn update_units(engine: &mut GameEngine, dt: f32) {
    let mut new_projectiles = Vec::new();
    let mut new_floating_texts = Vec::new();
    let has_exo = engine.player_tech.exo_boosters;
    let has_superheated = engine.player_tech.superheated_blades;
    let has_magnetic = engine.player_tech.magnetic_rounds;

    // Snapshot vị trí công trình và lính để tra cứu mục tiêu
    // (id, team, x, y, radius)
    let building_targets: Vec<(u32, Team, f32, f32, f32)> = engine
        .buildings
        .iter()
        .map(|b| (b.id, b.team, b.x, b.y, (b.width.max(b.height) * 0.5)))
        .collect();
    let unit_targets: Vec<(u32, Team, f32, f32)> = engine
        .units
        .iter()
        .map(|u| (u.id, u.team, u.x, u.y))
        .collect();
    let mineral_nodes: Vec<(u32, f32, f32)> =
        engine.minerals.iter().map(|m| (m.id, m.x, m.y)).collect();

    for unit in engine.units.iter_mut() {
        if unit.current_cooldown > 0.0 {
            unit.current_cooldown -= dt;
        }

        let mut cur_speed = unit.speed;
        if unit.team == Team::Player && has_exo {
            cur_speed *= 1.2;
        }

        match unit.order {
            UnitOrder::Idle => {
                if unit.unit_type != UnitType::Engineer {
                    let scan_range = unit.attack_range * 1.6;
                    // 1. Ưu tiên tìm lính địch gần nhất
                    let target_unit = unit_targets.iter().find(|(_, t, x, y)| {
                        *t != unit.team
                            && ((*x - unit.x).powi(2) + (*y - unit.y).powi(2)).sqrt() <= scan_range
                    });
                    if let Some((tid, _, _, _)) = target_unit {
                        unit.order = UnitOrder::AttackTarget {
                            target_id: *tid,
                            is_building: false,
                        };
                    } else {
                        // 2. Không có lính địch thì quét công trình địch trong bán kính 300px
                        let target_building = building_targets.iter().find(|(_, t, x, y, r)| {
                            if *t == unit.team {
                                return false;
                            }
                            let dist = ((*x - unit.x).powi(2) + (*y - unit.y).powi(2)).sqrt();
                            (dist - r).max(0.0) <= 300.0
                        });
                        if let Some((bid, _, _, _, _)) = target_building {
                            unit.order = UnitOrder::AttackTarget {
                                target_id: *bid,
                                is_building: true,
                            };
                        }
                    }
                }
            }
            UnitOrder::AttackMove { x, y } => {
                // Tự động tìm mục tiêu lính hoặc nhà trên đường di chuyển
                let scan_range = unit.attack_range * 1.6;
                let target_unit = unit_targets.iter().find(|(_, t, ux, uy)| {
                    *t != unit.team
                        && ((*ux - unit.x).powi(2) + (*uy - unit.y).powi(2)).sqrt() <= scan_range
                });
                if let Some((tid, _, _, _)) = target_unit {
                    unit.order = UnitOrder::AttackTarget {
                        target_id: *tid,
                        is_building: false,
                    };
                } else {
                    let target_building = building_targets.iter().find(|(_, t, bx, by, r)| {
                        if *t == unit.team {
                            return false;
                        }
                        let dist = ((*bx - unit.x).powi(2) + (*by - unit.y).powi(2)).sqrt();
                        (dist - r).max(0.0) <= 260.0
                    });
                    if let Some((bid, _, _, _, _)) = target_building {
                        unit.order = UnitOrder::AttackTarget {
                            target_id: *bid,
                            is_building: true,
                        };
                    } else {
                        let dx = x - unit.x;
                        let dy = y - unit.y;
                        let dist = (dx * dx + dy * dy).sqrt();
                        if dist > 4.0 {
                            let move_dist = (cur_speed * dt).min(dist);
                            unit.x += (dx / dist) * move_dist;
                            unit.y += (dy / dist) * move_dist;
                            unit.walk_cycle += dt * 10.0;
                            unit.facing_right = dx >= 0.0;
                        } else {
                            unit.order = UnitOrder::Idle;
                        }
                    }
                }
            }
            UnitOrder::MoveTo { x, y } => {
                let dx = x - unit.x;
                let dy = y - unit.y;
                let dist = (dx * dx + dy * dy).sqrt();
                if dist > 4.0 {
                    let move_dist = (cur_speed * dt).min(dist);
                    unit.x += (dx / dist) * move_dist;
                    unit.y += (dy / dist) * move_dist;
                    unit.walk_cycle += dt * 10.0;
                    unit.facing_right = dx >= 0.0;
                } else {
                    unit.order = UnitOrder::Idle;
                }
            }
            UnitOrder::Harvest {
                target_mineral_id,
                phase,
            } => {
                match phase {
                    HarvestPhase::MovingToNode => {
                        if let Some((_, mx, my)) = mineral_nodes
                            .iter()
                            .find(|(id, _, _)| *id == target_mineral_id)
                        {
                            let dx = mx - unit.x;
                            let dy = my - unit.y;
                            let dist = (dx * dx + dy * dy).sqrt();
                            if dist > 35.0 {
                                let move_dist = (cur_speed * dt).min(dist);
                                unit.x += (dx / dist) * move_dist;
                                unit.y += (dy / dist) * move_dist;
                                unit.walk_cycle += dt * 10.0;
                                unit.facing_right = dx >= 0.0;
                            } else {
                                unit.order = UnitOrder::Harvest {
                                    target_mineral_id,
                                    phase: HarvestPhase::Mining(0.0),
                                };
                            }
                        } else {
                            // Mỏ không còn -> Tìm mỏ gần nhất kế tiếp
                            let nearest = mineral_nodes.iter().min_by(|a, b| {
                                let da = (a.1 - unit.x).powi(2) + (a.2 - unit.y).powi(2);
                                let db = (b.1 - unit.x).powi(2) + (b.2 - unit.y).powi(2);
                                da.partial_cmp(&db).unwrap()
                            });
                            if let Some((nid, _, _)) = nearest {
                                unit.order = UnitOrder::Harvest {
                                    target_mineral_id: *nid,
                                    phase: HarvestPhase::MovingToNode,
                                };
                            } else {
                                unit.order = UnitOrder::Idle;
                            }
                        }
                    }
                    HarvestPhase::Mining(mut t) => {
                        let mining_duration =
                            if unit.team == Team::Player && engine.player_tech.optical_drills {
                                1.4
                            } else {
                                2.0
                            };
                        t += dt;
                        if t >= mining_duration {
                            unit.carried_minerals =
                                if unit.team == Team::Player && engine.player_tech.optical_drills {
                                    15.0
                                } else {
                                    10.0
                                };
                            // Trừ trữ lượng mỏ quặng
                            if let Some(m) = engine
                                .minerals
                                .iter_mut()
                                .find(|m| m.id == target_mineral_id)
                            {
                                m.amount -= unit.carried_minerals;
                            }
                            unit.order = UnitOrder::Harvest {
                                target_mineral_id,
                                phase: HarvestPhase::ReturningToBase,
                            };
                        } else {
                            unit.order = UnitOrder::Harvest {
                                target_mineral_id,
                                phase: HarvestPhase::Mining(t),
                            };
                        }
                    }
                    HarvestPhase::ReturningToBase => {
                        // Tìm Command Center gần nhất của phe mình
                        let my_cc = building_targets
                            .iter()
                            .filter(|(_, t, _, _, _)| *t == unit.team)
                            .min_by(|a, b| {
                                let da = (a.2 - unit.x).powi(2) + (a.3 - unit.y).powi(2);
                                let db = (b.2 - unit.x).powi(2) + (b.3 - unit.y).powi(2);
                                da.partial_cmp(&db).unwrap()
                            });

                        if let Some((_, _, cx, cy, _)) = my_cc {
                            let dx = cx - unit.x;
                            let dy = cy - unit.y;
                            let dist = (dx * dx + dy * dy).sqrt();
                            if dist > 55.0 {
                                let move_dist = (cur_speed * dt).min(dist);
                                unit.x += (dx / dist) * move_dist;
                                unit.y += (dy / dist) * move_dist;
                                unit.walk_cycle += dt * 10.0;
                                unit.facing_right = dx >= 0.0;
                            } else {
                                // Giao hàng thành công!
                                if unit.team == Team::Player {
                                    engine.player_eco.minerals += unit.carried_minerals;
                                    engine.minerals_mined_accumulator += unit.carried_minerals;
                                    new_floating_texts.push(FloatingText {
                                        id: 0,
                                        text: format!("+{} 💎", unit.carried_minerals as u32),
                                        x: unit.x,
                                        y: unit.y - 15.0,
                                        color: "#00e5ff".to_string(),
                                        lifetime: 1.0,
                                        max_lifetime: 1.0,
                                    });
                                } else {
                                    engine.enemy_eco.minerals += unit.carried_minerals;
                                }
                                unit.carried_minerals = 0.0;
                                unit.order = UnitOrder::Harvest {
                                    target_mineral_id,
                                    phase: HarvestPhase::MovingToNode,
                                };
                            }
                        }
                    }
                }
            }
            UnitOrder::Build { target_building_id } => {
                if let Some((_, _, bx, by, _)) = building_targets
                    .iter()
                    .find(|(id, _, _, _, _)| *id == target_building_id)
                {
                    let dx = bx - unit.x;
                    let dy = by - unit.y;
                    let dist = (dx * dx + dy * dy).sqrt();
                    if dist > 60.0 {
                        let move_dist = (cur_speed * dt).min(dist);
                        unit.x += (dx / dist) * move_dist;
                        unit.y += (dy / dist) * move_dist;
                        unit.walk_cycle += dt * 10.0;
                        unit.facing_right = dx >= 0.0;
                    }
                } else {
                    unit.order = UnitOrder::Idle;
                }
            }
            UnitOrder::AttackTarget {
                target_id,
                is_building,
            } => {
                let target_info = if is_building {
                    building_targets
                        .iter()
                        .find(|(id, t, _, _, _)| *id == target_id && *t != unit.team)
                        .map(|(_, _, x, y, r)| (*x, *y, *r))
                } else {
                    unit_targets
                        .iter()
                        .find(|(id, t, _, _)| *id == target_id && *t != unit.team)
                        .map(|(_, _, x, y)| (*x, *y, 10.0))
                };

                if let Some((tx, ty, target_radius)) = target_info {
                    let dx = tx - unit.x;
                    let dy = ty - unit.y;
                    let dist = (dx * dx + dy * dy).sqrt();
                    unit.facing_right = dx >= 0.0;

                    // KHOẢNG CÁCH TÍNH TỚI MÉP MỤC TIÊU
                    let effective_dist = (dist - target_radius).max(0.0);

                    if effective_dist > unit.attack_range {
                        // Di chuyển tiếp cận mép mục tiêu
                        let move_dist =
                            (cur_speed * dt).min(effective_dist - unit.attack_range + 5.0);
                        unit.x += (dx / dist) * move_dist;
                        unit.y += (dy / dist) * move_dist;
                        unit.walk_cycle += dt * 10.0;
                    } else {
                        // TẤN CÔNG ĐÍCH XÁC
                        if unit.current_cooldown <= 0.0 {
                            unit.current_cooldown = unit.attack_cooldown;
                            let mut dmg = unit.attack_damage;
                            if unit.team == Team::Player {
                                if unit.unit_type == UnitType::Rifleman && has_magnetic {
                                    dmg *= 1.25;
                                }
                                if unit.unit_type == UnitType::ShockTrooper && has_superheated {
                                    dmg *= 1.35;
                                }
                            }

                            // Điểm va chạm nằm ở mép mục tiêu
                            let hit_dist = dist.min(unit.attack_range + target_radius);
                            let hit_x = if dist > 0.0 {
                                unit.x + (dx / dist) * hit_dist
                            } else {
                                tx
                            };
                            let hit_y = if dist > 0.0 {
                                unit.y + (dy / dist) * hit_dist
                            } else {
                                ty
                            };

                            if unit.unit_type == UnitType::AcidBaneling {
                                // Bọ nổ cảm tử: nổ tung ngay lập tức
                                unit.hp = 0.0;
                                new_projectiles.push(Projectile {
                                    id: 0,
                                    team: unit.team,
                                    x: unit.x,
                                    y: unit.y,
                                    target_x: hit_x,
                                    target_y: hit_y,
                                    speed: 1500.0,
                                    damage: dmg,
                                    damage_type: DamageType::Explosive,
                                    aoe_radius: unit.aoe_radius,
                                    is_laser: false,
                                    lifetime: 0.1,
                                });
                            } else if unit.unit_type == UnitType::ShockTrooper
                                || unit.unit_type == UnitType::Vanguard
                                || unit.unit_type == UnitType::Engineer
                                || unit.unit_type == UnitType::BehemothTitan
                            {
                                // Cận chiến: tạo vệt chém hoặc dậm chấn động
                                new_projectiles.push(Projectile {
                                    id: 0,
                                    team: unit.team,
                                    x: unit.x,
                                    y: unit.y,
                                    target_x: hit_x,
                                    target_y: hit_y,
                                    speed: 900.0,
                                    damage: dmg,
                                    damage_type: unit.damage_type,
                                    aoe_radius: unit.aoe_radius,
                                    is_laser: false,
                                    lifetime: 0.15,
                                });
                            } else {
                                // Tầm xa
                                let is_laser = unit.unit_type == UnitType::LaserMarksman;
                                let speed = if is_laser {
                                    1800.0
                                } else if unit.unit_type == UnitType::TaoistSwordmaster {
                                    1100.0
                                } else if unit.unit_type == UnitType::DarkRocketeer {
                                    650.0
                                } else {
                                    750.0
                                };
                                new_projectiles.push(Projectile {
                                    id: 0,
                                    team: unit.team,
                                    x: unit.x,
                                    y: unit.y - 12.0,
                                    target_x: hit_x,
                                    target_y: hit_y,
                                    speed,
                                    damage: dmg,
                                    damage_type: unit.damage_type,
                                    aoe_radius: unit.aoe_radius,
                                    is_laser,
                                    lifetime: if is_laser { 0.25 } else { 1.5 },
                                });
                            }
                        }
                    }
                } else {
                    unit.order = UnitOrder::Idle;
                }
            }
        }
    }

    // Xóa lính chết do tự nổ
    engine.units.retain(|u| u.hp > 0.0);

    // Hồi máu linh quang từ Kim cang hộ thể (3 HP/s)
    if engine.player_tech.golden_bell_shield {
        for u in engine.units.iter_mut() {
            if u.team == Team::Player && u.hp > 0.0 && u.hp < u.max_hp {
                u.hp = (u.hp + 3.0 * dt).min(u.max_hp);
            }
        }
    }

    for mut p in new_projectiles {
        p.id = engine.alloc_id();
        engine.projectiles.push(p);
    }

    for mut ft in new_floating_texts {
        ft.id = engine.alloc_id();
        engine.floating_texts.push(ft);
    }

    // Xóa quặng đã khai thác hết
    engine.minerals.retain(|m| m.amount > 0.0);

    // Xóa lính tử trận
    engine.units.retain(|u| u.hp > 0.0);
}

fn update_buildings(engine: &mut GameEngine, dt: f32) {
    let mut new_spawned_units = Vec::new();
    let mut new_turret_projectiles = Vec::new();

    let engineer_pos: Vec<(Team, f32, f32)> = engine
        .units
        .iter()
        .filter(|u| u.unit_type == UnitType::Engineer)
        .map(|u| (u.team, u.x, u.y))
        .collect();

    let enemy_targets: Vec<(Team, f32, f32)> =
        engine.units.iter().map(|u| (u.team, u.x, u.y)).collect();

    for b in engine.buildings.iter_mut() {
        if b.build_progress < 1.0 {
            let is_worker_near = engineer_pos.iter().any(|(t, x, y)| {
                *t == b.team && ((*x - b.x).powi(2) + (*y - b.y).powi(2)).sqrt() <= 90.0
            });
            let build_speed = if is_worker_near { 0.2 } else { 0.05 };
            b.build_progress = (b.build_progress + build_speed * dt).min(1.0);
            b.hp = b.max_hp * b.build_progress;
        } else {
            // Huấn luyện lính
            if !b.training_queue.is_empty() {
                b.training_timer += dt;
                let mut required_time = b.training_queue[0].training_time();
                if b.team == Team::Player && engine.player_tech.hive_proliferation {
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

            // Tháp pháo phòng thủ tự động
            if b.building_type == BuildingType::TurretTower {
                if b.current_cooldown > 0.0 {
                    b.current_cooldown -= dt;
                } else {
                    let range = 250.0;
                    if let Some((_, ex, ey)) = enemy_targets.iter().find(|(t, x, y)| {
                        *t != b.team && ((*x - b.x).powi(2) + (*y - b.y).powi(2)).sqrt() <= range
                    }) {
                        b.current_cooldown =
                            if b.team == Team::Player && engine.player_tech.overload_strike {
                                0.6
                            } else {
                                1.0
                            };
                        let is_dark_rocket = b.team == Team::Player && engine.player_tech.dark_matter_rockets;
                        new_turret_projectiles.push(Projectile {
                            id: 0,
                            team: b.team,
                            x: b.x,
                            y: b.y - 20.0,
                            target_x: *ex,
                            target_y: *ey,
                            speed: if is_dark_rocket { 750.0 } else { 900.0 },
                            damage: if is_dark_rocket { 35.0 } else { 25.0 },
                            damage_type: if is_dark_rocket {
                                DamageType::Explosive
                            } else {
                                DamageType::Energy
                            },
                            aoe_radius: if is_dark_rocket { 40.0 } else { 0.0 },
                            is_laser: !is_dark_rocket,
                            lifetime: if is_dark_rocket { 1.2 } else { 0.3 },
                        });
                    }
                }
            }
        }
    }

    for (team, u_type, sx, sy, rx, ry) in new_spawned_units {
        let u_id = engine.alloc_id();
        let (speed, hp, range, dmg, cd, dmg_type, aoe, mut res) = match u_type {
            UnitType::Engineer => (
                120.0,
                90.0,
                30.0,
                5.0,
                1.0,
                DamageType::Physical,
                0.0,
                Resistances {
                    physical: 0.05,
                    energy: 0.0,
                    explosive: 0.0,
                    arcane: 0.0,
                },
            ),
            UnitType::ShockTrooper => {
                let hp = if team == Team::Player && engine.player_tech.energy_shielding {
                    190.0
                } else {
                    160.0
                };
                (
                    140.0,
                    hp,
                    40.0,
                    22.0,
                    0.8,
                    DamageType::Energy,
                    0.0,
                    Resistances {
                        physical: 0.15,
                        energy: 0.20,
                        explosive: 0.05,
                        arcane: 0.10,
                    },
                )
            }
            UnitType::Rifleman => (
                110.0,
                120.0,
                160.0,
                14.0,
                1.1,
                DamageType::Physical,
                0.0,
                Resistances {
                    physical: 0.10,
                    energy: 0.05,
                    explosive: 0.05,
                    arcane: 0.05,
                },
            ),
            UnitType::Vanguard => (
                90.0,
                320.0,
                45.0,
                18.0,
                1.2,
                DamageType::Physical,
                0.0,
                Resistances {
                    physical: 0.40,
                    energy: 0.50,
                    explosive: 0.25,
                    arcane: 0.15,
                },
            ),
            UnitType::LaserMarksman => (
                95.0,
                110.0,
                220.0,
                38.0,
                1.6,
                DamageType::Energy,
                0.0,
                Resistances {
                    physical: 0.05,
                    energy: 0.30,
                    explosive: 0.0,
                    arcane: 0.15,
                },
            ),
            UnitType::DarkRocketeer => (
                95.0,
                140.0,
                200.0,
                32.0,
                1.8,
                DamageType::Explosive,
                40.0,
                Resistances {
                    physical: 0.20,
                    energy: 0.10,
                    explosive: 0.35,
                    arcane: 0.05,
                },
            ),
            UnitType::TaoistSwordmaster => (
                130.0,
                170.0,
                140.0,
                28.0,
                1.0,
                DamageType::Arcane,
                0.0,
                Resistances {
                    physical: 0.15,
                    energy: 0.35,
                    explosive: 0.15,
                    arcane: 0.45,
                },
            ),
            UnitType::VoidWarlock => (
                100.0,
                135.0,
                175.0,
                30.0,
                1.5,
                DamageType::Arcane,
                35.0,
                Resistances {
                    physical: 0.10,
                    energy: 0.25,
                    explosive: 0.20,
                    arcane: 0.50,
                },
            ),
            UnitType::BehemothTitan => (
                80.0,
                650.0,
                55.0,
                48.0,
                1.4,
                DamageType::Physical,
                45.0,
                Resistances {
                    physical: 0.50,
                    energy: 0.20,
                    explosive: 0.45,
                    arcane: 0.10,
                },
            ),
            UnitType::AcidBaneling => (
                160.0,
                85.0,
                25.0,
                75.0,
                0.5,
                DamageType::Explosive,
                65.0,
                Resistances {
                    physical: 0.05,
                    energy: 0.05,
                    explosive: 0.20,
                    arcane: 0.20,
                },
            ),
        };

        let mut final_speed = speed;
        if team == Team::Player {
            if engine.player_tech.exo_boosters || engine.player_tech.hive_proliferation {
                final_speed *= 1.25;
            }
            if engine.player_tech.titan_alloy_plating {
                res.physical = (res.physical + 0.25).min(0.80);
            }
            if engine.player_tech.golden_bell_shield {
                res.energy = (res.energy + 0.25).min(0.80);
                res.arcane = (res.arcane + 0.30).min(0.80);
            }
            if engine.player_tech.primeval_scales
                && (u_type == UnitType::ShockTrooper
                    || u_type == UnitType::Vanguard
                    || u_type == UnitType::BehemothTitan
                    || u_type == UnitType::AcidBaneling)
            {
                res.physical = (res.physical + 0.25).min(0.80);
                res.explosive = (res.explosive + 0.25).min(0.80);
            }
        }

        engine.units.push(Unit {
            id: u_id,
            team,
            unit_type: u_type,
            x: sx,
            y: sy,
            target_x: rx,
            target_y: ry,
            speed: final_speed,
            hp,
            max_hp: hp,
            attack_range: range,
            attack_damage: dmg,
            damage_type: dmg_type,
            aoe_radius: aoe,
            resistances: res,
            attack_cooldown: cd,
            current_cooldown: 0.0,
            walk_cycle: 0.0,
            facing_right: rx >= sx,
            order: UnitOrder::MoveTo { x: rx, y: ry },
            carried_minerals: 0.0,
        });
    }

    for mut p in new_turret_projectiles {
        p.id = engine.alloc_id();
        engine.projectiles.push(p);
    }

    // Xóa nhà sập
    engine.buildings.retain(|b| b.hp > 0.0);
}

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

fn update_projectiles(engine: &mut GameEngine, dt: f32) {
    let has_corrosive = engine.player_tech.corrosive_curse;
    let mut new_texts = Vec::new();

    for p in engine.projectiles.iter_mut() {
        p.lifetime -= dt;
        let dx = p.target_x - p.x;
        let dy = p.target_y - p.y;
        let dist = (dx * dx + dy * dy).sqrt();
        if dist > 10.0 {
            let step = (p.speed * dt).min(dist);
            p.x += (dx / dist) * step;
            p.y += (dy / dist) * step;
        }

        if dist <= 18.0 || p.lifetime <= 0.0 {
            let is_aoe = p.aoe_radius > 0.0;
            let aoe_range = if is_aoe { p.aoe_radius.max(30.0) } else { 24.0 };

            // Sát thương lính địch
            for u in engine.units.iter_mut() {
                if u.team != p.team {
                    let d = ((u.x - p.target_x).powi(2) + (u.y - p.target_y).powi(2)).sqrt();
                    if d <= aoe_range {
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
                        u.hp -= taken;

                        if p.team == Team::Player && has_corrosive {
                            u.resistances.physical = (u.resistances.physical - 0.04).max(0.0);
                            u.resistances.energy = (u.resistances.energy - 0.04).max(0.0);
                            u.resistances.explosive = (u.resistances.explosive - 0.04).max(0.0);
                            u.resistances.arcane = (u.resistances.arcane - 0.04).max(0.0);
                        }

                        new_texts.push(FloatingText {
                            id: 0,
                            text: format!("-{:.0}", taken),
                            x: u.x,
                            y: u.y - 12.0,
                            color: p.damage_type.color_code().to_string(),
                            lifetime: 0.7,
                            max_lifetime: 0.7,
                        });

                        if !is_aoe {
                            break;
                        }
                    }
                }
            }

            // Sát thương công trình địch
            for b in engine.buildings.iter_mut() {
                if b.team != p.team {
                    let b_rad = b.width.max(b.height) * 0.5;
                    let dist_to_b =
                        ((b.x - p.target_x).powi(2) + (b.y - p.target_y).powi(2)).sqrt();
                    if dist_to_b <= b_rad + aoe_range {
                        let taken = calculate_effective_damage(
                            p.damage,
                            p.damage_type,
                            &b.resistances,
                        );
                        b.hp -= taken;

                        new_texts.push(FloatingText {
                            id: 0,
                            text: format!("-{:.0}", taken),
                            x: b.x,
                            y: b.y - b_rad,
                            color: p.damage_type.color_code().to_string(),
                            lifetime: 0.8,
                            max_lifetime: 0.8,
                        });

                        if !is_aoe {
                            break;
                        }
                    }
                }
            }

            p.lifetime = -1.0;
        }
    }

    for mut ft in new_texts {
        ft.id = engine.alloc_id();
        engine.floating_texts.push(ft);
    }

    engine.projectiles.retain(|p| p.lifetime > 0.0);
}

fn update_floating_texts(engine: &mut GameEngine, dt: f32) {
    for ft in engine.floating_texts.iter_mut() {
        ft.lifetime -= dt;
        ft.y -= 25.0 * dt;
    }
    engine.floating_texts.retain(|ft| ft.lifetime > 0.0);
}

fn update_fog(engine: &mut GameEngine) {
    engine.fog.reset_visibility_to_explored();

    // Tầm nhìn quanh công trình
    for b in &engine.buildings {
        if b.team == Team::Player {
            let r = if b.building_type == BuildingType::TurretTower {
                320.0
            } else {
                240.0
            };
            engine.fog.reveal_circle(b.x, b.y, r);
        }
    }

    // Tầm nhìn quanh lính
    for u in &engine.units {
        if u.team == Team::Player {
            let r = match u.unit_type {
                UnitType::LaserMarksman => 260.0,
                UnitType::Rifleman => 200.0,
                _ => 170.0,
            };
            engine.fog.reveal_circle(u.x, u.y, r);
        }
    }
}

fn check_game_over(engine: &mut GameEngine) {
    let player_has_cc = engine
        .buildings
        .iter()
        .any(|b| b.team == Team::Player && b.building_type == BuildingType::CommandCenter);
    let enemy_has_cc = engine
        .buildings
        .iter()
        .any(|b| b.team == Team::Enemy && b.building_type == BuildingType::CommandCenter);

    if !player_has_cc {
        engine.game_over = true;
        engine.outgoing_events.push(EngineToReactEvent::GameOver {
            player_won: false,
            stats_json: "{}".to_string(),
        });
        engine.outgoing_events.push(EngineToReactEvent::PlaySound {
            sound: "defeat".to_string(),
        });
    } else if !enemy_has_cc {
        engine.game_over = true;
        engine.outgoing_events.push(EngineToReactEvent::GameOver {
            player_won: true,
            stats_json: "{}".to_string(),
        });
        engine.outgoing_events.push(EngineToReactEvent::PlaySound {
            sound: "victory".to_string(),
        });
    }
}

fn send_sync_event(engine: &mut GameEngine) {
    let tech_json = serde_json::to_string(&engine.player_tech).unwrap_or_default();
    engine.outgoing_events.push(EngineToReactEvent::SyncState {
        player_minerals: engine.player_eco.minerals,
        player_energy: engine.player_eco.energy,
        player_supply: engine.player_eco.supply,
        player_max_supply: engine.player_eco.max_supply,
        tech_state: tech_json,
        game_time: engine.game_time,
        is_paused: engine.is_paused,
    });
}

// Xử lý Input từ Chuột và Phím
pub fn handle_mouse_down(engine: &mut GameEngine, button: i32, cx: f32, cy: f32) {
    let vh = engine.camera.viewport_h;

    // Kiểm tra bấm vào Minimap để kéo thả camera
    if (10.0..=170.0).contains(&cx) && (vh - 170.0..=vh - 10.0).contains(&cy) {
        engine.is_minimap_dragging = true;
        let norm_x = ((cx - 10.0) / 160.0).clamp(0.0, 1.0);
        let norm_y = ((cy - (vh - 170.0)) / 160.0).clamp(0.0, 1.0);
        engine.camera.x = norm_x * MAP_WIDTH;
        engine.camera.y = norm_y * MAP_HEIGHT;
        return;
    }

    if button == 0 {
        // Chuột trái
        if engine.is_attack_move_mode {
            engine.is_attack_move_mode = false;
            let (wx, wy) = engine.screen_to_world(cx, cy);
            let count = engine.selected_unit_ids.len();
            for (i, uid) in engine.selected_unit_ids.iter().enumerate() {
                if let Some(u) = engine
                    .units
                    .iter_mut()
                    .find(|u| u.id == *uid && u.team == Team::Player)
                {
                    let offset_x = ((i % 5) as f32 - (count.min(5) as f32 * 0.5)) * 20.0;
                    let offset_y = ((i / 5) as f32) * 20.0;
                    u.order = UnitOrder::AttackMove {
                        x: wx + offset_x,
                        y: wy + offset_y,
                    };
                }
            }
            engine.outgoing_events.push(EngineToReactEvent::PlaySound {
                sound: "attack_order".to_string(),
            });
            return;
        }

        engine.is_mouse_down = true;
        engine.mouse_down_start = (cx, cy);
        engine.selection_box = Some((cx, cy, cx, cy));

        // Kiểm tra xem có bấm vào các nút trên HUD của Bevy không
        if check_bevy_hud_click(engine, cx, cy) {
            engine.selection_box = None;
            engine.is_mouse_down = false;
        }
    } else if button == 2 {
        let vw = engine.camera.viewport_w;
        let vh = engine.camera.viewport_h;

        // Bấm chuột phải trên Command Card để hủy bớt lính trong hàng đợi và hoàn tiền
        if cx >= vw - 220.0 && cx <= vw - 10.0 && cy >= vh - 140.0 && cy <= vh - 10.0 {
            handle_command_card_right_click(engine, cx - (vw - 220.0), cy - (vh - 140.0));
            return;
        }

        // Chuột phải: Hủy Attack-Move hoặc ra lệnh di chuyển/tấn công/khai thác
        engine.is_attack_move_mode = false;
        let (wx, wy) = engine.screen_to_world(cx, cy);
        issue_right_click_order(engine, wx, wy);
    }
}

pub fn handle_mouse_up(engine: &mut GameEngine, button: i32, cx: f32, cy: f32) {
    if engine.is_minimap_dragging {
        engine.is_minimap_dragging = false;
        return;
    }

    if button == 0 && engine.is_mouse_down {
        engine.is_mouse_down = false;
        let (sx1, sy1) = engine.mouse_down_start;
        let (sx2, cy) = (cx, cy);

        let dx = (sx2 - sx1).abs();
        let dy = (cy - sy1).abs();

        if let Some(b_type) = engine.build_mode {
            // Đặt móng xây nhà
            let (wx, wy) = engine.screen_to_world(cx, cy);
            try_place_building(engine, b_type, wx, wy);
            engine.build_mode = None;
        } else if dx > 8.0 || dy > 8.0 {
            // Box selection
            let (w_min_x, w_min_y) = engine.screen_to_world(sx1.min(sx2), sy1.min(cy));
            let (w_max_x, w_max_y) = engine.screen_to_world(sx1.max(sx2), sy1.max(cy));

            engine.selected_unit_ids = engine
                .units
                .iter()
                .filter(|u| {
                    u.team == Team::Player
                        && u.x >= w_min_x
                        && u.x <= w_max_x
                        && u.y >= w_min_y
                        && u.y <= w_max_y
                })
                .map(|u| u.id)
                .collect();
            engine.selected_building_id = None;
        } else {
            // Single click selection
            let (wx, wy) = engine.screen_to_world(cx, cy);
            select_at_point(engine, wx, wy);
        }

        engine.selection_box = None;
    }
}

pub fn handle_mouse_move(engine: &mut GameEngine, cx: f32, cy: f32) {
    engine.mouse_screen_x = cx;
    engine.mouse_screen_y = cy;

    let vh = engine.camera.viewport_h;
    let vw = engine.camera.viewport_w;

    if engine.is_minimap_dragging {
        let norm_x = ((cx - 10.0) / 160.0).clamp(0.0, 1.0);
        let norm_y = ((cy - (vh - 170.0)) / 160.0).clamp(0.0, 1.0);
        engine.camera.x = norm_x * MAP_WIDTH;
        engine.camera.y = norm_y * MAP_HEIGHT;
        return;
    }

    if engine.is_mouse_down {
        let (sx1, sy1) = engine.mouse_down_start;
        engine.selection_box = Some((sx1, sy1, cx, cy));
    }

    // Hover trên Command Card
    if cx >= vw - 225.0 && cx <= vw - 10.0 && cy >= vh - 150.0 && cy <= vh - 10.0 {
        let rel_x = cx - (vw - 225.0);
        let rel_y = cy - (vh - 150.0);
        let col = ((rel_x - 5.0).max(0.0) / 67.0).min(2.0) as usize;
        let row = ((rel_y - 6.0).max(0.0) / 44.0).min(2.0) as usize;
        engine.hovered_command_button = Some(row * 3 + col);
    } else {
        engine.hovered_command_button = None;
    }
}

pub fn handle_hotkeys(engine: &mut GameEngine, key: &str) {
    match key {
        "KeyA" => {
            // Bật chế độ Attack-Move
            if !engine.selected_unit_ids.is_empty() {
                engine.is_attack_move_mode = true;
                engine.outgoing_events.push(EngineToReactEvent::PlaySound {
                    sound: "click".to_string(),
                });
            }
        }
        "KeyT" => {
            // Mở Modal Cây Công Nghệ trong React
            if engine.settings.auto_pause_on_tech {
                engine.is_paused = true;
            }
            engine
                .outgoing_events
                .push(EngineToReactEvent::OpenTechTreeModal);
        }
        "Escape" => {
            if engine.is_attack_move_mode {
                engine.is_attack_move_mode = false;
            } else if engine.build_mode.is_some() {
                engine.build_mode = None;
            } else {
                engine
                    .outgoing_events
                    .push(EngineToReactEvent::OpenSettingsModal);
            }
        }
        "Space" => {
            // Tạm dừng / Tiếp tục
            engine.is_paused = !engine.is_paused;
        }
        _ => {}
    }
}

fn select_at_point(engine: &mut GameEngine, wx: f32, wy: f32) {
    // Nếu đang chọn Worker và click chuột trái vào mỏ khoáng sản -> Hỗ trợ ra lệnh khai mỏ luôn
    let has_worker_selected = engine.selected_unit_ids.iter().any(|uid| {
        engine
            .units
            .iter()
            .any(|u| u.id == *uid && u.unit_type == UnitType::Engineer && u.team == Team::Player)
    });

    if has_worker_selected {
        let clicked_mineral = engine
            .minerals
            .iter()
            .find(|m| ((m.x - wx).powi(2) + (m.y - wy).powi(2)).sqrt() <= 50.0);
        if let Some(m) = clicked_mineral {
            let m_id = m.id;
            for uid in &engine.selected_unit_ids {
                if let Some(u) = engine
                    .units
                    .iter_mut()
                    .find(|u| u.id == *uid && u.team == Team::Player)
                {
                    if u.unit_type == UnitType::Engineer {
                        u.order = UnitOrder::Harvest {
                            target_mineral_id: m_id,
                            phase: HarvestPhase::MovingToNode,
                        };
                    }
                }
            }
            engine.outgoing_events.push(EngineToReactEvent::PlaySound {
                sound: "order".to_string(),
            });
            return;
        }
    }

    // 1. Kiểm tra bấm vào lính Player (dung sai 24px)
    let clicked_unit = engine
        .units
        .iter()
        .find(|u| ((u.x - wx).powi(2) + (u.y - wy).powi(2)).sqrt() <= 24.0);

    if let Some(u) = clicked_unit {
        engine.selected_unit_ids = vec![u.id];
        engine.selected_building_id = None;
        return;
    }

    // 2. Kiểm tra bấm vào công trình Player (có dung sai)
    let clicked_building = engine.buildings.iter().find(|b| {
        (wx - b.x).abs() <= b.width * 0.5 + 8.0 && (wy - b.y).abs() <= b.height * 0.5 + 8.0
    });

    if let Some(b) = clicked_building {
        engine.selected_building_id = Some(b.id);
        engine.selected_unit_ids.clear();
        return;
    }

    // Bấm ra bãi đất trống
    engine.selected_unit_ids.clear();
    engine.selected_building_id = None;
}

fn issue_right_click_order(engine: &mut GameEngine, wx: f32, wy: f32) {
    if engine.selected_unit_ids.is_empty() {
        if let Some(bid) = engine.selected_building_id {
            if let Some(b) = engine
                .buildings
                .iter_mut()
                .find(|b| b.id == bid && b.team == Team::Player)
            {
                b.rally_x = wx;
                b.rally_y = wy;
                engine.outgoing_events.push(EngineToReactEvent::PlaySound {
                    sound: "click".to_string(),
                });
            }
        }
        return;
    }

    // 1. Kiểm tra bấm vào mỏ khoáng sản (dung sai 55px)
    let clicked_mineral = engine
        .minerals
        .iter()
        .find(|m| ((m.x - wx).powi(2) + (m.y - wy).powi(2)).sqrt() <= 55.0);

    if let Some(m) = clicked_mineral {
        let m_id = m.id;
        for uid in &engine.selected_unit_ids {
            if let Some(u) = engine
                .units
                .iter_mut()
                .find(|u| u.id == *uid && u.team == Team::Player)
            {
                if u.unit_type == UnitType::Engineer {
                    u.order = UnitOrder::Harvest {
                        target_mineral_id: m_id,
                        phase: HarvestPhase::MovingToNode,
                    };
                }
            }
        }
        engine.outgoing_events.push(EngineToReactEvent::PlaySound {
            sound: "order".to_string(),
        });
        return;
    }

    // 2. Kiểm tra bấm vào nhà đang xây dở của mình để thợ mỏ đến xây
    let unfinished_building = engine.buildings.iter().find(|b| {
        b.team == Team::Player
            && b.build_progress < 1.0
            && (wx - b.x).abs() <= b.width * 0.5 + 10.0
            && (wy - b.y).abs() <= b.height * 0.5 + 10.0
    });

    if let Some(b) = unfinished_building {
        let bid = b.id;
        for uid in &engine.selected_unit_ids {
            if let Some(u) = engine
                .units
                .iter_mut()
                .find(|u| u.id == *uid && u.team == Team::Player)
            {
                if u.unit_type == UnitType::Engineer {
                    u.order = UnitOrder::Build {
                        target_building_id: bid,
                    };
                }
            }
        }
        engine.outgoing_events.push(EngineToReactEvent::PlaySound {
            sound: "order".to_string(),
        });
        return;
    }

    // 3. Kiểm tra bấm vào lính địch (Tấn công)
    let enemy_unit = engine.units.iter().find(|u| {
        u.team == Team::Enemy && ((u.x - wx).powi(2) + (u.y - wy).powi(2)).sqrt() <= 26.0
    });
    if let Some(eu) = enemy_unit {
        let eid = eu.id;
        for uid in &engine.selected_unit_ids {
            if let Some(u) = engine
                .units
                .iter_mut()
                .find(|u| u.id == *uid && u.team == Team::Player)
            {
                u.order = UnitOrder::AttackTarget {
                    target_id: eid,
                    is_building: false,
                };
            }
        }
        engine.outgoing_events.push(EngineToReactEvent::PlaySound {
            sound: "attack_order".to_string(),
        });
        return;
    }

    // 4. Kiểm tra bấm vào CÔNG TRÌNH ĐỊCH (Tấn công nhà địch hoàn chỉnh)
    let enemy_b = engine.buildings.iter().find(|b| {
        b.team == Team::Enemy
            && (wx - b.x).abs() <= b.width * 0.5 + 12.0
            && (wy - b.y).abs() <= b.height * 0.5 + 12.0
    });
    if let Some(eb) = enemy_b {
        let ebid = eb.id;
        for uid in &engine.selected_unit_ids {
            if let Some(u) = engine
                .units
                .iter_mut()
                .find(|u| u.id == *uid && u.team == Team::Player)
            {
                u.order = UnitOrder::AttackTarget {
                    target_id: ebid,
                    is_building: true,
                };
            }
        }
        engine.outgoing_events.push(EngineToReactEvent::PlaySound {
            sound: "attack_order".to_string(),
        });
        return;
    }

    // 5. Di chuyển thông thường tới điểm bấm (dàn đội hình thông minh)
    let count = engine.selected_unit_ids.len();
    for (i, uid) in engine.selected_unit_ids.iter().enumerate() {
        if let Some(u) = engine
            .units
            .iter_mut()
            .find(|u| u.id == *uid && u.team == Team::Player)
        {
            let offset_x = ((i % 5) as f32 - (count.min(5) as f32 * 0.5)) * 22.0;
            let offset_y = ((i / 5) as f32) * 22.0;
            u.order = UnitOrder::MoveTo {
                x: wx + offset_x,
                y: wy + offset_y,
            };
        }
    }
    engine.outgoing_events.push(EngineToReactEvent::PlaySound {
        sound: "order".to_string(),
    });
}

fn try_place_building(engine: &mut GameEngine, b_type: BuildingType, wx: f32, wy: f32) {
    let (cost_min, cost_en, w, h, max_hp) = match b_type {
        BuildingType::CommandCenter => (400.0, 0.0, 90.0, 90.0, 2000.0),
        BuildingType::SupplyDepot => (100.0, 0.0, 60.0, 60.0, 800.0),
        BuildingType::PowerReactor => (120.0, 0.0, 60.0, 60.0, 700.0),
        BuildingType::Barracks => (150.0, 0.0, 80.0, 70.0, 1200.0),
        BuildingType::TurretTower => (125.0, 50.0, 50.0, 50.0, 900.0),
    };

    if engine.player_eco.minerals >= cost_min && engine.player_eco.energy >= cost_en {
        engine.player_eco.minerals -= cost_min;
        engine.player_eco.energy -= cost_en;
        let bid = engine.alloc_id();
        let mut b_res = match b_type {
            BuildingType::CommandCenter => Resistances {
                physical: 0.25,
                energy: 0.15,
                explosive: 0.20,
                arcane: 0.10,
            },
            BuildingType::TurretTower => Resistances {
                physical: 0.20,
                energy: 0.20,
                explosive: 0.30,
                arcane: 0.10,
            },
            BuildingType::Barracks => Resistances {
                physical: 0.20,
                energy: 0.10,
                explosive: 0.20,
                arcane: 0.0,
            },
            _ => Resistances {
                physical: 0.15,
                energy: 0.10,
                explosive: 0.15,
                arcane: 0.0,
            },
        };
        if engine.player_tech.titan_alloy_plating {
            b_res.physical = (b_res.physical + 0.20).min(0.80);
        }

        let mut hp_mult = 1.0;
        if engine.player_tech.fortified_blueprints {
            hp_mult += 0.3;
        }
        if engine.player_tech.titan_alloy_plating {
            hp_mult += 0.25;
        }

        engine.buildings.push(Building {
            id: bid,
            team: Team::Player,
            building_type: b_type,
            x: wx,
            y: wy,
            width: w,
            height: h,
            hp: 1.0,
            max_hp: max_hp * hp_mult,
            damage_type: match b_type {
                BuildingType::TurretTower => {
                    if engine.player_tech.dark_matter_rockets {
                        DamageType::Explosive
                    } else {
                        DamageType::Energy
                    }
                }
                _ => DamageType::Physical,
            },
            resistances: b_res,
            build_progress: 0.05,
            attack_cooldown: 0.0,
            current_cooldown: 0.0,
            rally_x: wx + 40.0,
            rally_y: wy + 40.0,
            training_queue: Vec::new(),
            training_timer: 0.0,
        });

        // Tự động phân công kỹ sư đang chọn đến xây
        for uid in &engine.selected_unit_ids {
            if let Some(u) = engine
                .units
                .iter_mut()
                .find(|u| u.id == *uid && u.unit_type == UnitType::Engineer)
            {
                u.order = UnitOrder::Build {
                    target_building_id: bid,
                };
            }
        }

        engine.outgoing_events.push(EngineToReactEvent::PlaySound {
            sound: "place_building".to_string(),
        });
    }
}

// Bấm nút UI trên HUD Bevy (Góc trên, Minimap, Bảng nút lệnh góc dưới phải)
fn check_bevy_hud_click(engine: &mut GameEngine, sx: f32, sy: f32) -> bool {
    let vw = engine.camera.viewport_w;
    let vh = engine.camera.viewport_h;

    // 1. Nút mở Tech Tree [T] ở thanh trên cùng (x ~ vw - 190, y: 10..42, w: 90)
    if (vw - 190.0..=vw - 100.0).contains(&sx) && (10.0..=42.0).contains(&sy) {
        if engine.settings.auto_pause_on_tech {
            engine.is_paused = true;
        }
        engine
            .outgoing_events
            .push(EngineToReactEvent::OpenTechTreeModal);
        return true;
    }

    // 2. Nút Cài đặt [ESC] ở thanh trên cùng (x ~ vw - 90, y: 10..42, w: 80)
    if (vw - 90.0..=vw - 10.0).contains(&sx) && (10.0..=42.0).contains(&sy) {
        engine
            .outgoing_events
            .push(EngineToReactEvent::OpenSettingsModal);
        return true;
    }

    // 3. Minimap góc dưới trái (x: 10..170, y: vh - 170..vh - 10)
    if (10.0..=170.0).contains(&sx) && (vh - 170.0..=vh - 10.0).contains(&sy) {
        let norm_x = (sx - 10.0) / 160.0;
        let norm_y = (sy - (vh - 170.0)) / 160.0;
        engine.camera.x = norm_x * MAP_WIDTH;
        engine.camera.y = norm_y * MAP_HEIGHT;
        return true;
    }

    // 4. Command Card góc dưới phải (x: vw - 225..vw - 10, y: vh - 150..vh - 10)
    if (vw - 225.0..=vw - 10.0).contains(&sx) && (vh - 150.0..=vh - 10.0).contains(&sy) {
        handle_command_card_click(engine, sx - (vw - 225.0), sy - (vh - 150.0));
        return true;
    }

    false
}

fn handle_command_card_click(engine: &mut GameEngine, rel_x: f32, rel_y: f32) {
    let col = ((rel_x - 5.0).max(0.0) / 67.0).min(2.0) as usize;
    let row = ((rel_y - 6.0).max(0.0) / 44.0).min(2.0) as usize;
    let btn_idx = row * 3 + col;

    // Nếu đang chọn Worker -> Nút xây nhà
    let has_worker = engine.selected_unit_ids.iter().any(|uid| {
        engine
            .units
            .iter()
            .any(|u| u.id == *uid && u.unit_type == UnitType::Engineer)
    });

    if has_worker {
        match btn_idx {
            0 => engine.build_mode = Some(BuildingType::SupplyDepot),
            1 => engine.build_mode = Some(BuildingType::PowerReactor),
            2 => engine.build_mode = Some(BuildingType::Barracks),
            3 => engine.build_mode = Some(BuildingType::TurretTower),
            4 => engine.build_mode = Some(BuildingType::CommandCenter),
            _ => {}
        }
        return;
    }

    // Nếu đang chọn Nhà -> Mua lính
    if let Some(bid) = engine.selected_building_id {
        let b = engine
            .buildings
            .iter()
            .find(|b| b.id == bid && b.team == Team::Player && b.build_progress >= 1.0)
            .cloned();
        if let Some(b) = b {
            match b.building_type {
                BuildingType::CommandCenter => {
                    if btn_idx == 0 {
                        try_queue_unit(engine, bid, UnitType::Engineer);
                    }
                }
                BuildingType::Barracks => match btn_idx {
                    0 => try_queue_unit(engine, bid, UnitType::ShockTrooper),
                    1 => try_queue_unit(engine, bid, UnitType::Rifleman),
                    2 if engine.player_tech.energy_shielding => {
                        try_queue_unit(engine, bid, UnitType::Vanguard);
                    }
                    3 if engine.player_tech.focused_laser => {
                        try_queue_unit(engine, bid, UnitType::LaserMarksman);
                    }
                    4 if engine.player_tech.dark_matter_rockets => {
                        try_queue_unit(engine, bid, UnitType::DarkRocketeer);
                    }
                    5 if engine.player_tech.celestial_flying_swords => {
                        try_queue_unit(engine, bid, UnitType::TaoistSwordmaster);
                    }
                    6 if engine.player_tech.void_cataclysm => {
                        try_queue_unit(engine, bid, UnitType::VoidWarlock);
                    }
                    7 if engine.player_tech.behemoth_mutation => {
                        try_queue_unit(engine, bid, UnitType::BehemothTitan);
                    }
                    8 if engine.player_tech.acid_banelings => {
                        try_queue_unit(engine, bid, UnitType::AcidBaneling);
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }
}

fn handle_command_card_right_click(engine: &mut GameEngine, rel_x: f32, rel_y: f32) {
    let col = ((rel_x - 5.0).max(0.0) / 67.0).min(2.0) as usize;
    let row = ((rel_y - 6.0).max(0.0) / 44.0).min(2.0) as usize;
    let btn_idx = row * 3 + col;

    if let Some(bid) = engine.selected_building_id {
        let b = engine
            .buildings
            .iter()
            .find(|b| b.id == bid && b.team == Team::Player && b.build_progress >= 1.0)
            .cloned();
        if let Some(b) = b {
            match b.building_type {
                BuildingType::CommandCenter => {
                    if btn_idx == 0 {
                        try_cancel_unit(engine, bid, UnitType::Engineer);
                    }
                }
                BuildingType::Barracks => match btn_idx {
                    0 => try_cancel_unit(engine, bid, UnitType::ShockTrooper),
                    1 => try_cancel_unit(engine, bid, UnitType::Rifleman),
                    2 if engine.player_tech.energy_shielding => {
                        try_cancel_unit(engine, bid, UnitType::Vanguard);
                    }
                    3 if engine.player_tech.focused_laser => {
                        try_cancel_unit(engine, bid, UnitType::LaserMarksman);
                    }
                    4 if engine.player_tech.dark_matter_rockets => {
                        try_cancel_unit(engine, bid, UnitType::DarkRocketeer);
                    }
                    5 if engine.player_tech.celestial_flying_swords => {
                        try_cancel_unit(engine, bid, UnitType::TaoistSwordmaster);
                    }
                    6 if engine.player_tech.void_cataclysm => {
                        try_cancel_unit(engine, bid, UnitType::VoidWarlock);
                    }
                    7 if engine.player_tech.behemoth_mutation => {
                        try_cancel_unit(engine, bid, UnitType::BehemothTitan);
                    }
                    8 if engine.player_tech.acid_banelings => {
                        try_cancel_unit(engine, bid, UnitType::AcidBaneling);
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }
}

fn try_queue_unit(engine: &mut GameEngine, bid: u32, u_type: UnitType) {
    if engine.player_eco.supply >= engine.player_eco.max_supply {
        return; // Hết dân số
    }

    let (mut min_cost, mut en_cost) = u_type.cost();
    if engine.player_tech.hive_proliferation {
        min_cost *= 0.65;
        en_cost *= 0.65;
    }
    if engine.player_eco.minerals >= min_cost && engine.player_eco.energy >= en_cost {
        if let Some(b) = engine.buildings.iter_mut().find(|b| b.id == bid) {
            if b.training_queue.len() < 5 {
                engine.player_eco.minerals -= min_cost;
                engine.player_eco.energy -= en_cost;
                b.training_queue.push(u_type);
                engine.outgoing_events.push(EngineToReactEvent::PlaySound {
                    sound: "train_unit".to_string(),
                });
            }
        }
    }
}

fn try_cancel_unit(engine: &mut GameEngine, bid: u32, u_type: UnitType) {
    let mut refunded = false;
    let mut spawn_pos = (0.0, 0.0);
    let (mut min_cost, mut en_cost) = u_type.cost();
    if engine.player_tech.hive_proliferation {
        min_cost *= 0.65;
        en_cost *= 0.65;
    }

    if let Some(b) = engine
        .buildings
        .iter_mut()
        .find(|b| b.id == bid && b.team == Team::Player)
    {
        // Tìm vị trí cuối cùng của đơn vị loại này trong hàng đợi để hủy
        if let Some(idx) = b.training_queue.iter().rposition(|&t| t == u_type) {
            b.training_queue.remove(idx);
            // Nếu đơn vị bị hủy là đơn vị đang chạy ở đầu hàng đợi
            if idx == 0 {
                b.training_timer = 0.0;
            }
            refunded = true;
            spawn_pos = (b.x, b.y);
        }
    }

    if refunded {
        engine.player_eco.minerals += min_cost;
        engine.player_eco.energy += en_cost;
        let ft_id = engine.alloc_id();
        let refund_str = if en_cost > 0.0 {
            format!("+{:.0}💎 +{:.0}⚡", min_cost, en_cost)
        } else {
            format!("+{:.0}💎", min_cost)
        };
        engine.floating_texts.push(FloatingText {
            id: ft_id,
            text: refund_str,
            x: spawn_pos.0,
            y: spawn_pos.1 - 35.0,
            color: "#22c55e".to_string(),
            lifetime: 0.0,
            max_lifetime: 1.2,
        });
        engine.outgoing_events.push(EngineToReactEvent::PlaySound {
            sound: "click".to_string(),
        });
    }
}
