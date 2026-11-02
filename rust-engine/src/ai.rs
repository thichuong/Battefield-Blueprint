use crate::components::*;
use crate::rl::{BotStrategicAction, RlBrain, STATE_DIM};
use crate::GameEngine;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AiState {
    pub rally_point: (f32, f32),
    pub expansion_attempted: bool,
    pub last_attack_target: Option<(f32, f32)>,

    // RL Brain & Policy State
    pub rl_brain: RlBrain,
    pub current_action: BotStrategicAction,
    #[serde(default)]
    pub current_formation: FormationType,
    pub decision_timer: f32,
    pub last_state: Vec<f32>,
    pub cumulative_reward: f32,
    pub training_enabled: bool,
}

impl Default for AiState {
    fn default() -> Self {
        Self {
            rally_point: (1850.0, 580.0),
            expansion_attempted: false,
            last_attack_target: None,
            rl_brain: RlBrain::default(),
            current_action: BotStrategicAction::ExpandEco,
            current_formation: FormationType::Line,
            decision_timer: 0.0,
            last_state: vec![0.0; STATE_DIM],
            cumulative_reward: 0.0,
            training_enabled: false,
        }
    }
}

pub fn find_primary_hostile_team(engine: &GameEngine, team: Team) -> Team {
    let my_faction = engine.get_faction(team);
    // Tìm team thù địch có công trình hoặc lính đang hoạt động
    for i in 0..crate::components::MAX_PLAYERS {
        let candidate = Team::from_index(i);
        if engine.is_slot_active(candidate) && engine.get_faction(candidate) != my_faction {
            let has_presence = engine.buildings.iter().any(|b| b.team == candidate)
                || engine.units.iter().any(|u| u.team == candidate);
            if has_presence {
                return candidate;
            }
        }
    }
    // Dự phòng: trả về bất kỳ slot thù địch nào
    for i in 0..crate::components::MAX_PLAYERS {
        let candidate = Team::from_index(i);
        if engine.is_slot_active(candidate) && engine.get_faction(candidate) != my_faction {
            return candidate;
        }
    }
    if team == Team::Player { Team::Enemy } else { Team::Player }
}

pub fn find_weakest_hostile_team(engine: &GameEngine, team: Team) -> Team {
    let my_faction = engine.get_faction(team);
    let mut best_team = find_primary_hostile_team(engine, team);
    let mut lowest_score = f32::INFINITY;

    for i in 0..crate::components::MAX_PLAYERS {
        let candidate = Team::from_index(i);
        if engine.is_slot_active(candidate) && engine.get_faction(candidate) != my_faction {
            let cc_hp: f32 = engine.buildings.iter()
                .filter(|b| b.team == candidate && b.building_type == BuildingType::CommandCenter)
                .map(|b| b.hp)
                .sum();
            let combat_count = engine.units.iter()
                .filter(|u| u.team == candidate && u.unit_type != UnitType::Engineer)
                .count() as f32;

            if cc_hp > 0.0 {
                let score = cc_hp + combat_count * 150.0;
                if score < lowest_score {
                    lowest_score = score;
                    best_team = candidate;
                }
            }
        }
    }

    best_team
}

pub fn find_threatened_ally_base(engine: &GameEngine, team: Team) -> Option<(f32, f32)> {
    let my_faction = engine.get_faction(team);
    for i in 0..crate::components::MAX_PLAYERS {
        let ally_slot = Team::from_index(i);
        if engine.is_slot_active(ally_slot) && engine.get_faction(ally_slot) == my_faction {
            let base_pos = engine.base_pos(ally_slot);
            let has_threat = engine.units.iter().any(|u| {
                engine.are_hostile(team, u.team)
                    && ((u.x - base_pos.0).powi(2) + (u.y - base_pos.1).powi(2)) < 800.0_f32.powi(2)
            });
            if has_threat {
                return Some(base_pos);
            }
        }
    }
    None
}

pub struct TacticalSnapshot {
    pub my_workers: usize,
    pub my_combat: usize,
    pub my_ccs: usize,
    pub my_cc_total_hp: f32,
    pub my_cc_max_hp: f32,
    pub my_combat_total_hp: f32,
    pub my_combat_max_hp: f32,
    pub my_reactors: usize,
    pub my_converters: usize,
    pub my_barracks: usize,
    pub my_turrets: usize,

    pub my_frontline: usize,
    pub my_artillery: usize,
    pub my_magic: usize,

    pub opp_workers: usize,
    pub opp_combat: usize,
    pub opp_ccs: usize,
    pub opp_turrets: usize,
    pub opp_heavy_tanks: usize,
    pub opp_artillery: usize,
    pub opp_aoe_threat: usize,

    pub ally_combat: usize,
    pub nearest_enemy_dist: f32,
    pub base_threat: bool,
    pub active_hostile_factions: usize,
}

pub fn create_tactical_snapshot(engine: &GameEngine, team: Team) -> TacticalSnapshot {
    let _my_faction = engine.get_faction(team);
    let my_base = engine.base_pos(team);

    let mut snap = TacticalSnapshot {
        my_workers: 0,
        my_combat: 0,
        my_ccs: 0,
        my_cc_total_hp: 0.0,
        my_cc_max_hp: 0.0,
        my_combat_total_hp: 0.0,
        my_combat_max_hp: 0.0,
        my_reactors: 0,
        my_converters: 0,
        my_barracks: 0,
        my_turrets: 0,

        my_frontline: 0,
        my_artillery: 0,
        my_magic: 0,

        opp_workers: 0,
        opp_combat: 0,
        opp_ccs: 0,
        opp_turrets: 0,
        opp_heavy_tanks: 0,
        opp_artillery: 0,
        opp_aoe_threat: 0,

        ally_combat: 0,
        nearest_enemy_dist: crate::map::MAP_WIDTH,
        base_threat: false,
        active_hostile_factions: 0,
    };

    let mut hostile_factions = std::collections::HashSet::new();

    // 1 lượt quét duy nhất toàn bộ Units
    for u in &engine.units {
        let is_hostile = engine.are_hostile(team, u.team);
        if u.team == team {
            if u.unit_type == UnitType::Engineer {
                snap.my_workers += 1;
            } else {
                snap.my_combat += 1;
                snap.my_combat_total_hp += u.hp;
                snap.my_combat_max_hp += u.max_hp;

                match u.unit_type {
                    UnitType::ShockTrooper | UnitType::Vanguard | UnitType::BehemothTitan => {
                        snap.my_frontline += 1;
                    }
                    UnitType::Rifleman | UnitType::LaserMarksman | UnitType::DarkRocketeer => {
                        snap.my_artillery += 1;
                    }
                    UnitType::TaoistSwordmaster | UnitType::VoidWarlock | UnitType::AcidBaneling => {
                        snap.my_magic += 1;
                    }
                    _ => {}
                }
            }
        } else if is_hostile {
            hostile_factions.insert(engine.get_faction(u.team));
            if u.unit_type == UnitType::Engineer {
                snap.opp_workers += 1;
            } else {
                snap.opp_combat += 1;
                match u.unit_type {
                    UnitType::Vanguard | UnitType::BehemothTitan => {
                        snap.opp_heavy_tanks += 1;
                    }
                    UnitType::LaserMarksman | UnitType::DarkRocketeer => {
                        snap.opp_artillery += 1;
                    }
                    _ => {}
                }
                if matches!(u.unit_type, UnitType::DarkRocketeer | UnitType::VoidWarlock | UnitType::AcidBaneling | UnitType::BehemothTitan) {
                    snap.opp_aoe_threat += 1;
                }
            }

            let d = ((u.x - my_base.0).powi(2) + (u.y - my_base.1).powi(2)).sqrt();
            if d < snap.nearest_enemy_dist {
                snap.nearest_enemy_dist = d;
            }
            if d < 700.0 {
                snap.base_threat = true;
            }
        } else if engine.are_allies(team, u.team) && u.team != team && u.unit_type != UnitType::Engineer {
            snap.ally_combat += 1;
        }
    }

    // 1 lượt quét duy nhất toàn bộ Buildings
    for b in &engine.buildings {
        let is_hostile = engine.are_hostile(team, b.team);
        if u_or_b_is_my_team(b.team, team) {
            match b.building_type {
                BuildingType::CommandCenter => {
                    snap.my_ccs += 1;
                    snap.my_cc_total_hp += b.hp;
                    snap.my_cc_max_hp += b.max_hp;
                }
                BuildingType::PowerReactor => snap.my_reactors += 1,
                BuildingType::EnergyConverter => snap.my_converters += 1,
                BuildingType::Barracks => snap.my_barracks += 1,
                BuildingType::TurretTower => snap.my_turrets += 1,
                _ => {}
            }
        } else if is_hostile {
            hostile_factions.insert(engine.get_faction(b.team));
            if b.building_type == BuildingType::CommandCenter {
                snap.opp_ccs += 1;
            } else if b.building_type == BuildingType::TurretTower {
                snap.opp_turrets += 1;
            }
        }
    }

    snap.active_hostile_factions = hostile_factions.len();
    snap
}

