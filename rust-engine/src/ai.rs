use crate::components::*;
use crate::GameEngine;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AiState {
    pub attack_wave: u32,
    pub is_attacking: bool,
    pub wave_target_size: usize,
    pub rally_point: (f32, f32),
    pub expansion_attempted: bool,
    pub last_attack_target: Option<(f32, f32)>,
}

impl Default for AiState {
    fn default() -> Self {
        Self {
            attack_wave: 0,
            is_attacking: false,
            wave_target_size: 10,
            rally_point: (1950.0, 580.0),
            expansion_attempted: false,
            last_attack_target: None,
        }
    }
}

pub fn update_ai(engine: &mut GameEngine) {
    let enemy_workers: Vec<u32> = engine
        .units
        .iter()
        .filter(|u| u.team == Team::Enemy && u.unit_type == UnitType::Engineer)
        .map(|u| u.id)
        .collect();

    let enemy_combat_units: Vec<u32> = engine
        .units
        .iter()
        .filter(|u| u.team == Team::Enemy && u.unit_type != UnitType::Engineer)
        .map(|u| u.id)
        .collect();

    let enemy_ccs: Vec<(u32, f32, f32, f32)> = engine
        .buildings
        .iter()
        .filter(|b| b.team == Team::Enemy && b.building_type == BuildingType::CommandCenter)
        .map(|b| (b.id, b.x, b.y, b.build_progress))
        .collect();

    let completed_cc_count = enemy_ccs
        .iter()
        .filter(|(_, _, _, prog)| *prog >= 1.0)
        .count();
    let total_cc_count = enemy_ccs.len();

    // Điểm neo căn cứ chính của AI (hoặc CC đầu tiên)
    let main_base_pos = enemy_ccs
        .first()
        .map(|(_, x, y, _)| (*x, *y))
        .unwrap_or((2150.0, 350.0));

    // =========================================================================
    // 1. Quản lý Thợ mỏ AI: Tự động phân bổ đều vào các mỏ còn khoáng sản
    // =========================================================================
    let available_minerals: Vec<(u32, f32, f32)> = engine
        .minerals
        .iter()
        .filter(|m| m.amount > 0.0)
        .map(|m| (m.id, m.x, m.y))
        .collect();

    if !available_minerals.is_empty() {
        for wid in &enemy_workers {
            if let Some(w) = engine.units.iter_mut().find(|u| u.id == *wid) {
                if w.order == UnitOrder::Idle {
                    // Ưu tiên mỏ khoáng sản gần thợ mỏ nhất
                    let nearest_mineral = available_minerals
                        .iter()
                        .min_by(|a, b| {
                            let da = (a.1 - w.x).powi(2) + (a.2 - w.y).powi(2);
                            let db = (b.1 - w.x).powi(2) + (b.2 - w.y).powi(2);
                            da.partial_cmp(&db).unwrap()
                        })
                        .map(|m| m.0);

                    if let Some(target_mineral_id) = nearest_mineral {
                        w.order = UnitOrder::Harvest {
                            target_mineral_id,
                            phase: HarvestPhase::MovingToNode,
                        };
                    }
                }
            }
        }
    }

    // =========================================================================
    // 2. AI Xây dựng công trình: SupplyDepot, PowerReactor, EnergyConverter,
    //    Barracks, TurretTower, và mở rộng CommandCenter
    // =========================================================================
    let enemy_supply = engine.enemy_eco.supply;
    let enemy_max_supply = engine.enemy_eco.max_supply;
    let enemy_minerals = engine.enemy_eco.minerals;
    let enemy_energy = engine.enemy_eco.energy;

    let depot_count = engine
        .buildings
        .iter()
        .filter(|b| b.team == Team::Enemy && b.building_type == BuildingType::SupplyDepot)
        .count();
    let depot_under_construction = engine
        .buildings
        .iter()
        .filter(|b| {
            b.team == Team::Enemy
                && b.building_type == BuildingType::SupplyDepot
                && b.build_progress < 1.0
        })
        .count();

    let reactor_count = engine
        .buildings
        .iter()
        .filter(|b| b.team == Team::Enemy && b.building_type == BuildingType::PowerReactor)
        .count();
    let completed_reactors = engine
        .buildings
        .iter()
        .filter(|b| {
            b.team == Team::Enemy
                && b.building_type == BuildingType::PowerReactor
                && b.build_progress >= 1.0
        })
        .count();

    let converter_count = engine
        .buildings
        .iter()
        .filter(|b| b.team == Team::Enemy && b.building_type == BuildingType::EnergyConverter)
        .count();

    let barracks_count = engine
        .buildings
        .iter()
        .filter(|b| b.team == Team::Enemy && b.building_type == BuildingType::Barracks)
        .count();

    let turret_count = engine
        .buildings
        .iter()
        .filter(|b| b.team == Team::Enemy && b.building_type == BuildingType::TurretTower)
        .count();

    // 2.1 Xây Supply Depot (Mở rộng quy mô lớn, lên tới 200 supply)
    let (depot_cost_m, _) = engine.config.get_building_cost(BuildingType::SupplyDepot);
    let need_depot = (enemy_supply + 4 >= enemy_max_supply
        || (enemy_minerals >= 300.0 && enemy_supply + 8 >= enemy_max_supply))
        && enemy_max_supply < 200
        && depot_under_construction < 2;

    if need_depot && enemy_minerals >= depot_cost_m {
        if let Some((bx, by)) = find_ai_build_spot(
            engine,
            BuildingType::SupplyDepot,
            main_base_pos.0,
            main_base_pos.1,
            450.0,
            None,
        ) {
            try_ai_build(engine, BuildingType::SupplyDepot, bx, by);
        }
    }

    // 2.2 Xây Tháp pháo TurretTower (Phòng thủ lối vào căn cứ chính và căn cứ phụ)
    // Hướng Tây Nam từ căn cứ chính tới căn cứ người chơi (góc ~2.4 radian)
    let (turret_cost_m, turret_cost_e) = engine.config.get_building_cost(BuildingType::TurretTower);
    let target_turrets = if completed_cc_count > 1 { 4 } else { 2 };
    if turret_count < target_turrets
        && enemy_minerals >= turret_cost_m
        && enemy_energy >= turret_cost_e
    {
        let anchor = if turret_count >= 2 && enemy_ccs.len() > 1 {
            // Đặt tháp bảo vệ căn cứ mở rộng
            (enemy_ccs[1].1, enemy_ccs[1].2)
        } else {
            main_base_pos
        };
        if let Some((tx, ty)) = find_ai_build_spot(
            engine,
            BuildingType::TurretTower,
            anchor.0,
            anchor.1,
            350.0,
            Some(2.4),
        ) {
            try_ai_build(engine, BuildingType::TurretTower, tx, ty);
        }
    }

    // 2.3 Xây Power Reactor (Tăng số lượng lớn: 4 -> 8 -> 12-16)
    let (reactor_cost_m, _) = engine.config.get_building_cost(BuildingType::PowerReactor);
    let target_reactors = if completed_cc_count > 1 || engine.game_time > 150.0 {
        16
    } else if engine.game_time > 60.0 || engine.enemy_eco.minerals >= 300.0 {
        8
    } else {
        4
    };

    let reactor_under_construction = engine
        .buildings
        .iter()
        .filter(|b| {
            b.team == Team::Enemy
                && b.building_type == BuildingType::PowerReactor
                && b.build_progress < 1.0
        })
        .count();

    if reactor_count < target_reactors
        && reactor_under_construction < 2
        && engine.enemy_eco.minerals >= reactor_cost_m
    {
        let anchor = if reactor_count >= 8 && enemy_ccs.len() > 1 {
            (enemy_ccs[1].1, enemy_ccs[1].2)
        } else {
            (main_base_pos.0 + 80.0, main_base_pos.1 + 80.0)
        };
        if let Some((rx, ry)) = find_ai_build_spot(
            engine,
            BuildingType::PowerReactor,
            anchor.0,
            anchor.1,
            550.0,
            Some(0.8),
        ) {
            try_ai_build(engine, BuildingType::PowerReactor, rx, ry);
        }
    }

    // 2.4 Xây Energy Converter (Theo công thức: 4 PowerReactor thì có 1 EnergyConverter)
    let (conv_cost_m, conv_cost_e) = engine
        .config
        .get_building_cost(BuildingType::EnergyConverter);
    let target_converters = completed_reactors / 4;
    let converter_under_construction = engine
        .buildings
        .iter()
        .filter(|b| {
            b.team == Team::Enemy
                && b.building_type == BuildingType::EnergyConverter
                && b.build_progress < 1.0
        })
        .count();

    if converter_count < target_converters
        && converter_under_construction < 1
        && engine.enemy_eco.minerals >= conv_cost_m
        && engine.enemy_eco.energy >= conv_cost_e
    {
        let anchor = if converter_count >= 2 && enemy_ccs.len() > 1 {
            (enemy_ccs[1].1, enemy_ccs[1].2)
        } else {
            (main_base_pos.0, main_base_pos.1 + 100.0)
        };
        if let Some((cx, cy)) = find_ai_build_spot(
            engine,
            BuildingType::EnergyConverter,
            anchor.0,
            anchor.1,
            450.0,
            None,
        ) {
            try_ai_build(engine, BuildingType::EnergyConverter, cx, cy);
        }
    }

    // 2.5 Xây nhiều Barracks (2 -> 4 -> 6-8 trại lính sản xuất ồ ạt)
    let (barracks_cost_m, _) = engine.config.get_building_cost(BuildingType::Barracks);
    let target_barracks = if completed_cc_count > 1 || engine.enemy_eco.minerals >= 400.0 {
        6
    } else if engine.game_time > 60.0 || depot_count >= 2 {
        4
    } else {
        2
    };

    if barracks_count < target_barracks && engine.enemy_eco.minerals >= barracks_cost_m {
        if let Some((bx, by)) = find_ai_build_spot(
            engine,
            BuildingType::Barracks,
            main_base_pos.0 - 80.0,
            main_base_pos.1 + 80.0,
            420.0,
            Some(2.2),
        ) {
            try_ai_build(engine, BuildingType::Barracks, bx, by);
        }
    }

    // 2.6 Xây thêm CommandCenter (Mở rộng căn cứ tự nhiên tại mỏ quặng x: 1900, y: 1400)
    let (cc_cost_m, _) = engine.config.get_building_cost(BuildingType::CommandCenter);
    if total_cc_count < 2
        && barracks_count >= 1
        && depot_count >= 1
        && enemy_workers.len() >= 6
        && engine.enemy_eco.minerals >= cc_cost_m
    {
        // Mỏ quặng tự nhiên cho AI mở rộng: Node 23 tại (1900.0, 1400.0)
        let exp_anchor_x = 1980.0;
        let exp_anchor_y = 1320.0;
        if let Some((cx, cy)) = find_ai_build_spot(
            engine,
            BuildingType::CommandCenter,
            exp_anchor_x,
            exp_anchor_y,
            250.0,
            None,
        ) {
            if try_ai_build(engine, BuildingType::CommandCenter, cx, cy) {
                engine.ai_state.expansion_attempted = true;
            }
        }
    }

    // =========================================================================
    // 3. AI Huấn luyện đơn vị: Thợ mỏ theo từng CC & Lính tại tất cả Barracks
    // =========================================================================
    // 3.1 Huấn luyện Thợ mỏ (8-10 thợ/CC, tối đa 18-20 thợ mỏ)
    let (worker_min, _) = engine.config.get_unit_cost(UnitType::Engineer);
    let target_workers = (completed_cc_count.max(1) * 8).min(18);
    if enemy_workers.len() < target_workers
        && engine.enemy_eco.minerals >= worker_min
        && engine.enemy_eco.supply < engine.enemy_eco.max_supply
    {
        // Chọn CC có hàng đợi ngắn nhất
        if let Some(cc) = engine
            .buildings
            .iter_mut()
            .filter(|b| {
                b.team == Team::Enemy
                    && b.building_type == BuildingType::CommandCenter
                    && b.build_progress >= 1.0
            })
            .min_by_key(|b| b.training_queue.len())
        {
            if cc.training_queue.len() < 2 {
                engine.enemy_eco.minerals -= worker_min;
                cc.training_queue.push(UnitType::Engineer);
            }
        }
    }

    // 3.2 Huấn luyện lính chiến đấu tại TẤT CẢ các Barracks song song
    for b in engine.buildings.iter_mut() {
        if b.team == Team::Enemy
            && b.building_type == BuildingType::Barracks
            && b.build_progress >= 1.0
            && b.training_queue.len() < 2
            && engine.enemy_eco.supply < engine.enemy_eco.max_supply
        {
            // Binh chủng cấp cao đa dạng khi kinh tế dư dả
            if engine.enemy_eco.minerals >= 180.0
                && engine.enemy_eco.energy >= 80.0
                && engine.game_time > 40.0
            {
                let choice = (engine.units.len() + b.id as usize) % 6;
                let evo_unit = match choice {
                    0 => UnitType::DarkRocketeer,
                    1 => UnitType::TaoistSwordmaster,
                    2 => UnitType::VoidWarlock,
                    3 => UnitType::BehemothTitan,
                    4 => UnitType::LaserMarksman,
                    _ => UnitType::AcidBaneling,
                };
                let (mc, ec) = engine.config.get_unit_cost(evo_unit);
                if engine.enemy_eco.minerals >= mc && engine.enemy_eco.energy >= ec {
                    engine.enemy_eco.minerals -= mc;
                    engine.enemy_eco.energy -= ec;
                    b.training_queue.push(evo_unit);
                    continue;
                }
            }

            // Lính chiến đấu nòng cốt (Rifleman / ShockTrooper)
            let (rifle_m, rifle_e) = engine.config.get_unit_cost(UnitType::Rifleman);
            let (striker_m, striker_e) = engine.config.get_unit_cost(UnitType::ShockTrooper);

            if engine.enemy_eco.minerals >= rifle_m && engine.enemy_eco.energy >= rifle_e {
                engine.enemy_eco.minerals -= rifle_m;
                engine.enemy_eco.energy -= rifle_e;
                b.training_queue.push(UnitType::Rifleman);
            } else if engine.enemy_eco.minerals >= striker_m && engine.enemy_eco.energy >= striker_e
            {
                engine.enemy_eco.minerals -= striker_m;
                engine.enemy_eco.energy -= striker_e;
                b.training_queue.push(UnitType::ShockTrooper);
            }
        }
    }

    // =========================================================================
    // 4. Quân sự: Tập kết quân (Staging) & Tấn công theo đợt lớn (Deathball Wave)
    // =========================================================================
    // 4.1 Cập nhật điểm tập kết (Phía trước căn cứ chính, được tháp pháo yểm trợ)
    let rally_point = (main_base_pos.0 - 180.0, main_base_pos.1 + 180.0);
    engine.ai_state.rally_point = rally_point;

    // 4.2 Phản xạ Phòng vệ Căn cứ: Kiểm tra xem căn cứ AI có đang bị quân Player tấn công không
    let is_base_under_threat = engine.units.iter().any(|u| {
        u.team == Team::Player
            && enemy_ccs.iter().any(|(_, cx, cy, _)| {
                let dist = ((u.x - cx).powi(2) + (u.y - cy).powi(2)).sqrt();
                dist <= 650.0
            })
    });

    if is_base_under_threat {
        // Tìm vị trí quân địch gần căn cứ nhất để kéo toàn bộ lính ra nghênh chiến
        let threat_pos = engine
            .units
            .iter()
            .filter(|u| u.team == Team::Player)
            .min_by(|a, b| {
                let da = ((a.x - main_base_pos.0).powi(2) + (a.y - main_base_pos.1).powi(2)).sqrt();
                let db = ((b.x - main_base_pos.0).powi(2) + (b.y - main_base_pos.1).powi(2)).sqrt();
                da.partial_cmp(&db).unwrap()
            })
            .map(|u| (u.x, u.y));

        if let Some((tx, ty)) = threat_pos {
            for uid in &enemy_combat_units {
                if let Some(u) = engine.units.iter_mut().find(|u| u.id == *uid) {
                    if matches!(u.order, UnitOrder::Idle | UnitOrder::MoveTo { .. }) {
                        u.order = UnitOrder::AttackMove { x: tx, y: ty };
                    }
                }
            }
            return;
        }
    }

    // 4.3 Kiểm tra chuyển đổi trạng thái Đợt tấn công (Attack Wave state machine)
    let current_combat_count = enemy_combat_units.len();
    let wave_target = engine.ai_state.wave_target_size;

    if !engine.ai_state.is_attacking {
        // Giai đoạn tích lũy & tập kết quân (Staging)
        if current_combat_count >= wave_target {
            // Đã tích đủ quy mô quân đội -> Phát lệnh TỔNG TIẾN CÔNG!
            engine.ai_state.is_attacking = true;
        } else {
            // Chưa đủ quân số -> Di chuyển về điểm tập kết tạo đội hình phòng ngự
            for (i, uid) in enemy_combat_units.iter().enumerate() {
                if let Some(u) = engine.units.iter_mut().find(|u| u.id == *uid) {
                    if u.order == UnitOrder::Idle {
                        let offset_x = ((i % 5) as f32 - 2.0) * 26.0;
                        let offset_y = ((i / 5) as f32) * 26.0;
                        u.order = UnitOrder::MoveTo {
                            x: rally_point.0 + offset_x,
                            y: rally_point.1 + offset_y,
                        };
                    }
                }
            }
        }
    }

    if engine.ai_state.is_attacking {
        // Kiểm tra xem đợt tấn công đã tan rã (bị quét sạch hoặc hao hụt nặng) chưa
        if current_combat_count <= 3 || current_combat_count < (wave_target / 4).max(3) {
            // Đợt tấn công thất bại hoặc bị chặn lại -> Rút quân về củng cố, nâng cấp quy mô đợt sau
            engine.ai_state.is_attacking = false;
            engine.ai_state.attack_wave += 1;
            // Tăng quy mô đợt sau (Wave 1: 10, Wave 2: 18, Wave 3: 26, Wave 4+: 36 -> 48)
            engine.ai_state.wave_target_size = match engine.ai_state.attack_wave {
                1 => 18,
                2 => 26,
                3 => 36,
                _ => 45,
            };

            // Lính sống sót rút về điểm tập kết
            for uid in &enemy_combat_units {
                if let Some(u) = engine.units.iter_mut().find(|u| u.id == *uid) {
                    u.order = UnitOrder::MoveTo {
                        x: rally_point.0,
                        y: rally_point.1,
                    };
                }
            }
        } else {
            // Đang trong đợt tổng tấn công -> Quét sạch căn cứ người chơi
            let target_pos = engine
                .buildings
                .iter()
                .find(|b| b.team == Team::Player && b.building_type == BuildingType::CommandCenter)
                .map(|b| (b.x, b.y))
                .or_else(|| {
                    engine
                        .buildings
                        .iter()
                        .find(|b| b.team == Team::Player)
                        .map(|b| (b.x, b.y))
                })
                .or_else(|| {
                    engine
                        .units
                        .iter()
                        .find(|u| u.team == Team::Player)
                        .map(|u| (u.x, u.y))
                })
                .unwrap_or((250.0, 2050.0));

            engine.ai_state.last_attack_target = Some(target_pos);

            for (i, uid) in enemy_combat_units.iter().enumerate() {
                if let Some(u) = engine.units.iter_mut().find(|u| u.id == *uid) {
                    if matches!(u.order, UnitOrder::Idle | UnitOrder::MoveTo { .. }) {
                        let offset_x = ((i % 6) as f32 - 2.5) * 30.0;
                        let offset_y = ((i / 6) as f32) * 30.0;
                        u.order = UnitOrder::AttackMove {
                            x: target_pos.0 + offset_x,
                            y: target_pos.1 + offset_y,
                        };
                    }
                }
            }
        }
    }
}

