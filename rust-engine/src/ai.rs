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
    let mut s = Vec::with_capacity(STATE_DIM);

    // 1. Tiến độ thời gian trận đấu (chuẩn hóa 0-600s)
    let time_norm = (engine.game_time / 600.0).clamp(0.0, 1.0);
    s.push(time_norm);

    // 2-3. Giai đoạn: Early (<100s), Mid (100-260s)
    let is_early = if engine.game_time < 100.0 { 1.0 } else { 0.0 };
    let is_mid = if engine.game_time >= 100.0 && engine.game_time < 260.0 { 1.0 } else { 0.0 };
    s.push(is_early);
    s.push(is_mid);

    // 4-5. Kinh tế Bot: Khoáng sản & Năng lượng hiện tại
    s.push((engine.enemy_eco.minerals / 800.0).clamp(0.0, 2.0));
    s.push((engine.enemy_eco.energy / 400.0).clamp(0.0, 2.0));

    // 6. Tốc độ thu nhập khoáng sản / s (so sánh với Player)
    let income_rate = (engine.enemy_eco.minerals_per_sec / 50.0).clamp(0.0, 2.0);
    s.push(income_rate);

    // 7. Tỷ lệ Supply
    let supply_ratio = if engine.enemy_eco.max_supply > 0 {
        (engine.enemy_eco.supply as f32 / engine.enemy_eco.max_supply as f32).clamp(0.0, 1.0)
    } else {
        0.0
    };
    s.push(supply_ratio);

    // 8. Chênh lệch Supply so với trần (cảnh báo kẹt supply)
    let supply_headroom = (engine.enemy_eco.max_supply.saturating_sub(engine.enemy_eco.supply)) as f32;
    s.push((supply_headroom / 10.0).clamp(0.0, 2.0));

    // 9-10. Số thợ mỏ & Số nhà chính của Bot
    let my_workers = engine.units.iter().filter(|u| u.team == Team::Enemy && u.unit_type == UnitType::Engineer).count();
    let my_ccs = engine.buildings.iter().filter(|b| b.team == Team::Enemy && b.building_type == BuildingType::CommandCenter).count();
    s.push((my_workers as f32 / 18.0).clamp(0.0, 1.5));
    s.push((my_ccs as f32 / 3.0).clamp(0.0, 1.0));

    // 11-13. Số Reactor, Converter, Barracks, Turret
    let reactors = engine.buildings.iter().filter(|b| b.team == Team::Enemy && b.building_type == BuildingType::PowerReactor).count();
    let converters = engine.buildings.iter().filter(|b| b.team == Team::Enemy && b.building_type == BuildingType::EnergyConverter).count();
    let barracks = engine.buildings.iter().filter(|b| b.team == Team::Enemy && b.building_type == BuildingType::Barracks).count();
    let turrets = engine.buildings.iter().filter(|b| b.team == Team::Enemy && b.building_type == BuildingType::TurretTower).count();
    s.push((reactors as f32 / 10.0).clamp(0.0, 1.5));
    s.push((converters as f32 / 3.0).clamp(0.0, 1.0));
    s.push((barracks as f32 / 6.0).clamp(0.0, 1.5));
    s.push((turrets as f32 / 8.0).clamp(0.0, 1.5));

    // 14-16. Quân sự: Số lính chiến đấu phe Bot vs Player
    let enemy_combat = engine.units.iter().filter(|u| u.team == Team::Enemy && u.unit_type != UnitType::Engineer).count();
    let player_combat = engine.units.iter().filter(|u| u.team == Team::Player && u.unit_type != UnitType::Engineer).count();
    s.push((enemy_combat as f32 / 35.0).clamp(0.0, 2.0));
    s.push((player_combat as f32 / 35.0).clamp(0.0, 2.0));
    let mil_ratio = ((enemy_combat as f32 + 1.0) / (player_combat as f32 + 1.0)).clamp(0.1, 5.0) / 2.5 - 1.0;
    s.push(mil_ratio);

    // 17-18. Số công nghệ đã mở khóa của Bot vs Player
    let enemy_techs = count_unlocked_techs(&engine.enemy_tech);
    let player_techs = count_unlocked_techs(&engine.player_tech);
    s.push((enemy_techs as f32 / 8.0).clamp(0.0, 1.0));
    s.push((player_techs as f32 / 8.0).clamp(0.0, 1.0));

    // 19. Căn cứ Bot có đang bị đe dọa trực tiếp (lính Player tiếp cận <700px)
    let bot_base = (2150.0, 350.0);
    let base_threat = engine.units.iter().any(|u| {
        u.team == Team::Player && ((u.x - bot_base.0).powi(2) + (u.y - bot_base.1).powi(2)) < 700.0_f32.powi(2)
    });
    s.push(if base_threat { 1.0 } else { 0.0 });

    // 20. Khoảng cách gần nhất từ cánh quân Player tới căn cứ Bot
    let nearest_enemy_dist = engine.units.iter()
        .filter(|u| u.team == Team::Player)
        .map(|u| ((u.x - bot_base.0).powi(2) + (u.y - bot_base.1).powi(2)).sqrt())
        .min_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap_or(2400.0);
    s.push((nearest_enemy_dist / 2400.0).clamp(0.0, 1.0));

    // 21. Thợ mỏ của Player (thông tin để Bot quyết định Harass)
    let player_workers = engine.units.iter().filter(|u| u.team == Team::Player && u.unit_type == UnitType::Engineer).count();
    s.push((player_workers as f32 / 16.0).clamp(0.0, 1.5));

    // 22. Tình trạng nhà chính Player (còn bao nhiêu CC)
    let player_ccs = engine.buildings.iter().filter(|b| b.team == Team::Player && b.building_type == BuildingType::CommandCenter).count();
    s.push((player_ccs as f32 / 2.0).clamp(0.0, 1.0));

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
    // Tự động phân công thợ mỏ rảnh đi đào quặng
    assign_idle_enemy_workers(engine);

    // Chu kỳ ra quyết định RL (mỗi 1.5s in-game)
    engine.ai_state.decision_timer += 0.5;
    if engine.ai_state.decision_timer >= 1.5 {
        engine.ai_state.decision_timer = 0.0;

        let state = extract_state_vector(engine);
        let action = engine.ai_state.rl_brain.select_action(&state, engine.ai_state.training_enabled);
        engine.ai_state.current_action = action;
        engine.ai_state.last_state = state;

        apply_strategic_action(engine, action);
    }

    // Luôn duy trì phản xạ vi mô liên tục (Continuous Micro-Executors)
    maintain_continuous_ai_reflexes(engine);
}