#[inline]
fn u_or_b_is_my_team(a: Team, b: Team) -> bool {
    a == b
}

pub fn extract_state_vector(engine: &GameEngine) -> Vec<f32> {
    extract_state_vector_for_team(engine, Team::Enemy)
}

pub fn extract_state_vector_for_team(engine: &GameEngine, team: Team) -> Vec<f32> {
    let mut s = Vec::with_capacity(STATE_DIM);

    let my_eco = engine.team_eco(team);
    let my_tech = engine.team_tech(team);

    let opp_team = find_primary_hostile_team(engine, team);
    let opp_tech = engine.team_tech(opp_team);

    let snap = create_tactical_snapshot(engine, team);

    // 1. Tiến độ thời gian trận đấu (chuẩn hóa 0-1800s)
    let time_norm = (engine.game_time / 1800.0).clamp(0.0, 1.0);
    s.push(time_norm);

    // 2. Giai đoạn trận đấu: Early (0.0), Mid (0.5), Late (1.0)
    let game_phase = if engine.game_time < 240.0 {
        0.0
    } else if engine.game_time < 720.0 {
        0.5
    } else {
        1.0
    };
    s.push(game_phase);

    // 3-4. Kinh tế phe mình: Khoáng sản & Năng lượng hiện tại
    s.push((my_eco.minerals / 1000.0).clamp(0.0, 2.0));
    s.push((my_eco.energy / 500.0).clamp(0.0, 2.0));

    // 5. Tốc độ thu nhập khoáng sản / s
    let income_rate = (my_eco.minerals_per_sec / 50.0).clamp(0.0, 2.0);
    s.push(income_rate);

    // 6. Tỷ lệ Supply
    let supply_ratio = if my_eco.max_supply > 0 {
        (my_eco.supply as f32 / my_eco.max_supply as f32).clamp(0.0, 1.0)
    } else {
        0.0
    };
    s.push(supply_ratio);

    // 7. Chênh lệch Supply so với trần
    let supply_headroom = (my_eco.max_supply.saturating_sub(my_eco.supply)) as f32;
    s.push((supply_headroom / 10.0).clamp(0.0, 2.0));

    // 8-9. Số thợ mỏ & Số nhà chính của phe mình
    s.push((snap.my_workers as f32 / 18.0).clamp(0.0, 1.5));
    s.push((snap.my_ccs as f32 / 3.0).clamp(0.0, 1.0));

    // 10-13. Số Reactor, Converter, Barracks, Turret
    s.push((snap.my_reactors as f32 / 10.0).clamp(0.0, 1.5));
    s.push((snap.my_converters as f32 / 3.0).clamp(0.0, 1.0));
    s.push((snap.my_barracks as f32 / 6.0).clamp(0.0, 1.5));
    s.push((snap.my_turrets as f32 / 8.0).clamp(0.0, 1.5));

    // 14-16. Quân sự: Số lính chiến đấu phe mình vs đối thủ
    s.push((snap.my_combat as f32 / 35.0).clamp(0.0, 2.0));
    s.push((snap.opp_combat as f32 / 35.0).clamp(0.0, 2.0));
    let mil_ratio = ((snap.my_combat as f32 + 1.0) / (snap.opp_combat as f32 + 1.0)).clamp(0.1, 5.0) / 2.5 - 1.0;
    s.push(mil_ratio);

    // 17-18. Số công nghệ đã mở khóa của phe mình vs đối thủ (19 công nghệ)
    let my_unlocked = count_unlocked_techs(my_tech);
    let opp_unlocked = count_unlocked_techs(opp_tech);
    s.push((my_unlocked as f32 / 19.0).clamp(0.0, 1.0));
    s.push((opp_unlocked as f32 / 19.0).clamp(0.0, 1.0));

    // 19. Căn cứ phe mình có đang bị đe dọa trực tiếp
    s.push(if snap.base_threat { 1.0 } else { 0.0 });

    // 20. Khoảng cách gần nhất từ cánh quân đối thủ tới căn cứ phe mình
    s.push((snap.nearest_enemy_dist / crate::map::MAP_WIDTH).clamp(0.0, 1.0));

    // 21. Thợ mỏ của đối thủ
    s.push((snap.opp_workers as f32 / 16.0).clamp(0.0, 1.5));

    // 22. Tình trạng nhà chính của đối thủ
    s.push((snap.opp_ccs as f32 / 2.0).clamp(0.0, 1.0));

    // 23. Mức độ đe dọa sát thương diện rộng AoE của đối phương
    s.push((snap.opp_aoe_threat as f32 / 8.0).clamp(0.0, 2.0));

    // 24. Đội hình hiện tại của phe mình
    let cur_form = engine.team_formation(team).to_index() as f32 / 5.0;
    s.push(cur_form);

    // --- MỞ RỘNG TỪ 24 LÊN 40 CHIỀU ---

    // 25. Tốc độ thu nhập Năng lượng / s
    s.push((my_eco.energy_per_sec / 100.0).clamp(0.0, 2.0));

    // 26. Tỷ lệ hàng đợi nghiên cứu đang hoạt động (0..4)
    s.push((my_tech.active_researches.len() as f32 / 4.0).clamp(0.0, 1.0));

    // 27-30. Tiến độ 4 nhánh công nghệ riêng biệt
    let (sci, mag, mon, log) = count_branch_techs(my_tech);
    s.push((sci as f32 / 4.0).clamp(0.0, 1.0));
    s.push((mag as f32 / 4.0).clamp(0.0, 1.0));
    s.push((mon as f32 / 4.0).clamp(0.0, 1.0));
    s.push((log as f32 / 7.0).clamp(0.0, 1.0));

    // 31. Tỷ lệ % HP trung bình của các CommandCenter phe mình
    let cc_hp_ratio = if snap.my_cc_max_hp > 0.0 {
        (snap.my_cc_total_hp / snap.my_cc_max_hp).clamp(0.0, 1.0)
    } else {
        0.0
    };
    s.push(cc_hp_ratio);

    // 32. Tỷ lệ % HP trung bình của toàn quân đội phe mình
    let combat_hp_ratio = if snap.my_combat_max_hp > 0.0 {
        (snap.my_combat_total_hp / snap.my_combat_max_hp).clamp(0.0, 1.0)
    } else {
        0.0
    };
    s.push(combat_hp_ratio);

    // 33-35. Cơ cấu binh chủng phe mình (Frontline, Artillery, Magic)
    let total_my_c = (snap.my_combat as f32).max(1.0);
    s.push((snap.my_frontline as f32 / total_my_c).clamp(0.0, 1.0));
    s.push((snap.my_artillery as f32 / total_my_c).clamp(0.0, 1.0));
    s.push((snap.my_magic as f32 / total_my_c).clamp(0.0, 1.0));

    // 36-38. Cơ cấu lực lượng đối phương (Heavy Tanks, Artillery, Turrets)
    let total_opp_c = (snap.opp_combat as f32).max(1.0);
    s.push((snap.opp_heavy_tanks as f32 / total_opp_c).clamp(0.0, 1.0));
    s.push((snap.opp_artillery as f32 / total_opp_c).clamp(0.0, 1.0));
    s.push((snap.opp_turrets as f32 / 12.0).clamp(0.0, 1.5));

    // 39. Tương quan lực lượng Đồng minh (cho chế độ 2v2, 3v3, 4v4, 2v2v2)
    s.push((snap.ally_combat as f32 / 40.0).clamp(0.0, 2.0));

    // 40. Số lượng Phe thù địch còn sống trên bản đồ (quan trọng cho 8-FFA và 2v2v2)
    s.push((snap.active_hostile_factions as f32 / 7.0).clamp(0.0, 1.0));

    // --- MỞ RỘNG TỪ 40 LÊN 64 CHIỀU: CHUYÊN SÂU FARM NĂNG LƯỢNG & KHOÁNG SẢN ---
    // 41. Trữ lượng năng lượng so với ngưỡng chuyển đổi 1000.0 (EnergyConverter Threshold)
    s.push((my_eco.energy / crate::config::ENERGY_CONVERSION_THRESHOLD).clamp(0.0, 3.0));

    // 42. Năng lượng thặng dư trên ngưỡng 1000.0 (năng lượng sẵn sàng quy đổi ra khoáng)
    let surplus_energy = (my_eco.energy - crate::config::ENERGY_CONVERSION_THRESHOLD).max(0.0);
    s.push((surplus_energy / 1500.0).clamp(0.0, 3.0));

    // 43. Tốc độ chuyển đổi năng lượng -> khoáng sản lý thuyết/giây hiện tại
    let conversion_potential = (snap.my_converters as f32 * crate::config::CONVERT_MINERALS_PER_SEC / 100.0).clamp(0.0, 3.0);
    s.push(conversion_potential);

    // 44. Tỷ lệ cân bằng vàng giữa Converter và Reactor (lý tưởng là 1 Converter cho 2-3 Reactor)
    let conv_per_reactor_ratio = if snap.my_reactors > 0 {
        (snap.my_converters as f32 / (snap.my_reactors as f32 * 0.5)).clamp(0.0, 2.0)
    } else {
        0.0
    };
    s.push(conv_per_reactor_ratio);

    // 45. Trạng thái công nghệ nạp siêu tốc lò phản ứng (overcharged_reactors: +50% sản lượng điện)
    s.push(if my_tech.overcharged_reactors { 1.0 } else { 0.0 });

    // 46. Trạng thái công nghệ mũi khoan quang học (optical_drills: +50% sức chứa khai thác)
    s.push(if my_tech.optical_drills { 1.0 } else { 0.0 });

    // 47. Trạng thái công nghệ bản vẽ kiên cố (fortified_blueprints: +30% HP công trình)
    s.push(if my_tech.fortified_blueprints { 1.0 } else { 0.0 });

    // 48. Số lượng PowerReactor của đối thủ
    let opp_reactors = engine.buildings.iter().filter(|b| b.team == opp_team && b.building_type == BuildingType::PowerReactor).count();
    s.push((opp_reactors as f32 / 20.0).clamp(0.0, 2.0));

    // 49. Số lượng EnergyConverter của đối thủ
    let opp_converters = engine.buildings.iter().filter(|b| b.team == opp_team && b.building_type == BuildingType::EnergyConverter).count();
    s.push((opp_converters as f32 / 10.0).clamp(0.0, 2.0));

    // 50. Chênh lệch sản lượng năng lượng giữa phe mình và đối thủ
    let opp_eco = engine.team_eco(opp_team);
    let energy_flow_diff = ((my_eco.energy_per_sec - opp_eco.energy_per_sec) / 50.0).clamp(-2.0, 2.0);
    s.push(energy_flow_diff);

    // 51. Chênh lệch thu nhập khoáng sản giữa phe mình và đối thủ
    let min_flow_diff = ((my_eco.minerals_per_sec - opp_eco.minerals_per_sec) / 50.0).clamp(-2.0, 2.0);
    s.push(min_flow_diff);

    // 52. Số lượng mỏ khoáng còn trữ lượng trong bán kính 900px quanh căn cứ phe mình
    let my_base = engine.base_pos(team);
    let nearby_minerals_count = engine.minerals.iter().filter(|m| {
        m.amount > 0.0 && ((m.x - my_base.0).powi(2) + (m.y - my_base.1).powi(2)) < 900.0_f32.powi(2)
    }).count();
    s.push((nearby_minerals_count as f32 / 12.0).clamp(0.0, 1.5));

    // 53. Tổng lượng khoáng sản còn lại quanh căn cứ
    let nearby_minerals_amount: f32 = engine.minerals.iter().filter(|m| {
        m.amount > 0.0 && ((m.x - my_base.0).powi(2) + (m.y - my_base.1).powi(2)) < 900.0_f32.powi(2)
    }).map(|m| m.amount).sum();
    s.push((nearby_minerals_amount / 20000.0).clamp(0.0, 2.0));

    // 54. Đang có PowerReactor đang xây dở không
    let reactors_under_construction = engine.buildings.iter().filter(|b| {
        b.team == team && b.building_type == BuildingType::PowerReactor && b.build_progress < 1.0
    }).count();
    s.push((reactors_under_construction as f32 / 3.0).clamp(0.0, 1.0));

    // 55. Đang có EnergyConverter đang xây dở không
    let converters_under_construction = engine.buildings.iter().filter(|b| {
        b.team == team && b.building_type == BuildingType::EnergyConverter && b.build_progress < 1.0
    }).count();
    s.push((converters_under_construction as f32 / 2.0).clamp(0.0, 1.0));

    // 56. Tỷ lệ thợ mỏ đang rảnh rỗi (idle workers)
    let idle_workers = engine.units.iter().filter(|u| {
        u.team == team && u.unit_type == UnitType::Engineer && u.order == UnitOrder::Idle
    }).count();
    s.push((idle_workers as f32 / (snap.my_workers.max(1) as f32)).clamp(0.0, 1.0));

    // 57. Số lượng nhà Barracks đang xây dở
    let barracks_under_construction = engine.buildings.iter().filter(|b| {
        b.team == team && b.building_type == BuildingType::Barracks && b.build_progress < 1.0
    }).count();
    s.push((barracks_under_construction as f32 / 3.0).clamp(0.0, 1.0));

    // 58. Tỷ lệ % tải của các hàng đợi sản xuất quân lính
    let barracks_queue_load: f32 = engine.buildings.iter().filter(|b| {
        b.team == team && b.building_type == BuildingType::Barracks && b.build_progress >= 1.0
    }).map(|b| b.training_queue.len() as f32).sum::<f32>() / (snap.my_barracks.max(1) as f32 * 2.0);
    s.push(barracks_queue_load.clamp(0.0, 1.0));

    // 59. Số lượng tháp pháo đang xây dở
    let turrets_under_construction = engine.buildings.iter().filter(|b| {
        b.team == team && b.building_type == BuildingType::TurretTower && b.build_progress < 1.0
    }).count();
    s.push((turrets_under_construction as f32 / 4.0).clamp(0.0, 1.0));

    // 60. Khoảng cách an toàn của mỏ khoáng gần nhất so với quân địch
    let dist_mineral_to_enemy = engine.minerals.iter()
        .filter(|m| m.amount > 0.0 && ((m.x - my_base.0).powi(2) + (m.y - my_base.1).powi(2)) < 900.0_f32.powi(2))
        .map(|m| {
            engine.units.iter()
                .filter(|u| engine.are_hostile(team, u.team))
                .map(|u| ((u.x - m.x).powi(2) + (u.y - m.y).powi(2)).sqrt())
                .fold(f32::INFINITY, f32::min)
        })
        .fold(f32::INFINITY, f32::min);
    let norm_safe_dist = if dist_mineral_to_enemy.is_infinite() { 1.0 } else { (dist_mineral_to_enemy / 2000.0).clamp(0.0, 1.0) };
    s.push(norm_safe_dist);

    // 61. Trạng thái công nghệ nhánh Ma thuật cấp cao (void_cataclysm)
    s.push(if my_tech.void_cataclysm { 1.0 } else { 0.0 });

    // 62. Trạng thái công nghệ nhánh Quái vật cấp cao (behemoth_mutation)
    s.push(if my_tech.behemoth_mutation { 1.0 } else { 0.0 });

    // 63. Trạng thái công nghệ nhánh Khoa học cấp cao (dark_matter_rockets)
    s.push(if my_tech.dark_matter_rockets { 1.0 } else { 0.0 });

    // 64. Trạng thái sẵn sàng tổng lực công kích (Macro Readiness: kinh tế mạnh + quân lực dồi dào)
    let macro_readiness = if snap.my_combat >= 20 && snap.my_reactors >= 6 && snap.my_converters >= 2 {
        1.0
    } else {
        (snap.my_combat as f32 / 20.0 * 0.5 + snap.my_reactors as f32 / 6.0 * 0.5).clamp(0.0, 1.0)
    };
    s.push(macro_readiness);

    // Đảm bảo đúng 64 chiều
    if s.len() > STATE_DIM {
        s.truncate(STATE_DIM);
    }
    while s.len() < STATE_DIM {
        s.push(0.0);
    }

    s
}