pub fn find_ai_build_spot(
    engine: &GameEngine,
    b_type: BuildingType,
    anchor_x: f32,
    anchor_y: f32,
    max_radius: f32,
    bias_angle: Option<f32>,
) -> Option<(f32, f32)> {
    let (bw, bh) = engine
        .config
        .get_building(b_type)
        .map(|c| (c.width, c.height))
        .unwrap_or((70.0, 70.0));

    let radius_step = 65.0;
    let mut current_radius = 80.0;

    while current_radius <= max_radius {
        let num_angles = ((2.0 * std::f32::consts::PI * current_radius) / 70.0).max(8.0) as usize;
        for i in 0..num_angles {
            let angle = if let Some(base_ang) = bias_angle {
                let offset = if i % 2 == 0 {
                    (i / 2) as f32
                } else {
                    -(i.div_ceil(2) as f32)
                };
                base_ang + offset * (std::f32::consts::PI * 2.0 / num_angles as f32)
            } else {
                (i as f32) * (std::f32::consts::PI * 2.0 / num_angles as f32)
            };

            let cand_x = anchor_x + current_radius * angle.cos();
            let cand_y = anchor_y + current_radius * angle.sin();

            // Kiểm tra giới hạn map (cách mép an toàn)
            if !(150.0..=2250.0).contains(&cand_x) || !(150.0..=2250.0).contains(&cand_y) {
                continue;
            }

            // Kiểm tra va chạm với các công trình khác
            let collides_building = engine.buildings.iter().any(|b| {
                let clearance_x = (bw + b.width) * 0.5 + 20.0;
                let clearance_y = (bh + b.height) * 0.5 + 20.0;
                (cand_x - b.x).abs() < clearance_x && (cand_y - b.y).abs() < clearance_y
            });
            if collides_building {
                continue;
            }

            // Kiểm tra khoảng cách an toàn với mỏ quặng để không cản đường thợ mỏ
            let collides_minerals = engine.minerals.iter().any(|m| {
                let dist = ((cand_x - m.x).powi(2) + (cand_y - m.y).powi(2)).sqrt();
                dist < 80.0
            });
            if collides_minerals {
                continue;
            }

            // Kiểm tra va chạm với chướng ngại vật
            let collides_obstacles = engine.obstacles.iter().any(|obs| {
                let obs_cx = obs.x + obs.width * 0.5;
                let obs_cy = obs.y + obs.height * 0.5;
                let clearance_x = (bw + obs.width) * 0.5 + 15.0;
                let clearance_y = (bh + obs.height) * 0.5 + 15.0;
                (cand_x - obs_cx).abs() < clearance_x && (cand_y - obs_cy).abs() < clearance_y
            });
            if collides_obstacles {
                continue;
            }

            return Some((cand_x, cand_y));
        }
        current_radius += radius_step;
    }
    None
}

