use crate::components::*;
use crate::rl::STATE_DIM;
use crate::GameEngine;

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
    if team == Team::Player {
        Team::Enemy
    } else {
        Team::Player
    }
}

pub fn find_weakest_hostile_team(engine: &GameEngine, team: Team) -> Team {
    let my_faction = engine.get_faction(team);
    let mut best_team = find_primary_hostile_team(engine, team);
    let mut lowest_score = f32::INFINITY;

    for i in 0..crate::components::MAX_PLAYERS {
        let candidate = Team::from_index(i);
        if engine.is_slot_active(candidate) && engine.get_faction(candidate) != my_faction {
            let cc_hp: f32 = engine
                .buildings
                .iter()
                .filter(|b| b.team == candidate && b.building_type == BuildingType::CommandCenter)
                .map(|b| b.hp)
                .sum();
            let combat_count = engine
                .units
                .iter()
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
    pub opp_reactors: usize,
    pub opp_converters: usize,

    pub ally_combat: usize,
    pub nearest_enemy_dist: f32,
    pub base_threat: bool,
    pub active_hostile_factions: usize,

    // Construction & queues (Consolidated Single-Pass)
    pub my_idle_workers: usize,
    pub my_reactors_under_construction: usize,
    pub my_converters_under_construction: usize,
    pub my_barracks_under_construction: usize,
    pub my_turrets_under_construction: usize,
    pub my_barracks_queue_total: usize,

    // Nearby minerals metrics
    pub nearby_minerals_count: usize,
    pub nearby_minerals_amount: f32,
    pub dist_mineral_to_enemy: f32,
}

pub fn create_tactical_snapshot(engine: &GameEngine, team: Team) -> TacticalSnapshot {
    let _my_faction = engine.get_faction(team);
    let my_base = engine.base_pos(team);
    let opp_team = find_primary_hostile_team(engine, team);

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
        opp_reactors: 0,
        opp_converters: 0,

        ally_combat: 0,
        nearest_enemy_dist: crate::map::MAP_WIDTH,
        base_threat: false,
        active_hostile_factions: 0,

        my_idle_workers: 0,
        my_reactors_under_construction: 0,
        my_converters_under_construction: 0,
        my_barracks_under_construction: 0,
        my_turrets_under_construction: 0,
        my_barracks_queue_total: 0,

        nearby_minerals_count: 0,
        nearby_minerals_amount: 0.0,
        dist_mineral_to_enemy: f32::INFINITY,
    };

    // Mặt nạ bit phe phái (Zero Heap Allocation thay thế HashSet)
    let mut hostile_factions_mask: u8 = 0;

    // 1 lượt quét duy nhất toàn bộ Units
    for u in &engine.units {
        let is_hostile = engine.are_hostile(team, u.team);
        if u.team == team {
            if u.unit_type == UnitType::Engineer {
                snap.my_workers += 1;
                if u.order == UnitOrder::Idle {
                    snap.my_idle_workers += 1;
                }
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
                    UnitType::TaoistSwordmaster
                    | UnitType::VoidWarlock
                    | UnitType::AcidBaneling => {
                        snap.my_magic += 1;
                    }
                    _ => {}
                }
            }
        } else if is_hostile {
            let faction = engine.get_faction(u.team);
            hostile_factions_mask |= 1 << (faction.min(7));
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
                if matches!(
                    u.unit_type,
                    UnitType::DarkRocketeer
                        | UnitType::VoidWarlock
                        | UnitType::AcidBaneling
                        | UnitType::BehemothTitan
                ) {
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
        } else if engine.are_allies(team, u.team)
            && u.team != team
            && u.unit_type != UnitType::Engineer
        {
            snap.ally_combat += 1;
        }
    }

    // 1 lượt quét duy nhất toàn bộ Buildings
    for b in &engine.buildings {
        let is_hostile = engine.are_hostile(team, b.team);
        if u_or_b_is_my_team(b.team, team) {
            let is_under_construction = b.build_progress < 1.0;
            if is_under_construction {
                match b.building_type {
                    BuildingType::PowerReactor => snap.my_reactors_under_construction += 1,
                    BuildingType::EnergyConverter => snap.my_converters_under_construction += 1,
                    BuildingType::Barracks => snap.my_barracks_under_construction += 1,
                    BuildingType::TurretTower => snap.my_turrets_under_construction += 1,
                    _ => {}
                }
            }
            match b.building_type {
                BuildingType::CommandCenter => {
                    snap.my_ccs += 1;
                    snap.my_cc_total_hp += b.hp;
                    snap.my_cc_max_hp += b.max_hp;
                }
                BuildingType::PowerReactor => snap.my_reactors += 1,
                BuildingType::EnergyConverter => snap.my_converters += 1,
                BuildingType::Barracks => {
                    snap.my_barracks += 1;
                    if !is_under_construction {
                        snap.my_barracks_queue_total += b.training_queue.len();
                    }
                }
                BuildingType::TurretTower => snap.my_turrets += 1,
                _ => {}
            }
        } else if is_hostile {
            let faction = engine.get_faction(b.team);
            hostile_factions_mask |= 1 << (faction.min(7));
            if b.team == opp_team {
                match b.building_type {
                    BuildingType::PowerReactor => snap.opp_reactors += 1,
                    BuildingType::EnergyConverter => snap.opp_converters += 1,
                    _ => {}
                }
            }
            if b.building_type == BuildingType::CommandCenter {
                snap.opp_ccs += 1;
            } else if b.building_type == BuildingType::TurretTower {
                snap.opp_turrets += 1;
            }
        }
    }

    // 1 lượt quét duy nhất mỏ khoáng sản
    let r_sq_900 = 900.0 * 900.0;
    for m in &engine.minerals {
        if m.amount > 0.0 {
            let d_sq = (m.x - my_base.0).powi(2) + (m.y - my_base.1).powi(2);
            if d_sq < r_sq_900 {
                snap.nearby_minerals_count += 1;
                snap.nearby_minerals_amount += m.amount;
            }
        }
    }

    if snap.nearest_enemy_dist < crate::map::MAP_WIDTH {
        snap.dist_mineral_to_enemy = (snap.nearest_enemy_dist - 300.0).max(0.0);
    }

    snap.active_hostile_factions = hostile_factions_mask.count_ones() as usize;
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
    let mil_ratio =
        ((snap.my_combat as f32 + 1.0) / (snap.opp_combat as f32 + 1.0)).clamp(0.1, 5.0) / 2.5
            - 1.0;
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
    let conversion_potential =
        (snap.my_converters as f32 * crate::config::CONVERT_MINERALS_PER_SEC / 100.0)
            .clamp(0.0, 3.0);
    s.push(conversion_potential);

    // 44. Tỷ lệ cân bằng vàng giữa Converter và Reactor (lý tưởng là 1 Converter cho 2-3 Reactor)
    let conv_per_reactor_ratio = if snap.my_reactors > 0 {
        (snap.my_converters as f32 / (snap.my_reactors as f32 * 0.5)).clamp(0.0, 2.0)
    } else {
        0.0
    };
    s.push(conv_per_reactor_ratio);

    // 45. Trạng thái công nghệ nạp siêu tốc lò phản ứng (overcharged_reactors: +50% sản lượng điện)
    s.push(if my_tech.overcharged_reactors {
        1.0
    } else {
        0.0
    });

    // 46. Trạng thái công nghệ mũi khoan quang học (optical_drills: +50% sức chứa khai thác)
    s.push(if my_tech.optical_drills { 1.0 } else { 0.0 });

    // 47. Trạng thái công nghệ bản vẽ kiên cố (fortified_blueprints: +30% HP công trình)
    s.push(if my_tech.fortified_blueprints {
        1.0
    } else {
        0.0
    });

    // 48. Số lượng PowerReactor của đối thủ
    s.push((snap.opp_reactors as f32 / 20.0).clamp(0.0, 2.0));

    // 49. Số lượng EnergyConverter của đối thủ
    s.push((snap.opp_converters as f32 / 10.0).clamp(0.0, 2.0));

    // 50. Chênh lệch sản lượng năng lượng giữa phe mình và đối thủ
    let opp_eco = engine.team_eco(opp_team);
    let energy_flow_diff =
        ((my_eco.energy_per_sec - opp_eco.energy_per_sec) / 50.0).clamp(-2.0, 2.0);
    s.push(energy_flow_diff);

    // 51. Chênh lệch thu nhập khoáng sản giữa phe mình và đối thủ
    let min_flow_diff =
        ((my_eco.minerals_per_sec - opp_eco.minerals_per_sec) / 50.0).clamp(-2.0, 2.0);
    s.push(min_flow_diff);

    // 52. Số lượng mỏ khoáng còn trữ lượng trong bán kính 900px quanh căn cứ phe mình
    s.push((snap.nearby_minerals_count as f32 / 12.0).clamp(0.0, 1.5));

    // 53. Tổng lượng khoáng sản còn lại quanh căn cứ
    s.push((snap.nearby_minerals_amount / 20000.0).clamp(0.0, 2.0));

    // 54. Đang có PowerReactor đang xây dở không
    s.push((snap.my_reactors_under_construction as f32 / 3.0).clamp(0.0, 1.0));

    // 55. Đang có EnergyConverter đang xây dở không
    s.push((snap.my_converters_under_construction as f32 / 2.0).clamp(0.0, 1.0));

    // 56. Tỷ lệ thợ mỏ đang rảnh rỗi (idle workers)
    s.push((snap.my_idle_workers as f32 / (snap.my_workers.max(1) as f32)).clamp(0.0, 1.0));

    // 57. Số lượng nhà Barracks đang xây dở
    s.push((snap.my_barracks_under_construction as f32 / 3.0).clamp(0.0, 1.0));

    // 58. Tỷ lệ % tải của các hàng đợi sản xuất quân lính
    let barracks_queue_load =
        snap.my_barracks_queue_total as f32 / (snap.my_barracks.max(1) as f32 * 2.0);
    s.push(barracks_queue_load.clamp(0.0, 1.0));

    // 59. Số lượng tháp pháo đang xây dở
    s.push((snap.my_turrets_under_construction as f32 / 4.0).clamp(0.0, 1.0));

    // 60. Khoảng cách an toàn của mỏ khoáng gần nhất so với quân địch
    let norm_safe_dist = if snap.dist_mineral_to_enemy.is_infinite() {
        1.0
    } else {
        (snap.dist_mineral_to_enemy / 2000.0).clamp(0.0, 1.0)
    };
    s.push(norm_safe_dist);

    // 61. Trạng thái công nghệ nhánh Ma thuật cấp cao (void_cataclysm)
    s.push(if my_tech.void_cataclysm { 1.0 } else { 0.0 });

    // 62. Trạng thái công nghệ nhánh Quái vật cấp cao (behemoth_mutation)
    s.push(if my_tech.behemoth_mutation { 1.0 } else { 0.0 });

    // 63. Trạng thái công nghệ nhánh Khoa học cấp cao (dark_matter_rockets)
    s.push(if my_tech.dark_matter_rockets {
        1.0
    } else {
        0.0
    });

    // 64. Trạng thái sẵn sàng tổng lực công kích (Macro Readiness: kinh tế mạnh + quân lực dồi dào)
    let macro_readiness =
        if snap.my_combat >= 20 && snap.my_reactors >= 6 && snap.my_converters >= 2 {
            1.0
        } else {
            (snap.my_combat as f32 / 20.0 * 0.5 + snap.my_reactors as f32 / 6.0 * 0.5)
                .clamp(0.0, 1.0)
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
    if tech.focused_laser {
        sci += 1;
    }
    if tech.energy_shielding {
        sci += 1;
    }
    if tech.dark_matter_rockets {
        sci += 1;
    }
    if tech.titan_alloy_plating {
        sci += 1;
    }

    let mut mag = 0;
    if tech.celestial_flying_swords {
        mag += 1;
    }
    if tech.golden_bell_shield {
        mag += 1;
    }
    if tech.corrosive_curse {
        mag += 1;
    }
    if tech.void_cataclysm {
        mag += 1;
    }

    let mut mon = 0;
    if tech.primeval_scales {
        mon += 1;
    }
    if tech.behemoth_mutation {
        mon += 1;
    }
    if tech.acid_banelings {
        mon += 1;
    }
    if tech.hive_proliferation {
        mon += 1;
    }

    let mut log = 0;
    if tech.optical_drills {
        log += 1;
    }
    if tech.overcharged_reactors {
        log += 1;
    }
    if tech.fortified_blueprints {
        log += 1;
    }
    if tech.magnetic_rounds {
        log += 1;
    }
    if tech.exo_boosters {
        log += 1;
    }
    if tech.superheated_blades {
        log += 1;
    }
    if tech.overload_strike {
        log += 1;
    }

    (sci, mag, mon, log)
}

pub fn count_unlocked_techs(tech: &crate::resources::TechTreeState) -> usize {
    let (sci, mag, mon, log) = count_branch_techs(tech);
    sci + mag + mon + log
}