pub fn count_branch_techs(tech: &crate::resources::TechTreeState) -> (usize, usize, usize, usize) {
    let mut sci = 0;
    if tech.focused_laser { sci += 1; }
    if tech.energy_shielding { sci += 1; }
    if tech.dark_matter_rockets { sci += 1; }
    if tech.titan_alloy_plating { sci += 1; }

    let mut mag = 0;
    if tech.celestial_flying_swords { mag += 1; }
    if tech.golden_bell_shield { mag += 1; }
    if tech.corrosive_curse { mag += 1; }
    if tech.void_cataclysm { mag += 1; }

    let mut mon = 0;
    if tech.primeval_scales { mon += 1; }
    if tech.behemoth_mutation { mon += 1; }
    if tech.acid_banelings { mon += 1; }
    if tech.hive_proliferation { mon += 1; }

    let mut log = 0;
    if tech.optical_drills { log += 1; }
    if tech.overcharged_reactors { log += 1; }
    if tech.fortified_blueprints { log += 1; }
    if tech.magnetic_rounds { log += 1; }
    if tech.exo_boosters { log += 1; }
    if tech.superheated_blades { log += 1; }
    if tech.overload_strike { log += 1; }

    (sci, mag, mon, log)
}

pub fn count_unlocked_techs(tech: &crate::resources::TechTreeState) -> usize {
    let (sci, mag, mon, log) = count_branch_techs(tech);
    sci + mag + mon + log
}