pub fn try_ai_build(engine: &mut GameEngine, b_type: BuildingType, wx: f32, wy: f32) -> bool {
    let (min_cost, en_cost) = engine.config.get_building_cost(b_type);
    if engine.enemy_eco.minerals >= min_cost && engine.enemy_eco.energy >= en_cost {
        engine.enemy_eco.minerals -= min_cost;
        engine.enemy_eco.energy -= en_cost;
        let bid = engine.alloc_id();
        let mut building = engine
            .config
            .create_building(bid, Team::Enemy, b_type, wx, wy, false);
        building.hp = 1.0;
        building.build_progress = 0.1;
        building.rally_x = wx - 50.0;
        building.rally_y = wy;
        engine.buildings.push(building);
        true
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ai_state_default() {
        let state = AiState::default();
        assert_eq!(state.attack_wave, 0);
        assert!(!state.is_attacking);
        assert_eq!(state.wave_target_size, 10);
        assert!(!state.expansion_attempted);
    }

    #[test]
    fn test_ai_wave_scaling() {
        let mut state = AiState::default();
        assert_eq!(state.wave_target_size, 10);
        assert_eq!(state.attack_wave, 0);

        // Giả lập hoàn tất đợt 1
        state.attack_wave += 1;
        state.wave_target_size = match state.attack_wave {
            1 => 18,
            2 => 26,
            3 => 36,
            _ => 45,
        };
        assert_eq!(state.wave_target_size, 18);

        // Giả lập đợt 2
        state.attack_wave += 1;
        state.wave_target_size = match state.attack_wave {
            1 => 18,
            2 => 26,
            3 => 36,
            _ => 45,
        };
        assert_eq!(state.wave_target_size, 26);
    }

    #[test]
    fn test_converter_formula_ratio() {
        // Kiểm tra đúng công thức: Cứ 4 PowerReactor thì có 1 EnergyConverter
        let reactors_0 = 0;
        let converters_0 = reactors_0 / 4;
        assert_eq!(converters_0, 0);

        let reactors_3 = 3;
        let converters_3 = reactors_3 / 4;
        assert_eq!(converters_3, 0);

        let reactors_4 = 4;
        let converters_4 = reactors_4 / 4;
        assert_eq!(converters_4, 1);

        let reactors_7 = 7;
        let converters_7 = reactors_7 / 4;
        assert_eq!(converters_7, 1);

        let reactors_8 = 8;
        let converters_8 = reactors_8 / 4;
        assert_eq!(converters_8, 2);

        let reactors_16 = 16;
        let converters_16 = reactors_16 / 4;
        assert_eq!(converters_16, 4);
    }
}
