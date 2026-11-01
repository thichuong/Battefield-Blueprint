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

pub fn extract_state_vector(engine: &GameEngine) -> Vec<f32> {
    extract_state_vector_for_team(engine, Team::Enemy)
}

pub fn extract_state_vector_for_team(engine: &GameEngine, team: Team) -> Vec<f32> {
    let mut s = Vec::with_capacity(STATE_DIM);

    let my_eco = engine.team_eco(team);
    let my_tech = engine.team_tech(team);

    let opp_team = find_primary_hostile_team(engine, team);
    let _opp_eco = engine.team_eco(opp_team);
    let opp_tech = engine.team_tech(opp_team);

    let my_base_pos = engine.base_pos(team);
    let _opp_base_pos = engine.base_pos(opp_team);

    // 1. Tiến độ thời gian trận đấu (chuẩn hóa 0-600s)
    let time_norm = (engine.game_time / 600.0).clamp(0.0, 1.0);
    s.push(time_norm);

    // 2. Giai đoạn trận đấu: Early (0.0), Mid (0.5), Late (1.0)
    let game_phase = if engine.game_time < 100.0 {
        0.0
    } else if engine.game_time < 260.0 {
        0.5
    } else {
        1.0
    };
    s.push(game_phase);

    // 3-4. Kinh tế phe mình: Khoáng sản & Năng lượng hiện tại
    s.push((my_eco.minerals / 800.0).clamp(0.0, 2.0));
    s.push((my_eco.energy / 400.0).clamp(0.0, 2.0));

    // 6. Tốc độ thu nhập khoáng sản / s
    let income_rate = (my_eco.minerals_per_sec / 50.0).clamp(0.0, 2.0);
    s.push(income_rate);

    // 7. Tỷ lệ Supply
    let supply_ratio = if my_eco.max_supply > 0 {
        (my_eco.supply as f32 / my_eco.max_supply as f32).clamp(0.0, 1.0)
    } else {
        0.0
    };
    s.push(supply_ratio);

    // 8. Chênh lệch Supply so với trần (cảnh báo kẹt supply)
    let supply_headroom = (my_eco.max_supply.saturating_sub(my_eco.supply)) as f32;
    s.push((supply_headroom / 10.0).clamp(0.0, 2.0));

    // 9-10. Số thợ mỏ & Số nhà chính của phe mình
    let my_workers = engine.units.iter().filter(|u| u.team == team && u.unit_type == UnitType::Engineer).count();
    let my_ccs = engine.buildings.iter().filter(|b| b.team == team && b.building_type == BuildingType::CommandCenter).count();
    s.push((my_workers as f32 / 18.0).clamp(0.0, 1.5));
    s.push((my_ccs as f32 / 3.0).clamp(0.0, 1.0));

    // 11-13. Số Reactor, Converter, Barracks, Turret
    let reactors = engine.buildings.iter().filter(|b| b.team == team && b.building_type == BuildingType::PowerReactor).count();
    let converters = engine.buildings.iter().filter(|b| b.team == team && b.building_type == BuildingType::EnergyConverter).count();
    let barracks = engine.buildings.iter().filter(|b| b.team == team && b.building_type == BuildingType::Barracks).count();
    let turrets = engine.buildings.iter().filter(|b| b.team == team && b.building_type == BuildingType::TurretTower).count();
    s.push((reactors as f32 / 10.0).clamp(0.0, 1.5));
    s.push((converters as f32 / 3.0).clamp(0.0, 1.0));
    s.push((barracks as f32 / 6.0).clamp(0.0, 1.5));
    s.push((turrets as f32 / 8.0).clamp(0.0, 1.5));

    // 14-16. Quân sự: Số lính chiến đấu phe mình vs đối thủ (hoặc các phe thù địch)
    let my_combat = engine.units.iter().filter(|u| u.team == team && u.unit_type != UnitType::Engineer).count();
    let opp_combat = engine.units.iter().filter(|u| engine.are_hostile(team, u.team) && u.unit_type != UnitType::Engineer).count();
    s.push((my_combat as f32 / 35.0).clamp(0.0, 2.0));
    s.push((opp_combat as f32 / 35.0).clamp(0.0, 2.0));
    let mil_ratio = ((my_combat as f32 + 1.0) / (opp_combat as f32 + 1.0)).clamp(0.1, 5.0) / 2.5 - 1.0;
    s.push(mil_ratio);

    // 17-18. Số công nghệ đã mở khóa của phe mình vs đối thủ
    let my_unlocked = count_unlocked_techs(my_tech);
    let opp_unlocked = count_unlocked_techs(opp_tech);
    s.push((my_unlocked as f32 / 8.0).clamp(0.0, 1.0));
    s.push((opp_unlocked as f32 / 8.0).clamp(0.0, 1.0));

    // 19. Căn cứ phe mình có đang bị đe dọa trực tiếp (lính đối thủ tiếp cận <700px)
    let base_threat = engine.units.iter().any(|u| {
        engine.are_hostile(team, u.team) && ((u.x - my_base_pos.0).powi(2) + (u.y - my_base_pos.1).powi(2)) < 700.0_f32.powi(2)
    });
    s.push(if base_threat { 1.0 } else { 0.0 });

    // 20. Khoảng cách gần nhất từ cánh quân đối thủ tới căn cứ phe mình
    let nearest_enemy_dist = engine.units.iter()
        .filter(|u| engine.are_hostile(team, u.team))
        .map(|u| ((u.x - my_base_pos.0).powi(2) + (u.y - my_base_pos.1).powi(2)).sqrt())
        .min_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap_or(crate::map::MAP_WIDTH);
    s.push((nearest_enemy_dist / crate::map::MAP_WIDTH).clamp(0.0, 1.0));

    // 21. Thợ mỏ của đối thủ (để quyết định Harass)
    let opp_workers = engine.units.iter().filter(|u| engine.are_hostile(team, u.team) && u.unit_type == UnitType::Engineer).count();
    s.push((opp_workers as f32 / 16.0).clamp(0.0, 1.5));

    // 22. Tình trạng nhà chính của đối thủ
    let opp_ccs = engine.buildings.iter().filter(|b| engine.are_hostile(team, b.team) && b.building_type == BuildingType::CommandCenter).count();
    s.push((opp_ccs as f32 / 2.0).clamp(0.0, 1.0));

    // 23. Mức độ đe dọa sát thương diện rộng AoE của đối phương (Rocket, Warlock, Baneling, Titan, v.v.)
    let opp_aoe_threat = engine.units.iter().filter(|u| {
        engine.are_hostile(team, u.team) && (
            u.unit_type == UnitType::DarkRocketeer
            || u.unit_type == UnitType::VoidWarlock
            || u.unit_type == UnitType::AcidBaneling
            || u.unit_type == UnitType::BehemothTitan
        )
    }).count();
    s.push((opp_aoe_threat as f32 / 8.0).clamp(0.0, 2.0));

    // 24. Đội hình hiện tại của phe mình (chuẩn hóa 0.0 -> 1.0)
    let cur_form = engine.team_formation(team).to_index() as f32 / 5.0;
    s.push(cur_form);

    // Đảm bảo đúng 24 chiều
    if s.len() > STATE_DIM {
        s.truncate(STATE_DIM);
    }
    while s.len() < STATE_DIM {
        s.push(0.0);
    }

    s
}