pub fn update_ai(engine: &mut GameEngine) {
    update_ai_with_dt(engine, 0.5);
}

pub fn update_ai_with_dt(engine: &mut GameEngine, dt: f32) {
    for i in 1..crate::components::MAX_PLAYERS {
        let team = Team::from_index(i);
        if engine.is_slot_active(team) && !engine.player_slots[i].is_human && !engine.team_autoplay(team).config.enabled {
            let mut ai_state = engine.team_ai(team).clone();
            update_ai_for_team_with_dt(engine, team, &mut ai_state, dt);
            *engine.team_ai_mut(team) = ai_state;
        }
    }
}

pub fn update_ai_for_team(engine: &mut GameEngine, team: Team, ai_state: &mut AiState) {
    update_ai_for_team_with_dt(engine, team, ai_state, 0.5);
}

pub fn update_ai_for_team_with_dt(engine: &mut GameEngine, team: Team, ai_state: &mut AiState, dt: f32) {
    // Tự động phân công thợ mỏ rảnh đi đào quặng
    assign_idle_team_workers(engine, team);

    // Chu kỳ ra quyết định RL (mỗi 1.5s in-game)
    ai_state.decision_timer += dt;
    if ai_state.decision_timer >= 1.5 {
        ai_state.decision_timer = 0.0;

        let state = extract_state_vector_for_team(engine, team);
        let action = ai_state.rl_brain.select_action(&state, ai_state.training_enabled);
        let formation = ai_state.rl_brain.select_formation(&state, ai_state.training_enabled);

        ai_state.current_action = action;
        ai_state.current_formation = formation;
        ai_state.last_state = state;
        engine.set_team_formation(team, formation);

        apply_strategic_action_for_team(engine, team, action);
    }

    // Luôn duy trì phản xạ vi mô liên tục
    maintain_continuous_ai_reflexes_for_team(engine, team);
}

pub fn assign_idle_team_workers(engine: &mut GameEngine, team: Team) {
    let available_minerals: Vec<(u32, f32, f32)> = engine
        .minerals
        .iter()
        .filter(|m| m.amount > 0.0)
        .map(|m| (m.id, m.x, m.y))
        .collect();

    if available_minerals.is_empty() {
        return;
    }

    let team_workers: Vec<u32> = engine
        .units
        .iter()
        .filter(|u| u.team == team && u.unit_type == UnitType::Engineer && u.order == UnitOrder::Idle)
        .map(|u| u.id)
        .collect();

    for wid in team_workers {
        if let Some(w) = engine.units.iter_mut().find(|u| u.id == wid) {
            let nearest_mineral = available_minerals.iter().min_by(|a, b| {
                let da = (a.1 - w.x).powi(2) + (a.2 - w.y).powi(2);
                let db = (b.1 - w.x).powi(2) + (b.2 - w.y).powi(2);
                da.partial_cmp(&db).unwrap()
            }).map(|m| m.0);

            if let Some(target_mineral_id) = nearest_mineral {
                w.order = UnitOrder::Harvest {
                    target_mineral_id,
                    phase: HarvestPhase::MovingToNode,
                };
            }
        }
    }
}

pub fn apply_strategic_action(engine: &mut GameEngine, action: BotStrategicAction) {
    apply_strategic_action_for_team(engine, Team::Enemy, action);
}

pub fn apply_strategic_action_for_team(engine: &mut GameEngine, team: Team, action: BotStrategicAction) {
    match action {
        BotStrategicAction::ExpandEco => {
            execute_team_eco(engine, team, true);
            execute_team_training(engine, team, &[UnitType::ShockTrooper, UnitType::Rifleman]);
        }
        BotStrategicAction::BuildInfrastructure => {
            execute_team_infrastructure(engine, team);
            execute_team_training(engine, team, &[UnitType::ShockTrooper, UnitType::Rifleman]);
        }
        BotStrategicAction::BuildDefenseGrid => {
            execute_team_defense(engine, team);
            execute_team_training(engine, team, &[UnitType::ShockTrooper, UnitType::Rifleman]);
        }
        BotStrategicAction::TechScienceBranch => {
            try_team_research(engine, team, &["focused_laser", "energy_shielding", "dark_matter_rockets", "titan_alloy_plating"]);
            execute_team_training(engine, team, &[UnitType::LaserMarksman, UnitType::DarkRocketeer, UnitType::BehemothTitan, UnitType::Vanguard, UnitType::ShockTrooper]);
        }
        BotStrategicAction::TechMagicBranch => {
            try_team_research(engine, team, &["celestial_flying_swords", "golden_bell_shield", "corrosive_curse", "void_cataclysm"]);
            execute_team_training(engine, team, &[UnitType::VoidWarlock, UnitType::TaoistSwordmaster, UnitType::DarkRocketeer, UnitType::Rifleman]);
        }
        BotStrategicAction::TechMonsterBranch => {
            try_team_research(engine, team, &["primeval_scales", "behemoth_mutation", "acid_banelings", "hive_proliferation"]);
            execute_team_training(engine, team, &[UnitType::BehemothTitan, UnitType::AcidBaneling, UnitType::VoidWarlock, UnitType::LaserMarksman, UnitType::ShockTrooper]);
        }
        BotStrategicAction::MilitarySwarmRush => {
            execute_team_training(engine, team, &[UnitType::AcidBaneling, UnitType::ShockTrooper, UnitType::Rifleman, UnitType::DarkRocketeer]);
            command_team_military(engine, team, MilitaryCommand::HarassOrAttack);
        }
        BotStrategicAction::DefendAndCounter => {
            command_team_military(engine, team, MilitaryCommand::DefendBase);
            execute_team_training(engine, team, &[UnitType::BehemothTitan, UnitType::Vanguard, UnitType::LaserMarksman, UnitType::Rifleman]);
        }
        BotStrategicAction::HarassEconomy => {
            command_team_military(engine, team, MilitaryCommand::HarassWorkers);
            execute_team_training(engine, team, &[UnitType::AcidBaneling, UnitType::TaoistSwordmaster, UnitType::ShockTrooper]);
        }
        BotStrategicAction::AllOutAssault => {
            command_team_military(engine, team, MilitaryCommand::AllOutAssault);
            execute_team_training(engine, team, &[UnitType::BehemothTitan, UnitType::DarkRocketeer, UnitType::VoidWarlock, UnitType::LaserMarksman, UnitType::ShockTrooper]);
        }
        BotStrategicAction::RegroupAndConsolidate => {
            command_team_military(engine, team, MilitaryCommand::Regroup);
            execute_team_training(engine, team, &[UnitType::BehemothTitan, UnitType::Vanguard, UnitType::DarkRocketeer, UnitType::Rifleman]);
        }
        BotStrategicAction::TechLogisticsBranch => {
            try_team_research(engine, team, &[
                "optical_drills",
                "overcharged_reactors",
                "fortified_blueprints",
                "magnetic_rounds",
                "exo_boosters",
                "superheated_blades",
                "overload_strike",
            ]);
            execute_team_training(engine, team, &[UnitType::Rifleman, UnitType::ShockTrooper, UnitType::DarkRocketeer, UnitType::Vanguard]);
        }
        BotStrategicAction::SiegeArtilleryPush => {
            // Đội hình công thành phá tháp pháo địch
            command_team_military(engine, team, MilitaryCommand::SiegePush);
            execute_team_training(engine, team, &[UnitType::BehemothTitan, UnitType::DarkRocketeer, UnitType::LaserMarksman, UnitType::Vanguard]);
        }
        BotStrategicAction::CoordinateAllySupport => {
            if let Some(ally_threat_pos) = find_threatened_ally_base(engine, team) {
                command_team_military(engine, team, MilitaryCommand::SupportAlly(ally_threat_pos));
            } else {
                command_team_military(engine, team, MilitaryCommand::AllOutAssault);
            }
            execute_team_training(engine, team, &[UnitType::BehemothTitan, UnitType::DarkRocketeer, UnitType::VoidWarlock, UnitType::ShockTrooper]);
        }
        BotStrategicAction::FocusWeakestEnemy => {
            let weakest = find_weakest_hostile_team(engine, team);
            let weakest_base = engine.base_pos(weakest);
            command_team_military(engine, team, MilitaryCommand::AttackSpecific(weakest_base));
            execute_team_training(engine, team, &[UnitType::AcidBaneling, UnitType::TaoistSwordmaster, UnitType::DarkRocketeer, UnitType::LaserMarksman]);
        }
    }
}

