use super::macro_ops::{
    command_team_military, execute_team_continuous_recruitment, execute_team_defense,
    execute_team_eco, execute_team_infrastructure, execute_team_training, try_team_research,
    MilitaryCommand,
};
use super::reward::AiState;
use super::state::{
    extract_state_vector_for_team, find_primary_hostile_team, find_threatened_ally_base,
    find_weakest_hostile_team,
};
use crate::components::*;
use crate::rl::BotStrategicAction;
use crate::GameEngine;

pub fn update_ai(engine: &mut GameEngine) {
    update_ai_with_dt(engine, 0.5);
}

pub fn update_ai_with_dt(engine: &mut GameEngine, dt: f32) {
    for i in 1..crate::components::MAX_PLAYERS {
        let team = Team::from_index(i);
        if engine.is_slot_active(team)
            && !engine.player_slots[i].is_human
            && !engine.team_autoplay(team).config.enabled
        {
            let mut ai_state = engine.team_ai(team).clone();
            update_ai_for_team_with_dt(engine, team, &mut ai_state, dt);
            *engine.team_ai_mut(team) = ai_state;
        }
    }
}

pub fn update_ai_for_team(engine: &mut GameEngine, team: Team, ai_state: &mut AiState) {
    update_ai_for_team_with_dt(engine, team, ai_state, 0.5);
}