pub fn count_unlocked_techs(tech: &crate::resources::TechTreeState) -> usize {
    let mut count = 0;
    if tech.focused_laser { count += 1; }
    if tech.energy_shielding { count += 1; }
    if tech.dark_matter_rockets { count += 1; }
    if tech.titan_alloy_plating { count += 1; }
    if tech.celestial_flying_swords { count += 1; }
    if tech.golden_bell_shield { count += 1; }
    if tech.corrosive_curse { count += 1; }
    if tech.void_cataclysm { count += 1; }
    if tech.primeval_scales { count += 1; }
    if tech.behemoth_mutation { count += 1; }
    if tech.acid_banelings { count += 1; }
    if tech.hive_proliferation { count += 1; }
    if tech.optical_drills { count += 1; }
    if tech.overcharged_reactors { count += 1; }
    if tech.fortified_blueprints { count += 1; }
    if tech.magnetic_rounds { count += 1; }
    if tech.exo_boosters { count += 1; }
    if tech.superheated_blades { count += 1; }
    if tech.overload_strike { count += 1; }
    count
}

pub fn update_ai(engine: &mut GameEngine) {
    for i in 1..crate::components::MAX_PLAYERS {
        let team = Team::from_index(i);
        if engine.is_slot_active(team) && !engine.player_slots[i].is_human {
            let mut ai_state = engine.team_ai(team).clone();
            update_ai_for_team(engine, team, &mut ai_state);
            *engine.team_ai_mut(team) = ai_state;
        }
    }
}