pub fn maintain_continuous_ai_reflexes(engine: &mut GameEngine) {
    maintain_continuous_ai_reflexes_for_team(engine, Team::Enemy);
}

pub fn maintain_continuous_ai_reflexes_for_team(engine: &mut GameEngine, team: Team) {
    let _opp_team = find_primary_hostile_team(engine, team);
    let base_pos = engine.base_pos(team);

    let team_workers = engine.units.iter().filter(|u| u.team == team && u.unit_type == UnitType::Engineer).count();
    let team_ccs = engine.buildings.iter().filter(|b| b.team == team && b.building_type == BuildingType::CommandCenter).count();
    let worker_target = if team_ccs > 1 { 22 } else { 14 };

    // 1. Duy trì sản xuất thợ mỏ cơ bản
    let (eco_supply, eco_max_supply, eco_minerals) = {
        let eco = engine.team_eco(team);
        (eco.supply, eco.max_supply, eco.minerals)
    };

    if team_workers < worker_target && eco_supply < eco_max_supply && eco_minerals >= 50.0 {
        let cc_id = engine.buildings.iter().find(|b| {
            b.team == team && b.building_type == BuildingType::CommandCenter && b.build_progress >= 1.0 && b.training_queue.len() < 2
        }).map(|b| b.id);
        if let Some(bid) = cc_id {
            engine.team_eco_mut(team).minerals -= 50.0;
            if let Some(cc) = engine.buildings.iter_mut().find(|b| b.id == bid) {
                cc.training_queue.push(UnitType::Engineer);
            }
        }
    }

    // 2. Chống kẹt dân số (Supply Depot)
    let under_construction_depots = engine.buildings.iter().filter(|b| {
        b.team == team && b.building_type == BuildingType::SupplyDepot && b.build_progress < 1.0
    }).count();

    let (eco_supply, eco_max_supply, eco_minerals) = {
        let eco = engine.team_eco(team);
        (eco.supply, eco.max_supply, eco.minerals)
    };

    let needs_depot = eco_supply + 6 >= eco_max_supply;
    if needs_depot && eco_max_supply < 200 && under_construction_depots == 0 && eco_minerals >= 100.0 {
        let (depot_cost, _) = engine.config.get_building_cost(BuildingType::SupplyDepot);
        if eco_minerals >= depot_cost {
            if let Some((bx, by)) = crate::autoplay::find_team_build_spot(engine, team, BuildingType::SupplyDepot) {
                crate::autoplay::try_team_auto_build(engine, team, BuildingType::SupplyDepot, bx, by);
            }
        }
    }

    // 3. Xây Power Reactor
    let reactors = engine.buildings.iter().filter(|b| b.team == team && b.building_type == BuildingType::PowerReactor).count();
    let under_construction_reactors = engine.buildings.iter().filter(|b| {
        b.team == team && b.building_type == BuildingType::PowerReactor && b.build_progress < 1.0
    }).count();
    let eco_minerals = engine.team_eco(team).minerals;

    // Cho phép xây tối đa 36 PowerReactors, xây song song tới 2 lò cùng lúc khi kinh tế dư dả
    let max_parallel_reactors = if eco_minerals >= 300.0 { 2 } else { 1 };
    if reactors < 36 && under_construction_reactors < max_parallel_reactors {
        let should_build_reactor = if reactors < 4 {
            eco_minerals >= 100.0
        } else if reactors < 12 {
            eco_minerals >= 120.0
        } else {
            eco_minerals >= 150.0
        };

        if should_build_reactor {
            if let Some((bx, by)) = crate::autoplay::find_team_build_spot(engine, team, BuildingType::PowerReactor) {
                crate::autoplay::try_team_auto_build(engine, team, BuildingType::PowerReactor, bx, by);
            }
        }
    }

    // 3.5. Xây EnergyConverter (chuyển đổi Năng lượng thành Khoáng sản)
    let converters = engine.buildings.iter().filter(|b| b.team == team && b.building_type == BuildingType::EnergyConverter).count();
    let under_construction_conv = engine.buildings.iter().filter(|b| {
        b.team == team && b.building_type == BuildingType::EnergyConverter && b.build_progress < 1.0
    }).count();
    let (eco_min, eco_en) = (engine.team_eco(team).minerals, engine.team_eco(team).energy);

    // Cho phép xây tối đa 16 EnergyConverters (tỷ lệ 1 converter cho 2 reactors, hoặc kích hoạt sớm khi có điện)
    let max_parallel_converters = if eco_min >= 350.0 && eco_en >= 200.0 { 2 } else { 1 };
    let desired_converters = (reactors / 2).clamp(1, 16);
    let should_build_converter = converters < desired_converters
        && (reactors >= 2 || eco_en >= 200.0)
        && eco_min >= 150.0
        && eco_en >= 50.0;

    if under_construction_conv < max_parallel_converters && should_build_converter {
        if let Some((bx, by)) = crate::autoplay::find_team_build_spot(engine, team, BuildingType::EnergyConverter) {
            crate::autoplay::try_team_auto_build(engine, team, BuildingType::EnergyConverter, bx, by);
        }
    }

    // 4. Xây Barracks
    let barracks = engine.buildings.iter().filter(|b| b.team == team && b.building_type == BuildingType::Barracks).count();
    let under_construction_barracks = engine.buildings.iter().any(|b| {
        b.team == team && b.building_type == BuildingType::Barracks && b.build_progress < 1.0
    });
    if !under_construction_barracks && eco_min >= 150.0 {
        let should_build_barracks = match barracks {
            0 => true,
            1 => team_workers >= 6,
            2 => team_workers >= 10 && reactors >= 2,
            3 => team_workers >= 12 && reactors >= 3,
            4 => team_workers >= 14 && reactors >= 3,
            5 => team_workers >= 14 && reactors >= 4,
            6 => team_workers >= 14 && reactors >= 4,
            7 => eco_min >= 200.0,
            8 => eco_min >= 220.0,
            9 => eco_min >= 250.0,
            _ => false,
        };
        if should_build_barracks {
            if let Some((bx, by)) = crate::autoplay::find_team_build_spot(engine, team, BuildingType::Barracks) {
                crate::autoplay::try_team_auto_build(engine, team, BuildingType::Barracks, bx, by);
            }
        }
    }

    // 4.5. Xây TurretTower
    let turrets = engine.buildings.iter().filter(|b| b.team == team && b.building_type == BuildingType::TurretTower).count();
    let under_construction_turrets = engine.buildings.iter().any(|b| {
        b.team == team && b.building_type == BuildingType::TurretTower && b.build_progress < 1.0
    });
    if turrets < 10 && !under_construction_turrets && eco_min >= 100.0 && eco_en >= 50.0 {
        if let Some((bx, by)) = crate::autoplay::find_team_build_spot(engine, team, BuildingType::TurretTower) {
            crate::autoplay::try_team_auto_build(engine, team, BuildingType::TurretTower, bx, by);
        }
    }

    // 4.8. Nghiên cứu công nghệ liên tục
    let (active_res_len, tech_en) = (engine.team_tech(team).active_researches.len(), engine.team_eco(team).energy);
    if active_res_len < crate::config::MAX_TOTAL_CONCURRENT_RESEARCH && tech_en >= 80.0 {
        let tech_priorities = [
            "optical_drills",
            "focused_laser",
            "energy_shielding",
            "dark_matter_rockets",
            "primeval_scales",
            "behemoth_mutation",
            "corrosive_curse",
            "void_cataclysm",
            "celestial_flying_swords",
            "golden_bell_shield",
            "acid_banelings",
            "hive_proliferation",
            "overcharged_reactors",
            "titan_alloy_plating",
            "fortified_blueprints",
            "magnetic_rounds",
            "exo_boosters",
            "superheated_blades",
            "overload_strike",
        ];
        for &t in &tech_priorities {
            let (needs_research, target) = {
                let team_tech = engine.team_tech(team);
                if !crate::simulation::is_tech_unlocked(team_tech, t)
                    && !team_tech.active_researches.iter().any(|r| r.tech_id == t)
                {
                    let mut target_id = t.to_string();
                    if let Some(req) = crate::autoplay::get_tech_prerequisite(t) {
                        if !crate::simulation::is_tech_unlocked(team_tech, &req)
                            && !team_tech.active_researches.iter().any(|r| r.tech_id == req)
                        {
                            target_id = req;
                        }
                    }
                    (true, target_id)
                } else {
                    (false, String::new())
                }
            };

            if needs_research {
                let (team_tech, team_eco, cfg) = match team {
                    Team::Player => (&mut engine.player_tech, &mut engine.player_eco, &engine.config),
                    Team::Enemy => (&mut engine.enemy_tech, &mut engine.enemy_eco, &engine.config),
                    _ => {
                        let idx = team.index() - 2;
                        (&mut engine.extra_techs[idx], &mut engine.extra_ecos[idx], &engine.config)
                    }
                };
                if crate::simulation::process_start_research(team_tech, team_eco, cfg, &target) {
                    break;
                }
            }
        }
    }

    // 5. Tuyển quân liên tục
    execute_team_continuous_recruitment(engine, team);

    // 6. Điều phối tác chiến
    let my_combat = engine.units.iter().filter(|u| u.team == team && u.unit_type != UnitType::Engineer).count();

    // 6.1. Nếu căn cứ phe mình hoặc đồng minh bị đe dọa trực tiếp (<750px), phòng thủ
    let has_threat = engine.units.iter().any(|u| {
        engine.are_hostile(team, u.team) && ((u.x - base_pos.0).powi(2) + (u.y - base_pos.1).powi(2)) < 750.0_f32.powi(2)
    });
    if has_threat {
        command_team_military(engine, team, MilitaryCommand::DefendBase);
        return;
    }

    // 6.2. Tiến công
    let tech = engine.team_tech(team);
    let has_siege_tech = tech.dark_matter_rockets || tech.focused_laser;
    let assault_threshold = if has_siege_tech { 16 } else { 22 };

    let is_on_offensive = my_combat >= assault_threshold || engine.units.iter().any(|u| {
        u.team == team && u.unit_type != UnitType::Engineer && ((u.x - base_pos.0).powi(2) + (u.y - base_pos.1).powi(2)) > 1000.0_f32.powi(2)
    });

    if is_on_offensive {
        command_team_military(engine, team, MilitaryCommand::AllOutAssault);
        return;
    }

    // 6.3. Tập kết quân mới sinh
    let center = (crate::map::MAP_WIDTH * 0.5, crate::map::MAP_HEIGHT * 0.5);
    let dx = center.0 - base_pos.0;
    let dy = center.1 - base_pos.1;
    let len = (dx * dx + dy * dy).sqrt().max(1.0);
    let rally_pos = (base_pos.0 + dx / len * 400.0, base_pos.1 + dy / len * 400.0);

    let idle_combat_pairs: Vec<(u32, UnitType)> = engine.units.iter()
        .filter(|u| {
            u.team == team && u.unit_type != UnitType::Engineer && u.order == UnitOrder::Idle
                && ((u.x - base_pos.0).powi(2) + (u.y - base_pos.1).powi(2)) < 650.0_f32.powi(2)
        })
        .map(|u| (u.id, u.unit_type))
        .collect();

    if !idle_combat_pairs.is_empty() {
        let form = engine.team_formation(team);
        let form_pos = crate::formation::compute_formation_positions(
            &idle_combat_pairs,
            base_pos,
            rally_pos,
            form,
        );
        for (uid, tx, ty, off_x, off_y) in form_pos {
            if let Some(u) = engine.units.iter_mut().find(|u| u.id == uid) {
                u.formation_offset = (off_x, off_y);
                u.order = UnitOrder::MoveTo { x: tx, y: ty };
            }
        }
    }
}