pub fn update_ai_for_team_with_dt(
    engine: &mut GameEngine,
    team: Team,
    ai_state: &mut AiState,
    dt: f32,
) {
    // Tự động phân công thợ mỏ rảnh đi đào quặng
    assign_idle_team_workers(engine, team);

    // Chu kỳ ra quyết định RL (mỗi 1.5s in-game)
    ai_state.decision_timer += dt;
    if ai_state.decision_timer >= 1.5 {
        ai_state.decision_timer = 0.0;

        let state = extract_state_vector_for_team(engine, team);
        let action = ai_state
            .rl_brain
            .select_action(&state, ai_state.training_enabled);
        let formation = ai_state
            .rl_brain
            .select_formation(&state, ai_state.training_enabled);

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
        .filter(|u| {
            u.team == team && u.unit_type == UnitType::Engineer && u.order == UnitOrder::Idle
        })
        .map(|u| u.id)
        .collect();

    for wid in team_workers {
        if let Some(w) = engine.units.iter_mut().find(|u| u.id == wid) {
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

pub fn apply_strategic_action(engine: &mut GameEngine, action: BotStrategicAction) {
    apply_strategic_action_for_team(engine, Team::Enemy, action);
}

pub fn apply_strategic_action_for_team(
    engine: &mut GameEngine,
    team: Team,
    action: BotStrategicAction,
) {
    // 1. Kiểm tra quân số chiến đấu hiện tại
    let my_combat = engine
        .units
        .iter()
        .filter(|u| u.team == team && u.unit_type != UnitType::Engineer)
        .count();
    let tech = engine.team_tech(team);
    let has_endgame_tech =
        tech.behemoth_mutation || tech.void_cataclysm || tech.dark_matter_rockets;
    let wave_assault_threshold = if has_endgame_tech { 20 } else { 24 };

    // Kiểm tra căn cứ có bị đe dọa trực tiếp không
    let base_pos = engine.base_pos(team);
    let base_threat = engine.units.iter().any(|u| {
        engine.are_hostile(team, u.team)
            && ((u.x - base_pos.0).powi(2) + (u.y - base_pos.1).powi(2)) < 800.0_f32.powi(2)
    });

    match action {
        BotStrategicAction::ExpandEco => {
            execute_team_eco(engine, team, true);
            // Không spam lính cơ bản khi mở rộng kinh tế, ưu tiên dồn tiền cho CC & Thợ mỏ
        }
        BotStrategicAction::BuildInfrastructure => {
            execute_team_infrastructure(engine, team);
            // Không spam lính cơ bản khi xây hạ tầng (Reactors/Converters)
        }
        BotStrategicAction::BuildDefenseGrid => {
            execute_team_defense(engine, team);
            // Không spam lính cơ bản khi xây tháp canh phòng thủ
        }
        BotStrategicAction::TechScienceBranch => {
            try_team_research(
                engine,
                team,
                &[
                    "focused_laser",
                    "energy_shielding",
                    "dark_matter_rockets",
                    "titan_alloy_plating",
                ],
            );
            execute_team_training(
                engine,
                team,
                &[
                    UnitType::LaserMarksman,
                    UnitType::DarkRocketeer,
                    UnitType::BehemothTitan,
                    UnitType::Vanguard,
                    UnitType::ShockTrooper,
                ],
            );
        }
        BotStrategicAction::TechMagicBranch => {
            try_team_research(
                engine,
                team,
                &[
                    "celestial_flying_swords",
                    "golden_bell_shield",
                    "corrosive_curse",
                    "void_cataclysm",
                ],
            );
            execute_team_training(
                engine,
                team,
                &[
                    UnitType::VoidWarlock,
                    UnitType::TaoistSwordmaster,
                    UnitType::DarkRocketeer,
                    UnitType::Rifleman,
                ],
            );
        }
        BotStrategicAction::TechMonsterBranch => {
            try_team_research(
                engine,
                team,
                &[
                    "primeval_scales",
                    "behemoth_mutation",
                    "hive_proliferation",
                    "acid_banelings",
                ],
            );
            execute_team_training(
                engine,
                team,
                &[
                    UnitType::BehemothTitan,
                    UnitType::VoidWarlock,
                    UnitType::LaserMarksman,
                    UnitType::Vanguard,
                    UnitType::ShockTrooper,
                    UnitType::AcidBaneling,
                ],
            );
        }
        BotStrategicAction::MilitarySwarmRush => {
            execute_team_training(
                engine,
                team,
                &[
                    UnitType::ShockTrooper,
                    UnitType::Rifleman,
                    UnitType::DarkRocketeer,
                    UnitType::Vanguard,
                    UnitType::AcidBaneling,
                ],
            );
            if my_combat >= wave_assault_threshold || base_threat {
                command_team_military(engine, team, MilitaryCommand::HarassOrAttack);
            } else {
                command_team_military(engine, team, MilitaryCommand::Regroup);
            }
        }
        BotStrategicAction::DefendAndCounter => {
            command_team_military(engine, team, MilitaryCommand::DefendBase);
            execute_team_training(
                engine,
                team,
                &[
                    UnitType::BehemothTitan,
                    UnitType::Vanguard,
                    UnitType::LaserMarksman,
                    UnitType::Rifleman,
                ],
            );
        }
        BotStrategicAction::HarassEconomy => {
            execute_team_training(
                engine,
                team,
                &[
                    UnitType::TaoistSwordmaster,
                    UnitType::ShockTrooper,
                    UnitType::LaserMarksman,
                    UnitType::AcidBaneling,
                ],
            );
            if my_combat >= wave_assault_threshold || base_threat {
                command_team_military(engine, team, MilitaryCommand::HarassWorkers);
            } else {
                command_team_military(engine, team, MilitaryCommand::Regroup);
            }
        }
        BotStrategicAction::AllOutAssault => {
            execute_team_training(
                engine,
                team,
                &[
                    UnitType::BehemothTitan,
                    UnitType::DarkRocketeer,
                    UnitType::VoidWarlock,
                    UnitType::LaserMarksman,
                    UnitType::ShockTrooper,
                ],
            );
            if my_combat >= wave_assault_threshold || base_threat {
                command_team_military(engine, team, MilitaryCommand::AllOutAssault);
            } else {
                command_team_military(engine, team, MilitaryCommand::Regroup);
            }
        }
        BotStrategicAction::RegroupAndConsolidate => {
            command_team_military(engine, team, MilitaryCommand::Regroup);
            execute_team_training(
                engine,
                team,
                &[
                    UnitType::BehemothTitan,
                    UnitType::Vanguard,
                    UnitType::DarkRocketeer,
                    UnitType::Rifleman,
                ],
            );
        }
        BotStrategicAction::TechLogisticsBranch => {
            try_team_research(
                engine,
                team,
                &[
                    "optical_drills",
                    "overcharged_reactors",
                    "fortified_blueprints",
                    "magnetic_rounds",
                    "exo_boosters",
                    "superheated_blades",
                    "overload_strike",
                ],
            );
            execute_team_training(
                engine,
                team,
                &[
                    UnitType::Rifleman,
                    UnitType::ShockTrooper,
                    UnitType::DarkRocketeer,
                    UnitType::Vanguard,
                ],
            );
        }
        BotStrategicAction::SiegeArtilleryPush => {
            execute_team_training(
                engine,
                team,
                &[
                    UnitType::BehemothTitan,
                    UnitType::DarkRocketeer,
                    UnitType::LaserMarksman,
                    UnitType::Vanguard,
                ],
            );
            if my_combat >= wave_assault_threshold || base_threat {
                command_team_military(engine, team, MilitaryCommand::SiegePush);
            } else {
                command_team_military(engine, team, MilitaryCommand::Regroup);
            }
        }
        BotStrategicAction::CoordinateAllySupport => {
            if let Some(ally_threat_pos) = find_threatened_ally_base(engine, team) {
                command_team_military(engine, team, MilitaryCommand::SupportAlly(ally_threat_pos));
            } else if my_combat >= wave_assault_threshold || base_threat {
                command_team_military(engine, team, MilitaryCommand::AllOutAssault);
            } else {
                command_team_military(engine, team, MilitaryCommand::Regroup);
            }
            execute_team_training(
                engine,
                team,
                &[
                    UnitType::BehemothTitan,
                    UnitType::DarkRocketeer,
                    UnitType::VoidWarlock,
                    UnitType::ShockTrooper,
                ],
            );
        }
        BotStrategicAction::FocusWeakestEnemy => {
            execute_team_training(
                engine,
                team,
                &[
                    UnitType::TaoistSwordmaster,
                    UnitType::LaserMarksman,
                    UnitType::DarkRocketeer,
                    UnitType::ShockTrooper,
                    UnitType::AcidBaneling,
                ],
            );
            if my_combat >= wave_assault_threshold || base_threat {
                let weakest = find_weakest_hostile_team(engine, team);
                let weakest_base = engine.base_pos(weakest);
                command_team_military(engine, team, MilitaryCommand::AttackSpecific(weakest_base));
            } else {
                command_team_military(engine, team, MilitaryCommand::Regroup);
            }
        }
    }
}

pub fn maintain_continuous_ai_reflexes(engine: &mut GameEngine) {
    maintain_continuous_ai_reflexes_for_team(engine, Team::Enemy);
}

pub fn maintain_continuous_ai_reflexes_for_team(engine: &mut GameEngine, team: Team) {
    let _opp_team = find_primary_hostile_team(engine, team);
    let base_pos = engine.base_pos(team);

    let team_workers = engine
        .units
        .iter()
        .filter(|u| u.team == team && u.unit_type == UnitType::Engineer)
        .count();
    let team_ccs = engine
        .buildings
        .iter()
        .filter(|b| b.team == team && b.building_type == BuildingType::CommandCenter)
        .count();
    let worker_target = (12 + team_ccs * 6).min(36);

    // 1. Duy trì sản xuất thợ mỏ cơ bản
    let (eco_supply, eco_max_supply, eco_minerals) = {
        let eco = engine.team_eco(team);
        (eco.supply, eco.max_supply, eco.minerals)
    };

    if team_workers < worker_target && eco_supply < eco_max_supply && eco_minerals >= 50.0 {
        let cc_id = engine
            .buildings
            .iter()
            .find(|b| {
                b.team == team
                    && b.building_type == BuildingType::CommandCenter
                    && b.build_progress >= 1.0
                    && b.training_queue.len() < 2
            })
            .map(|b| b.id);
        if let Some(bid) = cc_id {
            engine.team_eco_mut(team).minerals -= 50.0;
            if let Some(cc) = engine.buildings.iter_mut().find(|b| b.id == bid) {
                cc.training_queue.push(UnitType::Engineer);
            }
        }
    }

    // 1.5. Chủ động mở rộng thêm CommandCenter (Khuyến khích tối đa 5 căn cứ)
    let under_construction_ccs = engine
        .buildings
        .iter()
        .filter(|b| {
            b.team == team
                && b.building_type == BuildingType::CommandCenter
                && b.build_progress < 1.0
        })
        .count();

    let reactors = engine
        .buildings
        .iter()
        .filter(|b| b.team == team && b.building_type == BuildingType::PowerReactor)
        .count();
    let eco_minerals = engine.team_eco(team).minerals;

    let wants_cc_expand = match team_ccs {
        0 => true,
        1 => engine.game_time >= 50.0 || reactors >= 3 || eco_minerals >= 380.0,
        2 => engine.game_time >= 150.0 || reactors >= 6 || eco_minerals >= 400.0,
        3 => engine.game_time >= 270.0 || reactors >= 10 || eco_minerals >= 400.0,
        4 => engine.game_time >= 400.0 || reactors >= 14 || eco_minerals >= 400.0,
        _ => false,
    };

    if wants_cc_expand && under_construction_ccs == 0 && eco_minerals >= 400.0 {
        if let Some((bx, by)) =
            crate::autoplay::find_team_build_spot(engine, team, BuildingType::CommandCenter)
        {
            crate::autoplay::try_team_auto_build(engine, team, BuildingType::CommandCenter, bx, by);
        }
    }

    // 2. Chống kẹt dân số (Supply Depot)
    let under_construction_depots = engine
        .buildings
        .iter()
        .filter(|b| {
            b.team == team && b.building_type == BuildingType::SupplyDepot && b.build_progress < 1.0
        })
        .count();

    let (eco_supply, eco_max_supply, eco_minerals) = {
        let eco = engine.team_eco(team);
        (eco.supply, eco.max_supply, eco.minerals)
    };

    let needs_depot = eco_supply + 6 >= eco_max_supply;
    if needs_depot
        && eco_max_supply < 200
        && under_construction_depots == 0
        && eco_minerals >= 100.0
    {
        let (depot_cost, _) = engine.config.get_building_cost(BuildingType::SupplyDepot);
        if eco_minerals >= depot_cost {
            if let Some((bx, by)) =
                crate::autoplay::find_team_build_spot(engine, team, BuildingType::SupplyDepot)
            {
                crate::autoplay::try_team_auto_build(
                    engine,
                    team,
                    BuildingType::SupplyDepot,
                    bx,
                    by,
                );
            }
        }
    }

    // 3. Xây Power Reactor
    let under_construction_reactors = engine
        .buildings
        .iter()
        .filter(|b| {
            b.team == team
                && b.building_type == BuildingType::PowerReactor
                && b.build_progress < 1.0
        })
        .count();
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
            if let Some((bx, by)) =
                crate::autoplay::find_team_build_spot(engine, team, BuildingType::PowerReactor)
            {
                crate::autoplay::try_team_auto_build(
                    engine,
                    team,
                    BuildingType::PowerReactor,
                    bx,
                    by,
                );
            }
        }
    }

    // 3.5. Xây EnergyConverter (chuyển đổi Năng lượng thành Khoáng sản)
    let converters = engine
        .buildings
        .iter()
        .filter(|b| b.team == team && b.building_type == BuildingType::EnergyConverter)
        .count();
    let under_construction_conv = engine
        .buildings
        .iter()
        .filter(|b| {
            b.team == team
                && b.building_type == BuildingType::EnergyConverter
                && b.build_progress < 1.0
        })
        .count();
    let (eco_min, eco_en) = (engine.team_eco(team).minerals, engine.team_eco(team).energy);

    // Cho phép xây tối đa 16 EnergyConverters (tỷ lệ 1 converter cho 2 reactors, hoặc kích hoạt sớm khi có điện)
    let max_parallel_converters = if eco_min >= 350.0 && eco_en >= 200.0 {
        2
    } else {
        1
    };
    let desired_converters = (reactors / 2).clamp(1, 16);
    let should_build_converter = converters < desired_converters
        && (reactors >= 2 || eco_en >= 200.0)
        && eco_min >= 150.0
        && eco_en >= 50.0;

    if under_construction_conv < max_parallel_converters && should_build_converter {
        if let Some((bx, by)) =
            crate::autoplay::find_team_build_spot(engine, team, BuildingType::EnergyConverter)
        {
            crate::autoplay::try_team_auto_build(
                engine,
                team,
                BuildingType::EnergyConverter,
                bx,
                by,
            );
        }
    }

    // 4. Xây Barracks
    let barracks = engine
        .buildings
        .iter()
        .filter(|b| b.team == team && b.building_type == BuildingType::Barracks)
        .count();
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
            if let Some((bx, by)) =
                crate::autoplay::find_team_build_spot(engine, team, BuildingType::Barracks)
            {
                crate::autoplay::try_team_auto_build(engine, team, BuildingType::Barracks, bx, by);
            }
        }
    }

    // 4.5. Xây TurretTower (Hiệu quả cao trên giá tiền: tăng trần lên 32 tháp, xây song song tới 2 tháp)
    let turrets = engine
        .buildings
        .iter()
        .filter(|b| b.team == team && b.building_type == BuildingType::TurretTower)
        .count();
    let under_construction_turrets = engine
        .buildings
        .iter()
        .filter(|b| {
            b.team == team && b.building_type == BuildingType::TurretTower && b.build_progress < 1.0
        })
        .count();

    let max_turrets = (10 + team_ccs * 6).min(32);
    let max_parallel_turrets = if eco_min >= 280.0 && eco_en >= 100.0 {
        2
    } else {
        1
    };

    if turrets < max_turrets
        && under_construction_turrets < max_parallel_turrets
        && eco_min >= 125.0
        && eco_en >= 50.0
    {
        if let Some((bx, by)) =
            crate::autoplay::find_team_build_spot(engine, team, BuildingType::TurretTower)
        {
            crate::autoplay::try_team_auto_build(engine, team, BuildingType::TurretTower, bx, by);
        }
    }

    // Tự động mở rộng hàng rào phòng thủ của team theo số lượng tháp đã xây
    if turrets >= 12 && engine.team_autoplay(team).config.turret_defense.rows < 3 {
        engine.team_autoplay_mut(team).config.turret_defense.rows = 3;
    } else if turrets >= 5 && engine.team_autoplay(team).config.turret_defense.rows < 2 {
        engine.team_autoplay_mut(team).config.turret_defense.rows = 2;
    }

    // 4.8. Nghiên cứu công nghệ liên tục (Ưu tiên kinh tế & công trình trước khi tới acid baneling)
    let (active_res_len, tech_en) = (
        engine.team_tech(team).active_researches.len(),
        engine.team_eco(team).energy,
    );
    if active_res_len < crate::config::MAX_TOTAL_CONCURRENT_RESEARCH && tech_en >= 80.0 {
        let tech_priorities = [
            "optical_drills",
            "overcharged_reactors",
            "fortified_blueprints",
            "focused_laser",
            "energy_shielding",
            "dark_matter_rockets",
            "primeval_scales",
            "behemoth_mutation",
            "corrosive_curse",
            "void_cataclysm",
            "celestial_flying_swords",
            "golden_bell_shield",
            "overload_strike",
            "titan_alloy_plating",
            "magnetic_rounds",
            "exo_boosters",
            "superheated_blades",
            "hive_proliferation",
            "acid_banelings",
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
                    Team::Player => (
                        &mut engine.player_tech,
                        &mut engine.player_eco,
                        &engine.config,
                    ),
                    Team::Enemy => (
                        &mut engine.enemy_tech,
                        &mut engine.enemy_eco,
                        &engine.config,
                    ),
                    _ => {
                        let idx = team.index() - 2;
                        (
                            &mut engine.extra_techs[idx],
                            &mut engine.extra_ecos[idx],
                            &engine.config,
                        )
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
    let my_combat = engine
        .units
        .iter()
        .filter(|u| u.team == team && u.unit_type != UnitType::Engineer)
        .count();

    // 6.1. Nếu căn cứ phe mình hoặc đồng minh bị đe dọa trực tiếp (<750px), phòng thủ
    let has_threat = engine.units.iter().any(|u| {
        engine.are_hostile(team, u.team)
            && ((u.x - base_pos.0).powi(2) + (u.y - base_pos.1).powi(2)) < 750.0_f32.powi(2)
    });
    if has_threat {
        command_team_military(engine, team, MilitaryCommand::DefendBase);
        return;
    }

    // 6.2. Tiến công: Chỉ phát động tấn công khi đã tập trung thành đợt lớn
    let tech = engine.team_tech(team);
    let has_endgame_tech =
        tech.behemoth_mutation || tech.void_cataclysm || tech.dark_matter_rockets;
    let assault_threshold = if has_endgame_tech { 20 } else { 24 };

    // Chỉ khi đạt quân số đợt lớn mới xuất quân AllOutAssault
    if my_combat >= assault_threshold {
        command_team_military(engine, team, MilitaryCommand::AllOutAssault);
        return;
    } else if my_combat < 10 {
        // Nếu đợt quân tấn công trước đó bị hao hụt chỉ còn dưới 10 con lính, chủ động rút về tập kết
        command_team_military(engine, team, MilitaryCommand::Regroup);
    }

    // 6.3. Tập kết quân mới sinh
    let center = (crate::map::MAP_WIDTH * 0.5, crate::map::MAP_HEIGHT * 0.5);
    let dx = center.0 - base_pos.0;
    let dy = center.1 - base_pos.1;
    let len = (dx * dx + dy * dy).sqrt().max(1.0);
    let rally_pos = (base_pos.0 + dx / len * 400.0, base_pos.1 + dy / len * 400.0);

    let idle_combat_pairs: Vec<(u32, UnitType)> = engine
        .units
        .iter()
        .filter(|u| {
            u.team == team
                && u.unit_type != UnitType::Engineer
                && u.order == UnitOrder::Idle
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