pub fn update_ai_for_team(engine: &mut GameEngine, team: Team, ai_state: &mut AiState) {
    // Tự động phân công thợ mỏ rảnh đi đào quặng
    assign_idle_team_workers(engine, team);

    // Chu kỳ ra quyết định RL (mỗi 1.5s in-game)
    ai_state.decision_timer += 0.5;
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

    // Luôn duy trì phản xạ vi mô liên tục (Continuous Micro-Executors)
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
            // Mở rộng kinh tế: Đào mỏ, thêm thợ mỏ từ CommandCenter và mở rộng căn cứ phụ
            execute_team_eco(engine, team, true);
            execute_team_training(engine, team, &[UnitType::ShockTrooper, UnitType::Rifleman]);
        }
        BotStrategicAction::BuildInfrastructure => {
            // Cân bằng hạ tầng: PowerReactor + Converter + SupplyDepot
            execute_team_infrastructure(engine, team);
            execute_team_training(engine, team, &[UnitType::ShockTrooper, UnitType::Rifleman]);
        }
        BotStrategicAction::BuildDefenseGrid => {
            // Xây dựng phòng tuyến tháp pháo và chống nghẽn
            execute_team_defense(engine, team);
            execute_team_training(engine, team, &[UnitType::ShockTrooper, UnitType::Rifleman]);
        }
        BotStrategicAction::TechScienceBranch => {
            // Nghiên cứu nhánh Khoa học & sản xuất quân tương ứng
            try_team_research(engine, team, &["focused_laser", "energy_shielding", "dark_matter_rockets", "titan_alloy_plating"]);
            execute_team_training(engine, team, &[UnitType::LaserMarksman, UnitType::Vanguard, UnitType::DarkRocketeer]);
        }
        BotStrategicAction::TechMagicBranch => {
            // Nghiên cứu nhánh Ma thuật & sản xuất quân tương ứng
            try_team_research(engine, team, &["celestial_flying_swords", "golden_bell_shield", "corrosive_curse", "void_cataclysm"]);
            execute_team_training(engine, team, &[UnitType::TaoistSwordmaster, UnitType::VoidWarlock]);
        }
        BotStrategicAction::TechMonsterBranch => {
            // Nghiên cứu nhánh Quái vật & sản xuất quân tương ứng
            try_team_research(engine, team, &["primeval_scales", "behemoth_mutation", "acid_banelings", "hive_proliferation"]);
            execute_team_training(engine, team, &[UnitType::AcidBaneling, UnitType::BehemothTitan]);
        }
        BotStrategicAction::MilitarySwarmRush => {
            // Tổng lực sản xuất lính cơ bản tốc độ cao để dồn ép
            execute_team_training(engine, team, &[UnitType::ShockTrooper, UnitType::Rifleman]);
            command_team_military(engine, team, MilitaryCommand::HarassOrAttack);
        }
        BotStrategicAction::DefendAndCounter => {
            // Thủ căn cứ và sẵn sàng phản công
            command_team_military(engine, team, MilitaryCommand::DefendBase);
            execute_team_training(engine, team, &[UnitType::ShockTrooper, UnitType::Rifleman]);
        }
        BotStrategicAction::HarassEconomy => {
            // Đột kích thợ mỏ đối thủ
            command_team_military(engine, team, MilitaryCommand::HarassWorkers);
            execute_team_training(engine, team, &[UnitType::ShockTrooper, UnitType::Rifleman]);
        }
        BotStrategicAction::AllOutAssault => {
            // Tổng tiến công tiêu diệt nhà chính đối thủ
            command_team_military(engine, team, MilitaryCommand::AllOutAssault);
            execute_team_training(engine, team, &[UnitType::ShockTrooper, UnitType::Rifleman]);
        }
        BotStrategicAction::RegroupAndConsolidate => {
            // Rút về điểm tập kết củng cố đội hình
            command_team_military(engine, team, MilitaryCommand::Regroup);
            execute_team_training(engine, team, &[UnitType::ShockTrooper, UnitType::Rifleman]);
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
    let eco_minerals = engine.team_eco(team).minerals;
    if reactors < 3 && eco_minerals >= 100.0 {
        let under_construction = engine.buildings.iter().any(|b| {
            b.team == team && b.building_type == BuildingType::PowerReactor && b.build_progress < 1.0
        });
        if !under_construction {
            if let Some((bx, by)) = crate::autoplay::find_team_build_spot(engine, team, BuildingType::PowerReactor) {
                crate::autoplay::try_team_auto_build(engine, team, BuildingType::PowerReactor, bx, by);
            }
        }
    } else if reactors < 14 && eco_minerals >= 180.0 {
        let under_construction = engine.buildings.iter().any(|b| {
            b.team == team && b.building_type == BuildingType::PowerReactor && b.build_progress < 1.0
        });
        if !under_construction {
            if let Some((bx, by)) = crate::autoplay::find_team_build_spot(engine, team, BuildingType::PowerReactor) {
                crate::autoplay::try_team_auto_build(engine, team, BuildingType::PowerReactor, bx, by);
            }
        }
    }

    // 3.5. Xây EnergyConverter
    let converters = engine.buildings.iter().filter(|b| b.team == team && b.building_type == BuildingType::EnergyConverter).count();
    let under_construction_conv = engine.buildings.iter().any(|b| {
        b.team == team && b.building_type == BuildingType::EnergyConverter && b.build_progress < 1.0
    });
    let (eco_min, eco_en) = (engine.team_eco(team).minerals, engine.team_eco(team).energy);
    if !under_construction_conv && eco_min >= 150.0 && eco_en >= 50.0 {
        if converters < (reactors / 2).min(6).max(1) && (reactors >= 2 || eco_en >= 250.0) {
            if let Some((bx, by)) = crate::autoplay::find_team_build_spot(engine, team, BuildingType::EnergyConverter) {
                crate::autoplay::try_team_auto_build(engine, team, BuildingType::EnergyConverter, bx, by);
            }
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

    // 4.8. Chủ động nâng cấp công nghệ khi thừa năng lượng
    let (active_res_len, tech_en) = (engine.team_tech(team).active_researches.len(), engine.team_eco(team).energy);
    if active_res_len < 2 && tech_en >= 100.0 {
        let tech_priorities = [
            "dark_matter_rockets",
            "focused_laser",
            "energy_shielding",
            "overcharged_reactors",
            "titan_alloy_plating",
            "celestial_flying_swords",
            "golden_bell_shield",
            "acid_banelings",
            "hive_proliferation",
        ];
        for &t in &tech_priorities {
            let (team_tech, team_eco, cfg) = match team {
                Team::Player => (&mut engine.player_tech, &mut engine.player_eco, &engine.config),
                Team::Enemy => (&mut engine.enemy_tech, &mut engine.enemy_eco, &engine.config),
                _ => {
                    let idx = team.index() - 2;
                    (&mut engine.extra_techs[idx], &mut engine.extra_ecos[idx], &engine.config)
                }
            };
            if !crate::simulation::is_tech_unlocked(team_tech, t)
                && !team_tech.active_researches.iter().any(|r| r.tech_id == t)
            {
                if crate::simulation::process_start_research(
                    team_tech,
                    team_eco,
                    cfg,
                    t,
                ) {
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

    let needs_tech = !tech.dark_matter_rockets || !tech.energy_shielding;
    let mineral_reserve = if needs_tech && team_combat >= 6 { 230.0 } else if team_combat >= 6 { 100.0 } else { 0.0 };

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

        let mut recruited = false;
        let priority_units = [
            UnitType::DarkRocketeer,
            UnitType::LaserMarksman,
            UnitType::Vanguard,
            UnitType::BehemothTitan,
            UnitType::VoidWarlock,
            UnitType::TaoistSwordmaster,
            UnitType::AcidBaneling,
        ];

        let tech = engine.team_tech(team).clone();

        for &u_type in &priority_units {
            if !crate::autoplay::can_train_unit(&tech, u_type) {
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

        // Tuyển lính cơ bản
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

    if reactors < 8 {
        let (reactor_cost, _) = engine.config.get_building_cost(BuildingType::PowerReactor);
        if minerals >= reactor_cost {
            let under_construction = engine.buildings.iter().any(|b| {
                b.team == team && b.building_type == BuildingType::PowerReactor && b.build_progress < 1.0
            });
            if !under_construction {
                if let Some((bx, by)) = crate::autoplay::find_team_build_spot(engine, team, BuildingType::PowerReactor) {
                    crate::autoplay::try_team_auto_build(engine, team, BuildingType::PowerReactor, bx, by);
                }
            }
        }
    }

    if reactors >= 3 && converters < (reactors / 3) {
        let (conv_m, conv_e) = engine.config.get_building_cost(BuildingType::EnergyConverter);
        if minerals >= conv_m && energy >= conv_e {
            let under_construction = engine.buildings.iter().any(|b| {
                b.team == team && b.building_type == BuildingType::EnergyConverter && b.build_progress < 1.0
            });
            if !under_construction {
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
        let (tech, eco, cfg) = match team {
            Team::Player => (&mut engine.player_tech, &mut engine.player_eco, &engine.config),
            Team::Enemy => (&mut engine.enemy_tech, &mut engine.enemy_eco, &engine.config),
            _ => {
                let idx = team.index() - 2;
                (&mut engine.extra_techs[idx], &mut engine.extra_ecos[idx], &engine.config)
            }
        };
        if !crate::simulation::is_tech_unlocked(tech, tech_id)
            && !tech.active_researches.iter().any(|r| r.tech_id == tech_id)
        {
            crate::simulation::process_start_research(
                tech,
                eco,
                cfg,
                tech_id,
            );
            break;
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
        for idx in 0..11 {
            let action = BotStrategicAction::from_index(idx);
            assert_eq!(action.to_index(), idx);
        }
    }
}
