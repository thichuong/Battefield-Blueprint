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
            decision_timer: 0.0,
            last_state: vec![0.0; STATE_DIM],
            cumulative_reward: 0.0,
            training_enabled: false,
        }
    }
}

pub fn extract_state_vector(engine: &GameEngine) -> Vec<f32> {
    extract_state_vector_for_team(engine, Team::Enemy)
}

pub fn extract_state_vector_for_team(engine: &GameEngine, team: Team) -> Vec<f32> {
    let mut s = Vec::with_capacity(STATE_DIM);

    let (my_eco, _opp_eco, my_tech, opp_tech) = match team {
        Team::Player => (&engine.player_eco, &engine.enemy_eco, &engine.player_tech, &engine.enemy_tech),
        Team::Enemy => (&engine.enemy_eco, &engine.player_eco, &engine.enemy_tech, &engine.player_tech),
    };

    let opp_team = match team {
        Team::Player => Team::Enemy,
        Team::Enemy => Team::Player,
    };

    let (my_base_pos, _opp_base_pos) = match team {
        Team::Player => ((250.0, 2050.0), (2150.0, 350.0)),
        Team::Enemy => ((2150.0, 350.0), (250.0, 2050.0)),
    };

    // 1. Tiến độ thời gian trận đấu (chuẩn hóa 0-600s)
    let time_norm = (engine.game_time / 600.0).clamp(0.0, 1.0);
    s.push(time_norm);

    // 2-3. Giai đoạn: Early (<100s), Mid (100-260s)
    let is_early = if engine.game_time < 100.0 { 1.0 } else { 0.0 };
    let is_mid = if engine.game_time >= 100.0 && engine.game_time < 260.0 { 1.0 } else { 0.0 };
    s.push(is_early);
    s.push(is_mid);

    // 4-5. Kinh tế phe mình: Khoáng sản & Năng lượng hiện tại
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

    // 14-16. Quân sự: Số lính chiến đấu phe mình vs đối thủ
    let my_combat = engine.units.iter().filter(|u| u.team == team && u.unit_type != UnitType::Engineer).count();
    let opp_combat = engine.units.iter().filter(|u| u.team == opp_team && u.unit_type != UnitType::Engineer).count();
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
        u.team == opp_team && ((u.x - my_base_pos.0).powi(2) + (u.y - my_base_pos.1).powi(2)) < 700.0_f32.powi(2)
    });
    s.push(if base_threat { 1.0 } else { 0.0 });

    // 20. Khoảng cách gần nhất từ cánh quân đối thủ tới căn cứ phe mình
    let nearest_enemy_dist = engine.units.iter()
        .filter(|u| u.team == opp_team)
        .map(|u| ((u.x - my_base_pos.0).powi(2) + (u.y - my_base_pos.1).powi(2)).sqrt())
        .min_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap_or(2400.0);
    s.push((nearest_enemy_dist / 2400.0).clamp(0.0, 1.0));

    // 21. Thợ mỏ của đối thủ (để quyết định Harass)
    let opp_workers = engine.units.iter().filter(|u| u.team == opp_team && u.unit_type == UnitType::Engineer).count();
    s.push((opp_workers as f32 / 16.0).clamp(0.0, 1.5));

    // 22. Tình trạng nhà chính của đối thủ
    let opp_ccs = engine.buildings.iter().filter(|b| b.team == opp_team && b.building_type == BuildingType::CommandCenter).count();
    s.push((opp_ccs as f32 / 2.0).clamp(0.0, 1.0));

    // Đảm bảo đủ 24 chiều
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
    let mut ai_state = engine.ai_state.clone();
    update_ai_for_team(engine, Team::Enemy, &mut ai_state);
    engine.ai_state = ai_state;
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
        ai_state.current_action = action;
        ai_state.last_state = state;

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
    let (base_pos, opp_team) = match team {
        Team::Player => ((250.0, 2050.0), Team::Enemy),
        Team::Enemy => ((2150.0, 350.0), Team::Player),
    };

    let team_workers = engine.units.iter().filter(|u| u.team == team && u.unit_type == UnitType::Engineer).count();
    let team_ccs = engine.buildings.iter().filter(|b| b.team == team && b.building_type == BuildingType::CommandCenter).count();
    let worker_target = if team_ccs > 1 { 22 } else { 14 };

    let eco = match team {
        Team::Player => &mut engine.player_eco,
        Team::Enemy => &mut engine.enemy_eco,
    };

    // 1. Duy trì sản xuất thợ mỏ cơ bản
    if team_workers < worker_target && eco.supply < eco.max_supply && eco.minerals >= 50.0 {
        if let Some(cc) = engine.buildings.iter_mut().find(|b| {
            b.team == team && b.building_type == BuildingType::CommandCenter && b.build_progress >= 1.0 && b.training_queue.len() < 2
        }) {
            let eco = match team {
                Team::Player => &mut engine.player_eco,
                Team::Enemy => &mut engine.enemy_eco,
            };
            eco.minerals -= 50.0;
            cc.training_queue.push(UnitType::Engineer);
        }
    }

    // 2. Chống kẹt dân số (Supply Depot)
    let under_construction_depots = engine.buildings.iter().filter(|b| {
        b.team == team && b.building_type == BuildingType::SupplyDepot && b.build_progress < 1.0
    }).count();

    let (eco_supply, eco_max_supply, eco_minerals) = {
        let eco = match team {
            Team::Player => &engine.player_eco,
            Team::Enemy => &engine.enemy_eco,
        };
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
    let eco_minerals = match team {
        Team::Player => engine.player_eco.minerals,
        Team::Enemy => engine.enemy_eco.minerals,
    };
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
    let (eco_min, eco_en) = match team {
        Team::Player => (engine.player_eco.minerals, engine.player_eco.energy),
        Team::Enemy => (engine.enemy_eco.minerals, engine.enemy_eco.energy),
    };
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
    let (active_res_len, tech_en) = match team {
        Team::Player => (engine.player_tech.active_researches.len(), engine.player_eco.energy),
        Team::Enemy => (engine.enemy_tech.active_researches.len(), engine.enemy_eco.energy),
    };
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
            let (team_tech, team_eco) = match team {
                Team::Player => (&mut engine.player_tech, &mut engine.player_eco),
                Team::Enemy => (&mut engine.enemy_tech, &mut engine.enemy_eco),
            };
            if !crate::simulation::is_tech_unlocked(team_tech, t)
                && !team_tech.active_researches.iter().any(|r| r.tech_id == t)
            {
                if crate::simulation::process_start_research(
                    team_tech,
                    team_eco,
                    &engine.config,
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

    // 6.1. Nếu căn cứ bị đe dọa trực tiếp (<750px), phòng thủ
    let has_threat = engine.units.iter().any(|u| {
        u.team == opp_team && ((u.x - base_pos.0).powi(2) + (u.y - base_pos.1).powi(2)) < 750.0_f32.powi(2)
    });
    if has_threat {
        command_team_military(engine, team, MilitaryCommand::DefendBase);
        return;
    }

    // 6.2. Tiến công
    let tech = match team {
        Team::Player => &engine.player_tech,
        Team::Enemy => &engine.enemy_tech,
    };
    let has_siege_tech = tech.dark_matter_rockets || tech.focused_laser;
    let assault_threshold = if has_siege_tech { 16 } else { 22 };

    let is_on_offensive = my_combat >= assault_threshold || engine.units.iter().any(|u| {
        if u.team != team || u.unit_type == UnitType::Engineer {
            false
        } else {
            match team {
                Team::Player => u.x > 1000.0 || u.y < 1300.0,
                Team::Enemy => u.x < 1400.0 || u.y > 1100.0,
            }
        }
    });

    if is_on_offensive {
        command_team_military(engine, team, MilitaryCommand::AllOutAssault);
        return;
    }

    // 6.3. Tập kết quân mới sinh
    let rally_pos = match team {
        Team::Player => (650.0, 1750.0),
        Team::Enemy => (1750.0, 650.0),
    };
    let idle_combat_units: Vec<u32> = engine.units.iter()
        .filter(|u| {
            if u.team != team || u.unit_type == UnitType::Engineer || u.order != UnitOrder::Idle {
                false
            } else {
                match team {
                    Team::Player => u.x < 800.0 && u.y > 1600.0,
                    Team::Enemy => u.x > 1600.0 && u.y < 800.0,
                }
            }
        })
        .map(|u| u.id)
        .collect();

    for (i, uid) in idle_combat_units.iter().enumerate() {
        if let Some(u) = engine.units.iter_mut().find(|u| u.id == *uid) {
            let offset_x = ((i % 5) as f32 - 2.0) * 25.0;
            let offset_y = ((i / 5) as f32) * 25.0;
            u.order = UnitOrder::MoveTo {
                x: rally_pos.0 + offset_x,
                y: rally_pos.1 + offset_y,
            };
        }
    }
}

pub fn execute_team_continuous_recruitment(engine: &mut GameEngine, team: Team) {
    let (supply, max_supply) = match team {
        Team::Player => (engine.player_eco.supply, engine.player_eco.max_supply),
        Team::Enemy => (engine.enemy_eco.supply, engine.enemy_eco.max_supply),
    };
    if supply >= max_supply {
        return;
    }

    let team_combat = engine.units.iter().filter(|u| u.team == team && u.unit_type != UnitType::Engineer).count();
    let (tech, _eco) = match team {
        Team::Player => (&engine.player_tech, &engine.player_eco),
        Team::Enemy => (&engine.enemy_tech, &engine.enemy_eco),
    };

    let needs_tech = !tech.dark_matter_rockets || !tech.energy_shielding;
    let mineral_reserve = if needs_tech && team_combat >= 6 { 230.0 } else if team_combat >= 6 { 100.0 } else { 0.0 };

    let completed_barracks: Vec<u32> = engine.buildings.iter()
        .filter(|b| b.team == team && b.building_type == BuildingType::Barracks && b.build_progress >= 1.0 && b.training_queue.len() < 2)
        .map(|b| b.id)
        .collect();

    for bid in completed_barracks {
        let (cur_supply, cur_max_supply) = match team {
            Team::Player => (engine.player_eco.supply, engine.player_eco.max_supply),
            Team::Enemy => (engine.enemy_eco.supply, engine.enemy_eco.max_supply),
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

        let tech = match team {
            Team::Player => &engine.player_tech,
            Team::Enemy => &engine.enemy_tech,
        };

        for &u_type in &priority_units {
            if !crate::autoplay::can_train_unit(tech, u_type) {
                continue;
            }
            let (mut min_cost, mut en_cost) = engine.config.get_unit_cost(u_type);
            if tech.hive_proliferation {
                min_cost *= 0.65;
                en_cost *= 0.65;
            }

            let eco = match team {
                Team::Player => &mut engine.player_eco,
                Team::Enemy => &mut engine.enemy_eco,
            };

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
                let tech = match team {
                    Team::Player => &engine.player_tech,
                    Team::Enemy => &engine.enemy_tech,
                };
                if tech.hive_proliferation {
                    min_cost *= 0.65;
                }
                let eco = match team {
                    Team::Player => &mut engine.player_eco,
                    Team::Enemy => &mut engine.enemy_eco,
                };
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

    let (supply, max_supply, minerals) = match team {
        Team::Player => (engine.player_eco.supply, engine.player_eco.max_supply, engine.player_eco.minerals),
        Team::Enemy => (engine.enemy_eco.supply, engine.enemy_eco.max_supply, engine.enemy_eco.minerals),
    };

    if team_workers < 16 && supply < max_supply {
        let (w_cost, _) = engine.config.get_unit_cost(UnitType::Engineer);
        if minerals >= w_cost {
            if let Some(cc) = engine.buildings.iter_mut().find(|b| {
                b.team == team && b.building_type == BuildingType::CommandCenter && b.build_progress >= 1.0 && b.training_queue.len() < 2
            }) {
                let eco = match team {
                    Team::Player => &mut engine.player_eco,
                    Team::Enemy => &mut engine.enemy_eco,
                };
                eco.minerals -= w_cost;
                cc.training_queue.push(UnitType::Engineer);
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

    let (minerals, energy) = match team {
        Team::Player => (engine.player_eco.minerals, engine.player_eco.energy),
        Team::Enemy => (engine.enemy_eco.minerals, engine.enemy_eco.energy),
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
    let (minerals, energy) = match team {
        Team::Player => (engine.player_eco.minerals, engine.player_eco.energy),
        Team::Enemy => (engine.enemy_eco.minerals, engine.enemy_eco.energy),
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
        let (tech, eco) = match team {
            Team::Player => (&mut engine.player_tech, &mut engine.player_eco),
            Team::Enemy => (&mut engine.enemy_tech, &mut engine.enemy_eco),
        };
        if !crate::simulation::is_tech_unlocked(tech, tech_id)
            && !tech.active_researches.iter().any(|r| r.tech_id == tech_id)
        {
            crate::simulation::process_start_research(
                tech,
                eco,
                &engine.config,
                tech_id,
            );
            break;
        }
    }
}

pub fn execute_team_training(engine: &mut GameEngine, team: Team, candidate_units: &[UnitType]) {
    let (supply, max_supply) = match team {
        Team::Player => (engine.player_eco.supply, engine.player_eco.max_supply),
        Team::Enemy => (engine.enemy_eco.supply, engine.enemy_eco.max_supply),
    };
    if supply >= max_supply {
        return;
    }

    let completed_barracks: Vec<u32> = engine.buildings.iter()
        .filter(|b| b.team == team && b.building_type == BuildingType::Barracks && b.build_progress >= 1.0 && b.training_queue.len() < 2)
        .map(|b| b.id)
        .collect();

    for bid in completed_barracks {
        let (cur_supply, cur_max_supply) = match team {
            Team::Player => (engine.player_eco.supply, engine.player_eco.max_supply),
            Team::Enemy => (engine.enemy_eco.supply, engine.enemy_eco.max_supply),
        };
        if cur_supply >= cur_max_supply {
            break;
        }

        for &u_type in candidate_units {
            let tech = match team {
                Team::Player => &engine.player_tech,
                Team::Enemy => &engine.enemy_tech,
            };
            if u_type == UnitType::Engineer || !crate::autoplay::can_train_unit(tech, u_type) {
                continue;
            }

            let (mut min_cost, mut en_cost) = engine.config.get_unit_cost(u_type);
            if tech.hive_proliferation {
                min_cost *= 0.65;
                en_cost *= 0.65;
            }

            let eco = match team {
                Team::Player => &mut engine.player_eco,
                Team::Enemy => &mut engine.enemy_eco,
            };

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
    let opp_team = match team {
        Team::Player => Team::Enemy,
        Team::Enemy => Team::Player,
    };

    let combat_unit_ids: Vec<u32> = engine.units.iter()
        .filter(|u| u.team == team && u.unit_type != UnitType::Engineer)
        .map(|u| u.id)
        .collect();

    if combat_unit_ids.is_empty() {
        return;
    }

    let (base_pos, default_opp_base, default_opp_worker_pos, default_rally) = match team {
        Team::Player => ((250.0, 2050.0), (2150.0, 350.0), (2100.0, 450.0), (650.0, 1750.0)),
        Team::Enemy => ((2150.0, 350.0), (250.0, 2050.0), (300.0, 1950.0), (1750.0, 650.0)),
    };

    match cmd {
        MilitaryCommand::DefendBase => {
            let nearest_threat = engine.units.iter()
                .filter(|u| u.team == opp_team)
                .min_by(|a, b| {
                    let da = (a.x - base_pos.0).powi(2) + (a.y - base_pos.1).powi(2);
                    let db = (b.x - base_pos.0).powi(2) + (b.y - base_pos.1).powi(2);
                    da.partial_cmp(&db).unwrap()
                })
                .map(|u| (u.x, u.y))
                .unwrap_or(base_pos);

            for uid in combat_unit_ids {
                if let Some(u) = engine.units.iter_mut().find(|u| u.id == uid) {
                    u.order = UnitOrder::AttackMove { x: nearest_threat.0, y: nearest_threat.1 };
                }
            }
        }
        MilitaryCommand::HarassWorkers => {
            let opp_worker_pos = engine.units.iter()
                .find(|u| u.team == opp_team && u.unit_type == UnitType::Engineer)
                .map(|u| (u.x, u.y))
                .unwrap_or(default_opp_worker_pos);

            for (i, uid) in combat_unit_ids.iter().enumerate() {
                if let Some(u) = engine.units.iter_mut().find(|u| u.id == *uid) {
                    let offset_x = ((i % 4) as f32 - 1.5) * 25.0;
                    let offset_y = ((i / 4) as f32) * 25.0;
                    u.order = UnitOrder::AttackMove {
                        x: opp_worker_pos.0 + offset_x,
                        y: opp_worker_pos.1 + offset_y,
                    };
                }
            }
        }
        MilitaryCommand::HarassOrAttack | MilitaryCommand::AllOutAssault => {
            let target = engine.buildings.iter()
                .find(|b| b.team == opp_team && b.building_type == BuildingType::CommandCenter)
                .map(|b| (b.x, b.y))
                .unwrap_or(default_opp_base);

            for (i, uid) in combat_unit_ids.iter().enumerate() {
                if let Some(u) = engine.units.iter_mut().find(|u| u.id == *uid) {
                    if let UnitOrder::AttackTarget { .. } = u.order {
                        continue;
                    }
                    if let UnitOrder::AttackMove { x, y } = u.order {
                        let d_sq = (x - target.0).powi(2) + (y - target.1).powi(2);
                        if d_sq < 90.0_f32.powi(2) {
                            continue;
                        }
                    }

                    let angle = (i as f32) * 0.45;
                    let radius = 40.0 + ((i % 4) as f32) * 15.0;
                    let target_x = (target.0 + angle.cos() * radius).clamp(60.0, 2340.0);
                    let target_y = (target.1 + angle.sin() * radius).clamp(60.0, 2340.0);

                    u.order = UnitOrder::AttackMove {
                        x: target_x,
                        y: target_y,
                    };
                }
            }
        }
        MilitaryCommand::Regroup => {
            for (i, uid) in combat_unit_ids.iter().enumerate() {
                if let Some(u) = engine.units.iter_mut().find(|u| u.id == *uid) {
                    if let UnitOrder::AttackTarget { .. } = u.order {
                        continue;
                    }
                    if let UnitOrder::MoveTo { x, y } = u.order {
                        let d_sq = (x - default_rally.0).powi(2) + (y - default_rally.1).powi(2);
                        if d_sq < 100.0_f32.powi(2) {
                            continue;
                        }
                    }
                    let offset_x = ((i % 5) as f32 - 2.0) * 25.0;
                    let offset_y = ((i / 5) as f32) * 25.0;
                    u.order = UnitOrder::MoveTo {
                        x: default_rally.0 + offset_x,
                        y: default_rally.1 + offset_y,
                    };
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