pub fn execute_team_continuous_recruitment(engine: &mut GameEngine, team: Team) {
    let (supply, max_supply) = {
        let eco = engine.team_eco(team);
        (eco.supply, eco.max_supply)
    };
    if supply >= max_supply {
        return;
    }

    let team_combat = engine.units.iter().filter(|u| u.team == team && u.unit_type != UnitType::Engineer).count();
    let tech = engine.team_tech(team);

    let needs_tech = !tech.dark_matter_rockets || !tech.behemoth_mutation || !tech.void_cataclysm;
    let mineral_reserve = if needs_tech && team_combat >= 8 { 180.0 } else if team_combat >= 8 { 70.0 } else { 0.0 };

    let completed_barracks: Vec<u32> = engine.buildings.iter()
        .filter(|b| b.team == team && b.building_type == BuildingType::Barracks && b.build_progress >= 1.0 && b.training_queue.len() < 2)
        .map(|b| b.id)
        .collect();

    let count_unit = |ut: UnitType| -> usize {
        engine.units.iter().filter(|u| u.team == team && u.unit_type == ut).count()
    };
    let titans = count_unit(UnitType::BehemothTitan);
    let rockets = count_unit(UnitType::DarkRocketeer);
    let lasers = count_unit(UnitType::LaserMarksman);
    let warlocks = count_unit(UnitType::VoidWarlock);
    let swords = count_unit(UnitType::TaoistSwordmaster);
    let shields = count_unit(UnitType::Vanguard);
    let banelings = count_unit(UnitType::AcidBaneling);

    let mut candidate_order = vec![
        (UnitType::BehemothTitan, titans, 4),
        (UnitType::DarkRocketeer, rockets, 6),
        (UnitType::LaserMarksman, lasers, 5),
        (UnitType::VoidWarlock, warlocks, 4),
        (UnitType::TaoistSwordmaster, swords, 5),
        (UnitType::Vanguard, shields, 4),
        (UnitType::AcidBaneling, banelings, 6),
    ];
    candidate_order.sort_by_key(|&(_, cur, quota)| {
        if cur >= quota { 100 + cur } else { cur }
    });

    for bid in completed_barracks {
        let (cur_supply, cur_max_supply) = {
            let eco = engine.team_eco(team);
            (eco.supply, eco.max_supply)
        };
        if cur_supply >= cur_max_supply {
            break;
        }

        let mut recruited = false;
        let tech = engine.team_tech(team).clone();

        for &(u_type, cur_cnt, quota) in &candidate_order {
            if !crate::autoplay::can_train_unit(&tech, u_type) {
                continue;
            }
            if cur_cnt >= quota && candidate_order.iter().any(|&(other_u, other_c, other_q)| {
                other_c < other_q && crate::autoplay::can_train_unit(&tech, other_u)
            }) {
                continue;
            }

            let (mut min_cost, mut en_cost) = engine.config.get_unit_cost(u_type);
            if tech.hive_proliferation {
                min_cost *= 0.65;
                en_cost *= 0.65;
            }

            let eco = engine.team_eco_mut(team);
            if eco.minerals >= mineral_reserve + min_cost && eco.energy >= en_cost {
                eco.minerals -= min_cost;
                eco.energy -= en_cost;
                if let Some(b) = engine.buildings.iter_mut().find(|b| b.id == bid) {
                    b.training_queue.push(u_type);
                    recruited = true;
                    break;
                }
            }
        }

        if !recruited {
            let basic_units = [UnitType::ShockTrooper, UnitType::Rifleman];
            for &u_type in &basic_units {
                let (mut min_cost, _) = engine.config.get_unit_cost(u_type);
                if tech.hive_proliferation {
                    min_cost *= 0.65;
                }
                let eco = engine.team_eco_mut(team);
                if eco.minerals >= mineral_reserve + min_cost {
                    eco.minerals -= min_cost;
                    if let Some(b) = engine.buildings.iter_mut().find(|b| b.id == bid) {
                        b.training_queue.push(u_type);
                        break;
                    }
                }
            }
        }
    }
}

