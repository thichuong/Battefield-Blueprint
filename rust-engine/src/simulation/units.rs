use crate::components::*;
use crate::map::{MAP_HEIGHT, MAP_WIDTH};
use crate::GameEngine;

pub fn update_units(engine: &mut GameEngine, dt: f32) {
    let mut new_projectiles = Vec::new();
    let mut new_floating_texts = Vec::new();
    let mut new_magic_circles = Vec::new();
    let mut mineral_deliveries: Vec<(Team, f32, f32, f32)> = Vec::new();
    let factions = engine.team_factions();

    let has_exo: [bool; 8] =
        std::array::from_fn(|i| engine.team_tech(Team::from_index(i)).exo_boosters);
    let has_superheated: [bool; 8] =
        std::array::from_fn(|i| engine.team_tech(Team::from_index(i)).superheated_blades);
    let has_magnetic: [bool; 8] =
        std::array::from_fn(|i| engine.team_tech(Team::from_index(i)).magnetic_rounds);
    let has_bell: [bool; 8] =
        std::array::from_fn(|i| engine.team_tech(Team::from_index(i)).golden_bell_shield);
    let has_drills: [bool; 8] =
        std::array::from_fn(|i| engine.team_tech(Team::from_index(i)).optical_drills);
    let warlock_cfg_dps = engine
        .config
        .get_unit(UnitType::VoidWarlock)
        .map(|u| u.percent_max_hp_dps)
        .filter(|&p| p > 0.0)
        .unwrap_or(crate::config::VOID_WARLOCK_DEFAULT_MAX_HP_DPS);

    engine
        .spatial_grid
        .rebuild_with_factions(&engine.units, &engine.buildings, &factions);
    let spatial_grid = &engine.spatial_grid;

    // Snapshot danh sách mỏ quặng và CCs (kích thước nhỏ: quặng ~30-40, CC ~8-16)
    let mineral_nodes: Vec<(u32, f32, f32)> =
        engine.minerals.iter().map(|m| (m.id, m.x, m.y)).collect();
    let cc_targets: Vec<(Team, f32, f32)> = engine
        .buildings
        .iter()
        .filter(|b| b.building_type == BuildingType::CommandCenter)
        .map(|b| (b.team, b.x, b.y))
        .collect();

    let num_units = engine.units.len();
    for u_idx in 0..num_units {
        let (u_team, mut u_order, u_type, mut u_laser_target, u_range, u_x, u_y, mut u_laser_stacks, mut u_cooldown, mut u_last_attack_pos) = {
            let u = &engine.units[u_idx];
            (
                u.team,
                u.order,
                u.unit_type,
                u.laser_target_id,
                u.attack_range,
                u.x,
                u.y,
                u.laser_stacks,
                u.current_cooldown,
                u.last_attack_pos,
            )
        };
        let u_team_idx = u_team.index() % 8;
        let my_faction = factions[u_team_idx];

        // Hàm tra cứu mục tiêu O(1) qua spatial_grid
        let get_target_info = |target_id: u32, is_building: bool| -> Option<(f32, f32, f32)> {
            if is_building {
                if let Some(&b_idx) = spatial_grid.building_id_to_index.get(&target_id) {
                    if b_idx < engine.buildings.len() {
                        let b = &engine.buildings[b_idx];
                        if b.id == target_id && factions[b.team.index() % 8] != my_faction && b.hp > 0.0 {
                            return Some((b.x, b.y, b.width.max(b.height) * 0.5));
                        }
                    }
                }
            } else {
                if let Some(&t_idx) = spatial_grid.unit_id_to_index.get(&target_id) {
                    if t_idx < engine.units.len() {
                        let target_u = &engine.units[t_idx];
                        if target_u.id == target_id
                            && factions[target_u.team.index() % 8] != my_faction
                            && target_u.hp > 0.0
                        {
                            return Some((target_u.x, target_u.y, 10.0));
                        }
                    }
                }
            }
            None
        };

        if u_type == UnitType::LaserMarksman {
            // LaserMarksman: Quản lý khóa mục tiêu (Target Lock)
            let mut lock_valid = false;
            if let Some((lid, lis_b)) = u_laser_target {
                if let Some((tx, ty, tr)) = get_target_info(lid, lis_b) {
                    let dx = tx - u_x;
                    let dy = ty - u_y;
                    let dist = (dx * dx + dy * dy).sqrt();
                    let effective_dist = (dist - tr).max(0.0);
                    if effective_dist <= u_range {
                        lock_valid = true;
                    }
                }
            }

            if lock_valid {
                let (lid, lis_b) = u_laser_target.unwrap();
                match u_order {
                    UnitOrder::MoveTo { .. } => {
                        // Người chơi chủ động ra lệnh di chuyển (rút lui), tạm ngắt tia laser
                        u_last_attack_pos = None;
                    }
                    UnitOrder::AttackTarget {
                        target_id,
                        is_building,
                    } if (target_id, is_building) != (lid, lis_b) => {
                        // Người chơi chủ động đổi sang tấn công mục tiêu khác: làm nguội 1s mới bắn được
                        u_laser_target = Some((target_id, is_building));
                        u_laser_stacks = 0;
                        u_last_attack_pos = None;
                        u_cooldown = u_cooldown.max(1.0);
                        u_order = UnitOrder::AttackTarget {
                            target_id,
                            is_building,
                        };
                    }
                    _ => {
                        // Mục tiêu còn sống và trong tầm bắn: KHÔNG đổi mục tiêu!
                        // Khóa chặt trạng thái AttackTarget vào mục tiêu này để tối ưu sát thương
                        u_order = UnitOrder::AttackTarget {
                            target_id: lid,
                            is_building: lis_b,
                        };
                    }
                }
            } else {
                // Mục tiêu cũ bị tiêu diệt hoặc ra khỏi tầm bắn -> Hủy khóa mục tiêu, làm nguội 1s khi đổi mục tiêu
                if u_laser_target.is_some() {
                    u_laser_target = None;
                    u_laser_stacks = 0;
                    u_last_attack_pos = None;
                    u_cooldown = u_cooldown.max(1.0);
                }
            }
        }

        // Lấy thông tin mục tiêu AttackTarget hoặc Build nhanh O(1) qua spatial_grid
        let attack_target_info = if let UnitOrder::AttackTarget {
            target_id,
            is_building,
        } = u_order
        {
            get_target_info(target_id, is_building)
        } else {
            None
        };

        let build_target_info = if let UnitOrder::Build { target_building_id } = u_order {
            if let Some(&b_idx) = spatial_grid.building_id_to_index.get(&target_building_id) {
                if b_idx < engine.buildings.len() {
                    let b = &engine.buildings[b_idx];
                    if b.id == target_building_id {
                        Some((b.x, b.y, b.build_progress))
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
            None
        };

        let unit = &mut engine.units[u_idx];
        unit.order = u_order;
        unit.laser_target_id = u_laser_target;
        unit.laser_stacks = u_laser_stacks;
        unit.current_cooldown = u_cooldown;
        unit.last_attack_pos = u_last_attack_pos;

        if unit.current_cooldown > 0.0 {
            unit.current_cooldown -= dt;
        }

        unit.attack_anim_timer = (unit.attack_anim_timer - dt * 3.5).max(0.0);

        // Vanguard: mỗi 5s nhận lớp khiên 100
        if unit.unit_type == UnitType::Vanguard {
            unit.shield_timer += dt;
            if unit.shield_timer >= 5.0 {
                unit.shield_timer -= 5.0;
                unit.shield = 100.0;
                unit.max_shield = 100.0;
            }
        }

        // TaoistSwordmaster: mỗi 1s tích 1 phi kiếm (tối đa 7)
        if unit.unit_type == UnitType::TaoistSwordmaster && unit.flying_swords < 7 {
            unit.sword_regen_timer += dt;
            if unit.sword_regen_timer >= 1.0 {
                unit.sword_regen_timer -= 1.0;
                unit.flying_swords = (unit.flying_swords + 1).min(7);
            }
        }

        // Nếu không trong trạng thái AttackTarget, dọn dẹp trạng thái ngắm/laser
        if !matches!(unit.order, UnitOrder::AttackTarget { .. }) {
            unit.last_attack_pos = None;
            if unit.unit_type != UnitType::LaserMarksman {
                unit.laser_target_id = None;
                unit.laser_stacks = 0;
            }
        }

        let u_team_idx = unit.team.index() % 8;
        let my_faction = factions[u_team_idx];

        let mut cur_speed = unit.speed;
        if has_exo[u_team_idx] {
            cur_speed *= 1.2;
        }

        match unit.order {
            UnitOrder::Idle => {
                if unit.unit_type != UnitType::Engineer {
                    let scan_range = (unit.attack_range * 1.6).max(300.0);
                    if let Some((target_id, is_building, _)) = spatial_grid.find_nearest_hostile(
                        glam::Vec2::new(unit.x, unit.y),
                        scan_range,
                        my_faction,
                    ) {
                        unit.order = UnitOrder::AttackTarget {
                            target_id,
                            is_building,
                        };
                    }
                }
            }
            UnitOrder::AttackMove { x, y } => {
                // Tự động tìm mục tiêu lính hoặc nhà trên đường di chuyển qua Spatial Grid (Ưu tiên tháp pháo & lính)
                let scan_range = (unit.attack_range * 2.2).max(380.0);
                if let Some((target_id, is_building, _)) = spatial_grid.find_nearest_hostile(
                    glam::Vec2::new(unit.x, unit.y),
                    scan_range,
                    my_faction,
                ) {
                    unit.order = UnitOrder::AttackTarget {
                        target_id,
                        is_building,
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
                        let has_drill = has_drills[u_team_idx];
                        let mining_duration = if has_drill {
                            engine.config.economy.worker_mining_time_upgraded
                        } else {
                            engine.config.economy.worker_mining_time
                        };
                        t += dt;
                        if t >= mining_duration {
                            unit.carried_minerals = if has_drill {
                                engine.config.economy.worker_carry_capacity_upgraded
                            } else {
                                engine.config.economy.worker_carry_capacity
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
                        // Tìm Command Center gần nhất của phe mình để giao khoáng sản
                        let my_cc = cc_targets
                            .iter()
                            .filter(|(t, _, _)| factions[t.index() % 8] == my_faction)
                            .min_by(|a, b| {
                                let da = (a.1 - unit.x).powi(2) + (a.2 - unit.y).powi(2);
                                let db = (b.1 - unit.x).powi(2) + (b.2 - unit.y).powi(2);
                                da.partial_cmp(&db).unwrap()
                            });

                        if let Some((_, cx, cy)) = my_cc {
                            let dx = *cx - unit.x;
                            let dy = *cy - unit.y;
                            let dist = (dx * dx + dy * dy).sqrt();
                            if dist > 55.0 {
                                let move_dist = (cur_speed * dt).min(dist);
                                unit.x += (dx / dist) * move_dist;
                                unit.y += (dy / dist) * move_dist;
                                unit.walk_cycle += dt * 10.0;
                                unit.facing_right = dx >= 0.0;
                            } else {
                                // Giao hàng thành công!
                                let carried = unit.carried_minerals;
                                mineral_deliveries.push((
                                    unit.team,
                                    carried,
                                    unit.x,
                                    unit.y - 15.0,
                                ));
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
            UnitOrder::Build { .. } => {
                let is_still_building = if let Some((bx, by, build_prog)) = build_target_info {
                    if build_prog < 1.0 {
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
                        true
                    } else {
                        false // Công trình đã hoàn thành (build_prog >= 1.0)
                    }
                } else {
                    false // Công trình không tồn tại nữa
                };

                if !is_still_building {
                    // Công trình đã xây xong! Engineer lập tức chuyển sang khai thác mỏ quặng gần nhất
                    let nearest_mineral = mineral_nodes
                        .iter()
                        .min_by(|a, b| {
                            let da = (a.1 - unit.x).powi(2) + (a.2 - unit.y).powi(2);
                            let db = (b.1 - unit.x).powi(2) + (b.2 - unit.y).powi(2);
                            da.partial_cmp(&db).unwrap()
                        })
                        .map(|m| m.0);

                    if let Some(target_mineral_id) = nearest_mineral {
                        unit.order = UnitOrder::Harvest {
                            target_mineral_id,
                            phase: HarvestPhase::MovingToNode,
                        };
                    } else {
                        unit.order = UnitOrder::Idle;
                    }
                }
            }
            UnitOrder::AttackTarget {
                target_id,
                is_building,
            } => {
                let target_info = attack_target_info;

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
                        if unit.unit_type == UnitType::LaserMarksman {
                            unit.last_attack_pos = None;
                        }
                    } else {
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

                        if unit.unit_type == UnitType::LaserMarksman {
                            if unit.last_attack_pos.is_some() {
                                unit.last_attack_pos = Some((hit_x, hit_y));
                            }
                        } else {
                            unit.last_attack_pos = Some((hit_x, hit_y));
                        }

                        // TẤN CÔNG ĐÍCH XÁC
                        if unit.current_cooldown <= 0.0 {
                            if unit.unit_type == UnitType::VoidWarlock {
                                // VoidWarlock không tấn công thường, mỗi 2s triệu hồi 1 vòng ma thuật duy trì 5s
                                unit.current_cooldown = unit.attack_cooldown;
                                unit.attack_anim_timer = 1.0;
                                let dps_pct = if unit.percent_max_hp_dps > 0.0 {
                                    unit.percent_max_hp_dps
                                } else {
                                    warlock_cfg_dps
                                };
                                new_magic_circles.push(MagicCircle {
                                    id: 0,
                                    team: unit.team,
                                    x: tx,
                                    y: ty,
                                    radius: crate::config::VOID_WARLOCK_CIRCLE_RADIUS,
                                    lifetime: crate::config::VOID_WARLOCK_CIRCLE_LIFETIME,
                                    max_lifetime: crate::config::VOID_WARLOCK_CIRCLE_LIFETIME,
                                    tick_timer: 0.0,
                                    damage_pct_per_sec: dps_pct,
                                    damage_type: unit.damage_type,
                                });
                            } else if unit.unit_type == UnitType::TaoistSwordmaster
                                && unit.flying_swords == 0
                            {
                                // Hết phi kiếm -> chờ tích lũy kiếm
                            } else {
                                unit.current_cooldown = unit.attack_cooldown;
                                unit.attack_anim_timer = 1.0;
                                let mut dmg = unit.attack_damage;
                                let has_mag = has_magnetic[u_team_idx];
                                let has_super = has_superheated[u_team_idx];
                                if unit.unit_type == UnitType::Rifleman && has_mag {
                                    dmg *= 1.25;
                                }
                                if unit.unit_type == UnitType::ShockTrooper && has_super {
                                    dmg *= 1.35;
                                }

                                if unit.unit_type == UnitType::LaserMarksman {
                                    // LaserMarksman: Tăng sát thương theo thời gian trên cùng mục tiêu
                                    // Sát thương khởi đầu là 1, mỗi lần tăng 2
                                    let current_target = (target_id, is_building);
                                    let is_same_target = unit.laser_target_id == Some(current_target);

                                    if is_same_target && unit.last_attack_pos.is_some() {
                                        unit.laser_stacks += 1;
                                    } else {
                                        unit.laser_target_id = Some(current_target);
                                        unit.laser_stacks = 0;
                                    }
                                    dmg = 1.0 + (unit.laser_stacks as f32) * 2.0;
                                    unit.last_attack_pos = Some((hit_x, hit_y));
                                } else if unit.unit_type == UnitType::TaoistSwordmaster {
                                    // Tiêu hao 1 phi kiếm
                                    unit.flying_swords = unit.flying_swords.saturating_sub(1);
                                }

                                if unit.unit_type == UnitType::AcidBaneling {
                                    // Bọ nổ cảm tử: nổ tung ngay lập tức, x2 sát thương lên công trình
                                    unit.hp = 0.0;
                                    new_projectiles.push(Projectile {
                                        id: 0,
                                        team: unit.team,
                                        x: hit_x,
                                        y: hit_y,
                                        prev_x: hit_x,
                                        prev_y: hit_y,
                                        target_x: hit_x,
                                        target_y: hit_y,
                                        speed: 1500.0,
                                        damage: dmg,
                                        damage_type: DamageType::Explosive,
                                        aoe_radius: unit.aoe_radius,
                                        is_laser: false,
                                        lifetime: 0.0,
                                        is_melee: true,
                                        building_damage_mult: 2.0,
                                        visual: ProjectileVisual::Bullet,
                                        angle: 0.0,
                                    });
                                } else if unit.unit_type == UnitType::ShockTrooper
                                    || unit.unit_type == UnitType::Vanguard
                                    || unit.unit_type == UnitType::Engineer
                                    || unit.unit_type == UnitType::BehemothTitan
                                {
                                    // Cận chiến: gây sát thương tức thì trực tiếp tại mục tiêu, không có tia đạn
                                    new_projectiles.push(Projectile {
                                        id: 0,
                                        team: unit.team,
                                        x: hit_x,
                                        y: hit_y,
                                        prev_x: hit_x,
                                        prev_y: hit_y,
                                        target_x: hit_x,
                                        target_y: hit_y,
                                        speed: 2000.0,
                                        damage: dmg,
                                        damage_type: unit.damage_type,
                                        aoe_radius: unit.aoe_radius,
                                        is_laser: false,
                                        lifetime: 0.0,
                                        is_melee: true,
                                        building_damage_mult: 1.0,
                                        visual: ProjectileVisual::Bullet,
                                        angle: 0.0,
                                    });
                                } else if unit.unit_type == UnitType::LaserMarksman {
                                    // LaserMarksman: Tia laser công kích liên tục trực tiếp vào mục tiêu.
                                    // Tia laser hiển thị đã được render riêng từ họng súng (gun tip) theo số cộng dồn.
                                    // Projectile tác động tức thì tại vị trí mục tiêu, đánh dấu is_melee: true để không vẽ tia trùng lặp.
                                    new_projectiles.push(Projectile {
                                        id: 0,
                                        team: unit.team,
                                        x: hit_x,
                                        y: hit_y,
                                        prev_x: hit_x,
                                        prev_y: hit_y,
                                        target_x: hit_x,
                                        target_y: hit_y,
                                        speed: 2400.0,
                                        damage: dmg,
                                        damage_type: unit.damage_type,
                                        aoe_radius: unit.aoe_radius,
                                        is_laser: false,
                                        lifetime: 0.0,
                                        is_melee: true,
                                        building_damage_mult: 1.0,
                                        visual: ProjectileVisual::Laser,
                                        angle: 0.0,
                                    });
                                } else {
                                    // Tầm xa (Rifleman, DarkRocketeer, TaoistSwordmaster)
                                    let speed = if unit.unit_type == UnitType::TaoistSwordmaster {
                                        1100.0
                                    } else if unit.unit_type == UnitType::DarkRocketeer {
                                        650.0
                                    } else {
                                        750.0
                                    };
                                    let spawn_y = unit.y - 12.0;
                                    let angle = (hit_y - spawn_y).atan2(hit_x - unit.x);
                                    let visual = match unit.unit_type {
                                        UnitType::Rifleman => ProjectileVisual::Bullet,
                                        UnitType::DarkRocketeer => ProjectileVisual::Rocket,
                                        UnitType::TaoistSwordmaster => {
                                            ProjectileVisual::FlyingSword
                                        }
                                        _ => ProjectileVisual::Bullet,
                                    };
                                    new_projectiles.push(Projectile {
                                        id: 0,
                                        team: unit.team,
                                        x: unit.x,
                                        y: spawn_y,
                                        prev_x: unit.x,
                                        prev_y: spawn_y,
                                        target_x: hit_x,
                                        target_y: hit_y,
                                        speed,
                                        damage: dmg,
                                        damage_type: unit.damage_type,
                                        aoe_radius: unit.aoe_radius,
                                        is_laser: false,
                                        lifetime: 1.5,
                                        is_melee: false,
                                        building_damage_mult: 1.0,
                                        visual,
                                        angle,
                                    });
                                }
                            }
                        }
                    }
                } else {
                    // Mục tiêu đã bị hạ gục! Nếu là đơn vị chiến đấu, tiếp tục AttackMove về căn cứ thù địch gần nhất
                    if unit.unit_type != UnitType::Engineer {
                        let nearest_enemy_cc = cc_targets
                            .iter()
                            .filter(|(t, _, _)| factions[t.index() % 8] != my_faction)
                            .min_by(|a, b| {
                                let da = (a.1 - unit.x).powi(2) + (a.2 - unit.y).powi(2);
                                let db = (b.1 - unit.x).powi(2) + (b.2 - unit.y).powi(2);
                                da.partial_cmp(&db).unwrap()
                            });

                        let (tx, ty) = if let Some((_, ex, ey)) = nearest_enemy_cc {
                            (*ex + unit.formation_offset.0, *ey + unit.formation_offset.1)
                        } else {
                            (
                                MAP_WIDTH * 0.5 + unit.formation_offset.0,
                                MAP_HEIGHT * 0.5 + unit.formation_offset.1,
                            )
                        };

                        unit.order = UnitOrder::AttackMove {
                            x: tx.clamp(60.0, MAP_WIDTH - 60.0),
                            y: ty.clamp(60.0, MAP_HEIGHT - 60.0),
                        };
                    } else {
                        unit.order = UnitOrder::Idle;
                    }
                }
            }
        }
    }

    // Giao khoáng sản vào kinh tế từng team
    for (team, amount, fx, fy) in mineral_deliveries {
        let t_idx = team.index() % 8;
        let bot_mult = engine.bot_resource_multiplier(team);
        let actual_amount = amount * bot_mult;
        engine.team_minerals_mined[t_idx] += actual_amount;
        let eco = match team {
            Team::Player => {
                engine.minerals_mined_accumulator += actual_amount;
                if !engine.is_headless {
                    new_floating_texts.push(FloatingText {
                        id: 0,
                        text: format!("+{} 💎", actual_amount as u32),
                        x: fx,
                        y: fy,
                        color: "#00e5ff".to_string(),
                        lifetime: 1.0,
                        max_lifetime: 1.0,
                    });
                }
                &mut engine.player_eco
            }
            Team::Enemy => &mut engine.enemy_eco,
            _ => &mut engine.extra_ecos[team.index() - 2],
        };
        eco.minerals += actual_amount;
    }

    // Xóa lính chết do tự nổ
    engine.units.retain(|u| u.hp > 0.0);

    // Hồi máu linh quang từ Kim cang hộ thể (3 HP/s)
    for u in engine.units.iter_mut() {
        let t_idx = u.team.index() % 8;
        if has_bell[t_idx] && u.hp > 0.0 && u.hp < u.max_hp {
            u.hp = (u.hp + 3.0 * dt).min(u.max_hp);
        }
    }

    for mut p in new_projectiles {
        p.id = engine.alloc_id();
        engine.projectiles.push(p);
    }

    for mut c in new_magic_circles {
        c.id = engine.alloc_id();
        engine.magic_circles.push(c);
    }

    for mut ft in new_floating_texts {
        ft.id = engine.alloc_id();
        engine.floating_texts.push(ft);
    }

    // Sinh hiệu ứng hạt vụn tan biến khi lính tử trận
    if !engine.is_headless {
        let dead_units: Vec<(f32, f32, Team, UnitType)> = engine
            .units
            .iter()
            .filter(|u| u.hp <= 0.0)
            .map(|u| (u.x, u.y, u.team, u.unit_type))
            .collect();
        for (x, y, team, u_type) in dead_units {
            let color = match team {
                Team::Player => "#00e5ff",
                _ => "#f43f5e",
            };
            if u_type == UnitType::AcidBaneling {
                engine.spawn_particles(x, y, 18, "#84cc16", 180.0, 3.2, 0.45);
            } else {
                engine.spawn_particles(x, y, 8, color, 85.0, 2.0, 0.35);
            }
        }
    }

    // Xóa quặng đã khai thác hết
    engine.minerals.retain(|m| m.amount > 0.0);

    // Xóa lính tử trận
    let pre_selected = engine.selected_unit_ids.len();
    engine.units.retain(|u| u.hp > 0.0);
    if pre_selected > 0 {
        engine.selected_unit_ids.retain(|id| engine.units.iter().any(|u| u.id == *id));
        if engine.selected_unit_ids.len() != pre_selected {
            engine.selection_dirty = true;
            engine.command_card_dirty = true;
        }
    }
}