fn assign_idle_enemy_workers(engine: &mut GameEngine) {
    let available_minerals: Vec<(u32, f32, f32)> = engine
        .minerals
        .iter()
        .filter(|m| m.amount > 0.0)
        .map(|m| (m.id, m.x, m.y))
        .collect();

    if available_minerals.is_empty() {
        return;
    }

    let enemy_workers: Vec<u32> = engine
        .units
        .iter()
        .filter(|u| u.team == Team::Enemy && u.unit_type == UnitType::Engineer && u.order == UnitOrder::Idle)
        .map(|u| u.id)
        .collect();

    for wid in enemy_workers {
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
    match action {
        BotStrategicAction::ExpandEco => {
            // Mở rộng kinh tế: Đào mỏ, thêm thợ mỏ từ CommandCenter và mở rộng căn cứ phụ
            execute_bot_eco(engine, true);
            execute_bot_training(engine, &[UnitType::ShockTrooper, UnitType::Rifleman]);
        }
        BotStrategicAction::BuildInfrastructure => {
            // Cân bằng hạ tầng: PowerReactor + Converter + SupplyDepot
            execute_bot_infrastructure(engine);
            execute_bot_training(engine, &[UnitType::ShockTrooper, UnitType::Rifleman]);
        }
        BotStrategicAction::BuildDefenseGrid => {
            // Xây dựng phòng tuyến tháp pháo và chống nghẽn
            execute_bot_defense(engine);
            execute_bot_training(engine, &[UnitType::ShockTrooper, UnitType::Rifleman]);
        }
        BotStrategicAction::TechScienceBranch => {
            // Nghiên cứu nhánh Khoa học & sản xuất quân tương ứng
            try_bot_research(engine, &["focused_laser", "energy_shielding", "dark_matter_rockets", "titan_alloy_plating"]);
            execute_bot_training(engine, &[UnitType::LaserMarksman, UnitType::Vanguard, UnitType::DarkRocketeer]);
        }
        BotStrategicAction::TechMagicBranch => {
            // Nghiên cứu nhánh Ma thuật & sản xuất quân tương ứng
            try_bot_research(engine, &["celestial_flying_swords", "golden_bell_shield", "corrosive_curse", "void_cataclysm"]);
            execute_bot_training(engine, &[UnitType::TaoistSwordmaster, UnitType::VoidWarlock]);
        }
        BotStrategicAction::TechMonsterBranch => {
            // Nghiên cứu nhánh Quái vật & sản xuất quân tương ứng
            try_bot_research(engine, &["primeval_scales", "behemoth_mutation", "acid_banelings", "hive_proliferation"]);
            execute_bot_training(engine, &[UnitType::AcidBaneling, UnitType::BehemothTitan]);
        }
        BotStrategicAction::MilitarySwarmRush => {
            // Tổng lực sản xuất lính cơ bản tốc độ cao để dồn ép
            execute_bot_training(engine, &[UnitType::ShockTrooper, UnitType::Rifleman]);
            command_bot_military(engine, MilitaryCommand::HarassOrAttack);
        }
        BotStrategicAction::DefendAndCounter => {
            // Thủ căn cứ và sẵn sàng phản công
            command_bot_military(engine, MilitaryCommand::DefendBase);
            execute_bot_training(engine, &[UnitType::ShockTrooper, UnitType::Rifleman]);
        }
        BotStrategicAction::HarassEconomy => {
            // Đột kích thợ mỏ người chơi
            command_bot_military(engine, MilitaryCommand::HarassWorkers);
            execute_bot_training(engine, &[UnitType::ShockTrooper, UnitType::Rifleman]);
        }
        BotStrategicAction::AllOutAssault => {
            // Tổng tiến công tiêu diệt nhà chính Player
            command_bot_military(engine, MilitaryCommand::AllOutAssault);
            execute_bot_training(engine, &[UnitType::ShockTrooper, UnitType::Rifleman]);
        }
        BotStrategicAction::RegroupAndConsolidate => {
            // Rút về điểm tập kết củng cố đội hình
            command_bot_military(engine, MilitaryCommand::Regroup);
            execute_bot_training(engine, &[UnitType::ShockTrooper, UnitType::Rifleman]);
        }
    }
}

fn maintain_continuous_ai_reflexes(engine: &mut GameEngine) {
    let base_pos = (2150.0, 350.0);

    // 1. Duy trì sản xuất thợ mỏ cơ bản
    let enemy_workers = engine.units.iter().filter(|u| u.team == Team::Enemy && u.unit_type == UnitType::Engineer).count();
    let enemy_ccs = engine.buildings.iter().filter(|b| b.team == Team::Enemy && b.building_type == BuildingType::CommandCenter).count();
    let worker_target = if enemy_ccs > 1 { 22 } else { 14 };
    if enemy_workers < worker_target && engine.enemy_eco.supply < engine.enemy_eco.max_supply && engine.enemy_eco.minerals >= 50.0 {
        if let Some(cc) = engine.buildings.iter_mut().find(|b| {
            b.team == Team::Enemy && b.building_type == BuildingType::CommandCenter && b.build_progress >= 1.0 && b.training_queue.len() < 2
        }) {
            engine.enemy_eco.minerals -= 50.0;
            cc.training_queue.push(UnitType::Engineer);
        }
    }

    // 2. Chống kẹt dân số (Supply Depot): Mở rộng trần dân số chủ động lên 200
    let under_construction_depots = engine.buildings.iter().filter(|b| {
        b.team == Team::Enemy && b.building_type == BuildingType::SupplyDepot && b.build_progress < 1.0
    }).count();
    let needs_depot = engine.enemy_eco.supply + 6 >= engine.enemy_eco.max_supply;
    if needs_depot && engine.enemy_eco.max_supply < 200 && under_construction_depots == 0 && engine.enemy_eco.minerals >= 100.0 {
        let (depot_cost, _) = engine.config.get_building_cost(BuildingType::SupplyDepot);
        if engine.enemy_eco.minerals >= depot_cost {
            if let Some((bx, by)) = crate::autoplay::find_team_build_spot(engine, Team::Enemy, BuildingType::SupplyDepot) {
                crate::autoplay::try_team_auto_build(engine, Team::Enemy, BuildingType::SupplyDepot, bx, by);
            }
        }
    }

    // 3. Xây Power Reactor để có năng lượng dùng cho tech và vũ khí cao cấp
    let reactors = engine.buildings.iter().filter(|b| b.team == Team::Enemy && b.building_type == BuildingType::PowerReactor).count();
    if reactors < 3 && engine.enemy_eco.minerals >= 100.0 {
        let under_construction = engine.buildings.iter().any(|b| {
            b.team == Team::Enemy && b.building_type == BuildingType::PowerReactor && b.build_progress < 1.0
        });
        if !under_construction {
            if let Some((bx, by)) = crate::autoplay::find_team_build_spot(engine, Team::Enemy, BuildingType::PowerReactor) {
                crate::autoplay::try_team_auto_build(engine, Team::Enemy, BuildingType::PowerReactor, bx, by);
            }
        }
    } else if reactors < 14 && engine.enemy_eco.minerals >= 180.0 {
        let under_construction = engine.buildings.iter().any(|b| {
            b.team == Team::Enemy && b.building_type == BuildingType::PowerReactor && b.build_progress < 1.0
        });
        if !under_construction {
            if let Some((bx, by)) = crate::autoplay::find_team_build_spot(engine, Team::Enemy, BuildingType::PowerReactor) {
                crate::autoplay::try_team_auto_build(engine, Team::Enemy, BuildingType::PowerReactor, bx, by);
            }
        }
    }

    // 3.5. Xây EnergyConverter để biến Năng lượng thành Khoáng sản (Biến năng lượng thừa thành lính!)
    let converters = engine.buildings.iter().filter(|b| b.team == Team::Enemy && b.building_type == BuildingType::EnergyConverter).count();
    let under_construction_conv = engine.buildings.iter().any(|b| {
        b.team == Team::Enemy && b.building_type == BuildingType::EnergyConverter && b.build_progress < 1.0
    });
    if !under_construction_conv && engine.enemy_eco.minerals >= 150.0 && engine.enemy_eco.energy >= 50.0 {
        if converters < (reactors / 2).min(6).max(1) && (reactors >= 2 || engine.enemy_eco.energy >= 250.0) {
            if let Some((bx, by)) = crate::autoplay::find_team_build_spot(engine, Team::Enemy, BuildingType::EnergyConverter) {
                crate::autoplay::try_team_auto_build(engine, Team::Enemy, BuildingType::EnergyConverter, bx, by);
            }
        }
    }

    // 4. Xây Barracks (Trại lính): Mở rộng dây chuyền sản xuất quân đội theo đà kinh tế
    let barracks = engine.buildings.iter().filter(|b| b.team == Team::Enemy && b.building_type == BuildingType::Barracks).count();
    let under_construction_barracks = engine.buildings.iter().any(|b| {
        b.team == Team::Enemy && b.building_type == BuildingType::Barracks && b.build_progress < 1.0
    });
    if !under_construction_barracks && engine.enemy_eco.minerals >= 150.0 {
        let should_build_barracks = match barracks {
            0 => true,
            1 => enemy_workers >= 6,
            2 => enemy_workers >= 10 && reactors >= 2,
            3 => enemy_workers >= 12 && reactors >= 3,
            4 => enemy_workers >= 14 && reactors >= 3,
            5 => enemy_workers >= 14 && reactors >= 4,
            6 => enemy_workers >= 14 && reactors >= 4,
            7 => engine.enemy_eco.minerals >= 200.0,
            8 => engine.enemy_eco.minerals >= 220.0,
            9 => engine.enemy_eco.minerals >= 250.0,
            _ => false,
        };
        if should_build_barracks {
            if let Some((bx, by)) = crate::autoplay::find_team_build_spot(engine, Team::Enemy, BuildingType::Barracks) {
                crate::autoplay::try_team_auto_build(engine, Team::Enemy, BuildingType::Barracks, bx, by);
            }
        }
    }

    // 4.5. Xây TurretTower phòng tuyến căn cứ nếu có năng lượng
    let turrets = engine.buildings.iter().filter(|b| b.team == Team::Enemy && b.building_type == BuildingType::TurretTower).count();
    let under_construction_turrets = engine.buildings.iter().any(|b| {
        b.team == Team::Enemy && b.building_type == BuildingType::TurretTower && b.build_progress < 1.0
    });
    if turrets < 10 && !under_construction_turrets && engine.enemy_eco.minerals >= 100.0 && engine.enemy_eco.energy >= 50.0 {
        if let Some((bx, by)) = crate::autoplay::find_team_build_spot(engine, Team::Enemy, BuildingType::TurretTower) {
            crate::autoplay::try_team_auto_build(engine, Team::Enemy, BuildingType::TurretTower, bx, by);
        }
    }

    // 4.8. Luôn chủ động nâng cấp cây công nghệ khi có năng lượng thừa để mở khóa quân hạng nặng
    if engine.enemy_tech.active_researches.len() < 2 && engine.enemy_eco.energy >= 100.0 {
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
            if !crate::simulation::is_tech_unlocked(&engine.enemy_tech, t)
                && !engine.enemy_tech.active_researches.iter().any(|r| r.tech_id == t)
            {
                if crate::simulation::process_start_research(
                    &mut engine.enemy_tech,
                    &mut engine.enemy_eco,
                    &engine.config,
                    t,
                ) {
                    break;
                }
            }
        }
    }

    // 5. Luôn duy trì sản xuất quân tại mọi trại lính rảnh rỗi (Never let Barracks idle!)
    execute_bot_continuous_recruitment(engine);

    // 6. Điều phối quân sự thông minh:
    let enemy_combat = engine.units.iter().filter(|u| u.team == Team::Enemy && u.unit_type != UnitType::Engineer).count();
    let _player_combat = engine.units.iter().filter(|u| u.team == Team::Player && u.unit_type != UnitType::Engineer).count();

    // 6.1. Nếu căn cứ Bot bị đe dọa trực tiếp (<750px), lập tức kéo quân về phòng thủ
    let has_threat = engine.units.iter().any(|u| {
        u.team == Team::Player && ((u.x - base_pos.0).powi(2) + (u.y - base_pos.1).powi(2)) < 750.0_f32.powi(2)
    });
    if has_threat {
        command_bot_military(engine, MilitaryCommand::DefendBase);
        return;
    }

    // 6.2. Đòn timing attack áp đảo & Duy trì áp lực tấn công liên tục:
    let has_siege_tech = engine.enemy_tech.dark_matter_rockets || engine.enemy_tech.focused_laser;
    let assault_threshold = if has_siege_tech { 16 } else { 22 };

    let is_on_offensive = enemy_combat >= assault_threshold || engine.units.iter().any(|u| {
        u.team == Team::Enemy && u.unit_type != UnitType::Engineer && (u.x < 1400.0 || u.y > 1100.0)
    });

    if is_on_offensive {
        command_bot_military(engine, MilitaryCommand::AllOutAssault);
        return;
    }

    // 6.3. Khi chưa đủ quân số công thành và chưa tiến công, chỉ đưa các lính mới tại căn cứ (x > 1600) về điểm tập kết rally
    let idle_combat_units: Vec<u32> = engine.units.iter()
        .filter(|u| u.team == Team::Enemy && u.unit_type != UnitType::Engineer && u.order == UnitOrder::Idle && u.x > 1600.0)
        .map(|u| u.id)
        .collect();
    for (i, uid) in idle_combat_units.iter().enumerate() {
        if let Some(u) = engine.units.iter_mut().find(|u| u.id == *uid) {
            let offset_x = ((i % 5) as f32 - 2.0) * 25.0;
            let offset_y = ((i / 5) as f32) * 25.0;
            u.order = UnitOrder::MoveTo {
                x: 1750.0 + offset_x,
                y: 650.0 + offset_y,
            };
        }
    }
}