pub fn execute_team_eco(engine: &mut GameEngine, team: Team, try_expansion: bool) {
    let team_workers = engine.units.iter().filter(|u| u.team == team && u.unit_type == UnitType::Engineer).count();
    let team_ccs = engine.buildings.iter().filter(|b| b.team == team && b.building_type == BuildingType::CommandCenter).count();

    let (supply, max_supply, minerals) = {
        let eco = engine.team_eco(team);
        (eco.supply, eco.max_supply, eco.minerals)
    };

    if team_workers < 16 && supply < max_supply {
        let (w_cost, _) = engine.config.get_unit_cost(UnitType::Engineer);
        if minerals >= w_cost {
            let cc_id = engine.buildings.iter().find(|b| {
                b.team == team && b.building_type == BuildingType::CommandCenter && b.build_progress >= 1.0 && b.training_queue.len() < 2
            }).map(|b| b.id);
            if let Some(bid) = cc_id {
                let eco = engine.team_eco_mut(team);
                eco.minerals -= w_cost;
                if let Some(cc) = engine.buildings.iter_mut().find(|b| b.id == bid) {
                    cc.training_queue.push(UnitType::Engineer);
                }
            }
        }
    }

    if try_expansion && team_ccs < 2 && minerals >= 400.0 {
        if let Some((bx, by)) = crate::autoplay::find_team_build_spot(engine, team, BuildingType::CommandCenter) {
            crate::autoplay::try_team_auto_build(engine, team, BuildingType::CommandCenter, bx, by);
        }
    }
}

pub fn execute_team_infrastructure(engine: &mut GameEngine, team: Team) {
    let reactors = engine.buildings.iter().filter(|b| b.team == team && b.building_type == BuildingType::PowerReactor).count();
    let converters = engine.buildings.iter().filter(|b| b.team == team && b.building_type == BuildingType::EnergyConverter).count();
    let barracks = engine.buildings.iter().filter(|b| b.team == team && b.building_type == BuildingType::Barracks).count();

    let (minerals, energy) = {
        let eco = engine.team_eco(team);
        (eco.minerals, eco.energy)
    };

    if barracks < 4 {
        let (barracks_cost, _) = engine.config.get_building_cost(BuildingType::Barracks);
        if minerals >= barracks_cost {
            let under_construction = engine.buildings.iter().any(|b| {
                b.team == team && b.building_type == BuildingType::Barracks && b.build_progress < 1.0
            });
            if !under_construction {
                if let Some((bx, by)) = crate::autoplay::find_team_build_spot(engine, team, BuildingType::Barracks) {
                    crate::autoplay::try_team_auto_build(engine, team, BuildingType::Barracks, bx, by);
                }
            }
        }
    }

    if reactors < 36 {
        let (reactor_cost, _) = engine.config.get_building_cost(BuildingType::PowerReactor);
        if minerals >= reactor_cost {
            let under_construction = engine.buildings.iter().filter(|b| {
                b.team == team && b.building_type == BuildingType::PowerReactor && b.build_progress < 1.0
            }).count();
            let max_parallel = if minerals >= 300.0 { 2 } else { 1 };
            if under_construction < max_parallel {
                if let Some((bx, by)) = crate::autoplay::find_team_build_spot(engine, team, BuildingType::PowerReactor) {
                    crate::autoplay::try_team_auto_build(engine, team, BuildingType::PowerReactor, bx, by);
                }
            }
        }
    }

    if reactors >= 2 && converters < (reactors / 2).clamp(1, 16) {
        let (conv_m, conv_e) = engine.config.get_building_cost(BuildingType::EnergyConverter);
        if minerals >= conv_m && energy >= conv_e {
            let under_construction = engine.buildings.iter().filter(|b| {
                b.team == team && b.building_type == BuildingType::EnergyConverter && b.build_progress < 1.0
            }).count();
            let max_parallel = if minerals >= 350.0 && energy >= 200.0 { 2 } else { 1 };
            if under_construction < max_parallel {
                if let Some((bx, by)) = crate::autoplay::find_team_build_spot(engine, team, BuildingType::EnergyConverter) {
                    crate::autoplay::try_team_auto_build(engine, team, BuildingType::EnergyConverter, bx, by);
                }
            }
        }
    }
}

pub fn execute_team_defense(engine: &mut GameEngine, team: Team) {
    let turrets = engine.buildings.iter().filter(|b| b.team == team && b.building_type == BuildingType::TurretTower).count();
    let (minerals, energy) = {
        let eco = engine.team_eco(team);
        (eco.minerals, eco.energy)
    };
    if turrets < 6 {
        let (turret_m, turret_e) = engine.config.get_building_cost(BuildingType::TurretTower);
        if minerals >= turret_m && energy >= turret_e {
            if let Some((bx, by)) = crate::autoplay::find_team_build_spot(engine, team, BuildingType::TurretTower) {
                crate::autoplay::try_team_auto_build(engine, team, BuildingType::TurretTower, bx, by);
            }
        }
    }
}

pub fn try_team_research(engine: &mut GameEngine, team: Team, tech_chain: &[&str]) {
    for &tech_id in tech_chain {
        let (needs_research, target) = {
            let team_tech = engine.team_tech(team);
            if !crate::simulation::is_tech_unlocked(team_tech, tech_id)
                && !team_tech.active_researches.iter().any(|r| r.tech_id == tech_id)
            {
                let mut t = tech_id.to_string();
                if let Some(req) = crate::autoplay::get_tech_prerequisite(tech_id) {
                    if !crate::simulation::is_tech_unlocked(team_tech, &req)
                        && !team_tech.active_researches.iter().any(|r| r.tech_id == req)
                    {
                        t = req;
                    }
                }
                (true, t)
            } else {
                (false, String::new())
            }
        };

        if needs_research {
            let (tech, eco, cfg) = match team {
                Team::Player => (&mut engine.player_tech, &mut engine.player_eco, &engine.config),
                Team::Enemy => (&mut engine.enemy_tech, &mut engine.enemy_eco, &engine.config),
                _ => {
                    let idx = team.index() - 2;
                    (&mut engine.extra_techs[idx], &mut engine.extra_ecos[idx], &engine.config)
                }
            };
            if crate::simulation::process_start_research(tech, eco, cfg, &target) {
                return;
            }
        }
    }

    // Nếu đã hoàn thành chuỗi chỉ định, tự động nâng tiếp các công nghệ còn lại
    for &all_t in &crate::autoplay::ALL_TECH_IDS {
        let (needs_research, target) = {
            let team_tech = engine.team_tech(team);
            if !crate::simulation::is_tech_unlocked(team_tech, all_t)
                && !team_tech.active_researches.iter().any(|r| r.tech_id == all_t)
            {
                let mut t = all_t.to_string();
                if let Some(req) = crate::autoplay::get_tech_prerequisite(all_t) {
                    if !crate::simulation::is_tech_unlocked(team_tech, &req)
                        && !team_tech.active_researches.iter().any(|r| r.tech_id == req)
                    {
                        t = req;
                    }
                }
                (true, t)
            } else {
                (false, String::new())
            }
        };

        if needs_research {
            let (tech, eco, cfg) = match team {
                Team::Player => (&mut engine.player_tech, &mut engine.player_eco, &engine.config),
                Team::Enemy => (&mut engine.enemy_tech, &mut engine.enemy_eco, &engine.config),
                _ => {
                    let idx = team.index() - 2;
                    (&mut engine.extra_techs[idx], &mut engine.extra_ecos[idx], &engine.config)
                }
            };
            if crate::simulation::process_start_research(tech, eco, cfg, &target) {
                return;
            }
        }
    }
}