fn execute_bot_continuous_recruitment(engine: &mut GameEngine) {
    if engine.enemy_eco.supply >= engine.enemy_eco.max_supply {
        return;
    }

    let enemy_combat = engine.units.iter().filter(|u| u.team == Team::Enemy && u.unit_type != UnitType::Engineer).count();

    // Giữ quỹ dự phòng khoáng sản cho nghiên cứu công nghệ tối thượng (Dark Matter Rockets 220 min)
    let needs_tech = !engine.enemy_tech.dark_matter_rockets || !engine.enemy_tech.energy_shielding;
    let mineral_reserve = if needs_tech && enemy_combat >= 6 { 230.0 } else if enemy_combat >= 6 { 100.0 } else { 0.0 };

    let completed_barracks: Vec<u32> = engine.buildings.iter()
        .filter(|b| b.team == Team::Enemy && b.building_type == BuildingType::Barracks && b.build_progress >= 1.0 && b.training_queue.len() < 2)
        .map(|b| b.id)
        .collect();

    for bid in completed_barracks {
        if engine.enemy_eco.supply >= engine.enemy_eco.max_supply {
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

        for &u_type in &priority_units {
            if !crate::autoplay::can_train_unit(&engine.enemy_tech, u_type) {
                continue;
            }
            let (mut min_cost, mut en_cost) = engine.config.get_unit_cost(u_type);
            if engine.enemy_tech.hive_proliferation {
                min_cost *= 0.65;
                en_cost *= 0.65;
            }
            if engine.enemy_eco.minerals >= mineral_reserve + min_cost && engine.enemy_eco.energy >= en_cost {
                engine.enemy_eco.minerals -= min_cost;
                engine.enemy_eco.energy -= en_cost;
                if let Some(b) = engine.buildings.iter_mut().find(|b| b.id == bid) {
                    b.training_queue.push(u_type);
                    recruited = true;
                    break;
                }
            }
        }

        // Tuyển lính cơ bản (Rifleman / ShockTrooper) nếu còn tiền VƯỢT TRÊN mức quỹ dự phòng mineral_reserve
        if !recruited {
            let basic_units = [UnitType::ShockTrooper, UnitType::Rifleman];
            for &u_type in &basic_units {
                let (mut min_cost, _) = engine.config.get_unit_cost(u_type);
                if engine.enemy_tech.hive_proliferation {
                    min_cost *= 0.65;
                }
                if engine.enemy_eco.minerals >= mineral_reserve + min_cost {
                    engine.enemy_eco.minerals -= min_cost;
                    if let Some(b) = engine.buildings.iter_mut().find(|b| b.id == bid) {
                        b.training_queue.push(u_type);
                        break;
                    }
                }
            }
        }
    }
}

fn execute_bot_eco(engine: &mut GameEngine, try_expansion: bool) {
    let enemy_workers = engine.units.iter().filter(|u| u.team == Team::Enemy && u.unit_type == UnitType::Engineer).count();
    let enemy_ccs = engine.buildings.iter().filter(|b| b.team == Team::Enemy && b.building_type == BuildingType::CommandCenter).count();

    // Mua thợ mỏ nếu còn thiếu (< 16 thợ)
    if enemy_workers < 16 && engine.enemy_eco.supply < engine.enemy_eco.max_supply {
        let (w_cost, _) = engine.config.get_unit_cost(UnitType::Engineer);
        if engine.enemy_eco.minerals >= w_cost {
            if let Some(cc) = engine.buildings.iter_mut().find(|b| {
                b.team == Team::Enemy && b.building_type == BuildingType::CommandCenter && b.build_progress >= 1.0 && b.training_queue.len() < 2
            }) {
                engine.enemy_eco.minerals -= w_cost;
                cc.training_queue.push(UnitType::Engineer);
            }
        }
    }

    // Xây CommandCenter mở rộng mỏ phụ nếu có đủ tiền
    if try_expansion && enemy_ccs < 2 && engine.enemy_eco.minerals >= 400.0 {
        if let Some((bx, by)) = crate::autoplay::find_team_build_spot(engine, Team::Enemy, BuildingType::CommandCenter) {
            crate::autoplay::try_team_auto_build(engine, Team::Enemy, BuildingType::CommandCenter, bx, by);
        }
    }
}

fn execute_bot_infrastructure(engine: &mut GameEngine) {
    let reactors = engine.buildings.iter().filter(|b| b.team == Team::Enemy && b.building_type == BuildingType::PowerReactor).count();
    let converters = engine.buildings.iter().filter(|b| b.team == Team::Enemy && b.building_type == BuildingType::EnergyConverter).count();
    let barracks = engine.buildings.iter().filter(|b| b.team == Team::Enemy && b.building_type == BuildingType::Barracks).count();

    // Xây Barracks (trại lính) nếu còn ít (< 4 trại lính)
    if barracks < 4 {
        let (barracks_cost, _) = engine.config.get_building_cost(BuildingType::Barracks);
        if engine.enemy_eco.minerals >= barracks_cost {
            let under_construction = engine.buildings.iter().any(|b| {
                b.team == Team::Enemy && b.building_type == BuildingType::Barracks && b.build_progress < 1.0
            });
            if !under_construction {
                if let Some((bx, by)) = crate::autoplay::find_team_build_spot(engine, Team::Enemy, BuildingType::Barracks) {
                    crate::autoplay::try_team_auto_build(engine, Team::Enemy, BuildingType::Barracks, bx, by);
                }
            }
        }
    }

    // Xây PowerReactor nếu năng lượng thấp hoặc ít lò (< 8 lò)
    if reactors < 8 {
        let (reactor_cost, _) = engine.config.get_building_cost(BuildingType::PowerReactor);
        if engine.enemy_eco.minerals >= reactor_cost {
            let under_construction = engine.buildings.iter().any(|b| {
                b.team == Team::Enemy && b.building_type == BuildingType::PowerReactor && b.build_progress < 1.0
            });
            if !under_construction {
                if let Some((bx, by)) = crate::autoplay::find_team_build_spot(engine, Team::Enemy, BuildingType::PowerReactor) {
                    crate::autoplay::try_team_auto_build(engine, Team::Enemy, BuildingType::PowerReactor, bx, by);
                }
            }
        }
    }

    // Xây EnergyConverter khi có nhiều hơn 3 Reactor
    if reactors >= 3 && converters < (reactors / 3) {
        let (conv_m, conv_e) = engine.config.get_building_cost(BuildingType::EnergyConverter);
        if engine.enemy_eco.minerals >= conv_m && engine.enemy_eco.energy >= conv_e {
            let under_construction = engine.buildings.iter().any(|b| {
                b.team == Team::Enemy && b.building_type == BuildingType::EnergyConverter && b.build_progress < 1.0
            });
            if !under_construction {
                if let Some((bx, by)) = crate::autoplay::find_team_build_spot(engine, Team::Enemy, BuildingType::EnergyConverter) {
                    crate::autoplay::try_team_auto_build(engine, Team::Enemy, BuildingType::EnergyConverter, bx, by);
                }
            }
        }
    }
}

fn execute_bot_defense(engine: &mut GameEngine) {
    let turrets = engine.buildings.iter().filter(|b| b.team == Team::Enemy && b.building_type == BuildingType::TurretTower).count();
    if turrets < 6 {
        let (turret_m, turret_e) = engine.config.get_building_cost(BuildingType::TurretTower);
        if engine.enemy_eco.minerals >= turret_m && engine.enemy_eco.energy >= turret_e {
            if let Some((bx, by)) = crate::autoplay::find_team_build_spot(engine, Team::Enemy, BuildingType::TurretTower) {
                crate::autoplay::try_team_auto_build(engine, Team::Enemy, BuildingType::TurretTower, bx, by);
            }
        }
    }
}

fn try_bot_research(engine: &mut GameEngine, tech_chain: &[&str]) {
    for &tech_id in tech_chain {
        if !crate::simulation::is_tech_unlocked(&engine.enemy_tech, tech_id)
            && !engine.enemy_tech.active_researches.iter().any(|r| r.tech_id == tech_id)
        {
            crate::simulation::process_start_research(
                &mut engine.enemy_tech,
                &mut engine.enemy_eco,
                &engine.config,
                tech_id,
            );
            break;
        }
    }
}

fn execute_bot_training(engine: &mut GameEngine, candidate_units: &[UnitType]) {
    if engine.enemy_eco.supply >= engine.enemy_eco.max_supply {
        return;
    }

    let completed_barracks: Vec<u32> = engine.buildings.iter()
        .filter(|b| b.team == Team::Enemy && b.building_type == BuildingType::Barracks && b.build_progress >= 1.0 && b.training_queue.len() < 2)
        .map(|b| b.id)
        .collect();

    for bid in completed_barracks {
        if engine.enemy_eco.supply >= engine.enemy_eco.max_supply {
            break;
        }

        for &u_type in candidate_units {
            if u_type == UnitType::Engineer || !crate::autoplay::can_train_unit(&engine.enemy_tech, u_type) {
                continue;
            }

            let (mut min_cost, mut en_cost) = engine.config.get_unit_cost(u_type);
            if engine.enemy_tech.hive_proliferation {
                min_cost *= 0.65;
                en_cost *= 0.65;
            }

            if engine.enemy_eco.minerals >= min_cost && engine.enemy_eco.energy >= en_cost {
                engine.enemy_eco.minerals -= min_cost;
                engine.enemy_eco.energy -= en_cost;
                if let Some(b) = engine.buildings.iter_mut().find(|b| b.id == bid) {
                    b.training_queue.push(u_type);
                    break;
                }
            }
        }
    }
}

enum MilitaryCommand {
    DefendBase,
    HarassWorkers,
    HarassOrAttack,
    AllOutAssault,
    Regroup,
}

fn command_bot_military(engine: &mut GameEngine, cmd: MilitaryCommand) {
    let combat_unit_ids: Vec<u32> = engine.units.iter()
        .filter(|u| u.team == Team::Enemy && u.unit_type != UnitType::Engineer)
        .map(|u| u.id)
        .collect();

    if combat_unit_ids.is_empty() {
        return;
    }

    match cmd {
        MilitaryCommand::DefendBase => {
            let base_pos = (2150.0, 350.0);
            let nearest_threat = engine.units.iter()
                .filter(|u| u.team == Team::Player)
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
            // Tìm thợ mỏ hoặc mỏ quặng người chơi đang khai thác
            let player_worker_pos = engine.units.iter()
                .find(|u| u.team == Team::Player && u.unit_type == UnitType::Engineer)
                .map(|u| (u.x, u.y))
                .unwrap_or((300.0, 1950.0));

            for (i, uid) in combat_unit_ids.iter().enumerate() {
                if let Some(u) = engine.units.iter_mut().find(|u| u.id == *uid) {
                    let offset_x = ((i % 4) as f32 - 1.5) * 25.0;
                    let offset_y = ((i / 4) as f32) * 25.0;
                    u.order = UnitOrder::AttackMove {
                        x: player_worker_pos.0 + offset_x,
                        y: player_worker_pos.1 + offset_y,
                    };
                }
            }
        }
        MilitaryCommand::HarassOrAttack | MilitaryCommand::AllOutAssault => {
            // Tác chiến tổng lực: Bao vây và dồn sát thương tiêu diệt nhà chính Player (CommandCenter)
            let target = engine.buildings.iter()
                .find(|b| b.team == Team::Player && b.building_type == BuildingType::CommandCenter)
                .map(|b| (b.x, b.y))
                .unwrap_or((250.0, 2050.0));

            for (i, uid) in combat_unit_ids.iter().enumerate() {
                if let Some(u) = engine.units.iter_mut().find(|u| u.id == *uid) {
                    // Không ngắt nhịp nếu đơn vị đang bận khai hỏa vào mục tiêu (AttackTarget)
                    if let UnitOrder::AttackTarget { .. } = u.order {
                        continue;
                    }
                    // Nếu đang AttackMove về mục tiêu căn cứ rồi thì không làm gián đoạn
                    if let UnitOrder::AttackMove { x, y } = u.order {
                        let d_sq = (x - target.0).powi(2) + (y - target.1).powi(2);
                        if d_sq < 90.0_f32.powi(2) {
                            continue;
                        }
                    }

                    // Tản nhẹ tọa độ xung quanh nhà chính để bao vây toàn diện
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
            let rally = (1750.0, 650.0);
            for (i, uid) in combat_unit_ids.iter().enumerate() {
                if let Some(u) = engine.units.iter_mut().find(|u| u.id == *uid) {
                    if let UnitOrder::AttackTarget { .. } = u.order {
                        continue;
                    }
                    if let UnitOrder::MoveTo { x, y } = u.order {
                        let d_sq = (x - rally.0).powi(2) + (y - rally.1).powi(2);
                        if d_sq < 100.0_f32.powi(2) {
                            continue;
                        }
                    }
                    let offset_x = ((i % 5) as f32 - 2.0) * 25.0;
                    let offset_y = ((i / 5) as f32) * 25.0;
                    u.order = UnitOrder::MoveTo {
                        x: rally.0 + offset_x,
                        y: rally.1 + offset_y,
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
        let s = extract_state_vector(&engine);
        assert_eq!(s.len(), STATE_DIM);
    }

    #[test]
    fn test_strategic_action_mapping() {
        for idx in 0..11 {
            let action = BotStrategicAction::from_index(idx);
            assert_eq!(action.to_index(), idx);
        }
    }
}