pub fn execute_team_training(engine: &mut GameEngine, team: Team, candidate_units: &[UnitType]) {
    let (supply, max_supply) = {
        let eco = engine.team_eco(team);
        (eco.supply, eco.max_supply)
    };
    if supply >= max_supply {
        return;
    }

    let completed_barracks: Vec<u32> = engine.buildings.iter()
        .filter(|b| b.team == team && b.building_type == BuildingType::Barracks && b.build_progress >= 1.0 && b.training_queue.len() < 2)
        .map(|b| b.id)
        .collect();

    for bid in completed_barracks {
        let (cur_supply, cur_max_supply) = {
            let eco = engine.team_eco(team);
            (eco.supply, eco.max_supply)
        };
        if cur_supply >= cur_max_supply {
            break;
        }

        for &u_type in candidate_units {
            let tech = engine.team_tech(team);
            if u_type == UnitType::Engineer || !crate::autoplay::can_train_unit(tech, u_type) {
                continue;
            }

            let (mut min_cost, mut en_cost) = engine.config.get_unit_cost(u_type);
            if tech.hive_proliferation {
                min_cost *= 0.65;
                en_cost *= 0.65;
            }

            let eco = engine.team_eco_mut(team);
            if eco.minerals >= min_cost && eco.energy >= en_cost {
                eco.minerals -= min_cost;
                eco.energy -= en_cost;
                if let Some(b) = engine.buildings.iter_mut().find(|b| b.id == bid) {
                    b.training_queue.push(u_type);
                    break;
                }
            }
        }
    }
}

pub enum MilitaryCommand {
    DefendBase,
    HarassWorkers,
    HarassOrAttack,
    AllOutAssault,
    Regroup,
    SiegePush,
    SupportAlly((f32, f32)),
    AttackSpecific((f32, f32)),
}

pub fn command_team_military(engine: &mut GameEngine, team: Team, cmd: MilitaryCommand) {
    let opp_team = find_primary_hostile_team(engine, team);

    let combat_pairs: Vec<(u32, UnitType)> = engine.units.iter()
        .filter(|u| u.team == team && u.unit_type != UnitType::Engineer)
        .map(|u| (u.id, u.unit_type))
        .collect();

    if combat_pairs.is_empty() {
        return;
    }

    let base_pos = engine.base_pos(team);
    let default_opp_base = engine.base_pos(opp_team);
    let default_opp_worker_pos = (default_opp_base.0 + 50.0, default_opp_base.1 + 50.0);

    let center = (crate::map::MAP_WIDTH * 0.5, crate::map::MAP_HEIGHT * 0.5);
    let dx = center.0 - base_pos.0;
    let dy = center.1 - base_pos.1;
    let len = (dx * dx + dy * dy).sqrt().max(1.0);
    let default_rally = (base_pos.0 + dx / len * 350.0, base_pos.1 + dy / len * 350.0);

    let form = engine.team_formation(team);

    let start_center = {
        let mut sum_x = 0.0;
        let mut sum_y = 0.0;
        for (uid, _) in &combat_pairs {
            if let Some(u) = engine.units.iter().find(|u| u.id == *uid) {
                sum_x += u.x;
                sum_y += u.y;
            }
        }
        (sum_x / combat_pairs.len() as f32, sum_y / combat_pairs.len() as f32)
    };

    match cmd {
        MilitaryCommand::DefendBase => {
            let nearest_threat = engine.units.iter()
                .filter(|u| engine.are_hostile(team, u.team))
                .min_by(|a, b| {
                    let da = (a.x - base_pos.0).powi(2) + (a.y - base_pos.1).powi(2);
                    let db = (b.x - base_pos.0).powi(2) + (b.y - base_pos.1).powi(2);
                    da.partial_cmp(&db).unwrap()
                })
                .map(|u| (u.x, u.y))
                .unwrap_or(base_pos);

            let form_pos = crate::formation::compute_formation_positions(
                &combat_pairs,
                start_center,
                nearest_threat,
                form,
            );

            for (uid, tx, ty, off_x, off_y) in form_pos {
                if let Some(u) = engine.units.iter_mut().find(|u| u.id == uid) {
                    u.formation_offset = (off_x, off_y);
                    u.order = UnitOrder::AttackMove { x: tx, y: ty };
                }
            }
        }
        MilitaryCommand::HarassWorkers => {
            let opp_worker_pos = engine.units.iter()
                .find(|u| engine.are_hostile(team, u.team) && u.unit_type == UnitType::Engineer)
                .map(|u| (u.x, u.y))
                .unwrap_or(default_opp_worker_pos);

            let form_pos = crate::formation::compute_formation_positions(
                &combat_pairs,
                start_center,
                opp_worker_pos,
                form,
            );

            for (uid, tx, ty, off_x, off_y) in form_pos {
                if let Some(u) = engine.units.iter_mut().find(|u| u.id == uid) {
                    u.formation_offset = (off_x, off_y);
                    u.order = UnitOrder::AttackMove { x: tx, y: ty };
                }
            }
        }
        MilitaryCommand::HarassOrAttack | MilitaryCommand::AllOutAssault => {
            let target = engine.buildings.iter()
                .find(|b| engine.are_hostile(team, b.team) && b.building_type == BuildingType::CommandCenter)
                .map(|b| (b.x, b.y))
                .unwrap_or(default_opp_base);

            let form_pos = crate::formation::compute_formation_positions(
                &combat_pairs,
                start_center,
                target,
                form,
            );

            for (uid, tx, ty, off_x, off_y) in form_pos {
                if let Some(u) = engine.units.iter_mut().find(|u| u.id == uid) {
                    if let UnitOrder::AttackTarget { .. } = u.order {
                        continue;
                    }
                    if let UnitOrder::AttackMove { x, y } = u.order {
                        let d_sq = (x - target.0).powi(2) + (y - target.1).powi(2);
                        if d_sq < 90.0_f32.powi(2) {
                            continue;
                        }
                    }

                    u.formation_offset = (off_x, off_y);
                    u.order = UnitOrder::AttackMove { x: tx, y: ty };
                }
            }
        }
        MilitaryCommand::SiegePush => {
            // Nhắm thẳng vào tháp pháo hoặc nhà chính đối phương
            let target = engine.buildings.iter()
                .find(|b| engine.are_hostile(team, b.team) && b.building_type == BuildingType::TurretTower)
                .map(|b| (b.x, b.y))
                .or_else(|| {
                    engine.buildings.iter()
                        .find(|b| engine.are_hostile(team, b.team) && b.building_type == BuildingType::CommandCenter)
                        .map(|b| (b.x, b.y))
                })
                .unwrap_or(default_opp_base);

            let form_pos = crate::formation::compute_formation_positions(
                &combat_pairs,
                start_center,
                target,
                FormationType::Wedge,
            );

            for (uid, tx, ty, off_x, off_y) in form_pos {
                if let Some(u) = engine.units.iter_mut().find(|u| u.id == uid) {
                    u.formation_offset = (off_x, off_y);
                    u.order = UnitOrder::AttackMove { x: tx, y: ty };
                }
            }
        }
        MilitaryCommand::SupportAlly(target) | MilitaryCommand::AttackSpecific(target) => {
            let form_pos = crate::formation::compute_formation_positions(
                &combat_pairs,
                start_center,
                target,
                form,
            );

            for (uid, tx, ty, off_x, off_y) in form_pos {
                if let Some(u) = engine.units.iter_mut().find(|u| u.id == uid) {
                    u.formation_offset = (off_x, off_y);
                    u.order = UnitOrder::AttackMove { x: tx, y: ty };
                }
            }
        }
        MilitaryCommand::Regroup => {
            let form_pos = crate::formation::compute_formation_positions(
                &combat_pairs,
                start_center,
                default_rally,
                form,
            );

            for (uid, tx, ty, off_x, off_y) in form_pos {
                if let Some(u) = engine.units.iter_mut().find(|u| u.id == uid) {
                    if let UnitOrder::AttackTarget { .. } = u.order {
                        continue;
                    }
                    if let UnitOrder::MoveTo { x, y } = u.order {
                        let d_sq = (x - default_rally.0).powi(2) + (y - default_rally.1).powi(2);
                        if d_sq < 100.0_f32.powi(2) {
                            continue;
                        }
                    }

                    u.formation_offset = (off_x, off_y);
                    u.order = UnitOrder::MoveTo { x: tx, y: ty };
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ai_state_default() {
        let state = AiState::default();
        assert_eq!(state.current_action, BotStrategicAction::ExpandEco);
    }

    #[test]
    fn test_state_extraction_dimension() {
        let engine = GameEngine::new_headless();
        let s_enemy = extract_state_vector(&engine);
        assert_eq!(s_enemy.len(), STATE_DIM);

        let s_player = extract_state_vector_for_team(&engine, Team::Player);
        assert_eq!(s_player.len(), STATE_DIM);
    }

    #[test]
    fn test_strategic_action_mapping() {
        for idx in 0..15 {
            let action = BotStrategicAction::from_index(idx);
            assert_eq!(action.to_index(), idx);
        }
    }
}
