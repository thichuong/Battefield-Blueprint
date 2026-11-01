use crate::components::{BuildingType, Team, UnitOrder, UnitType};
use crate::resources::TechTreeState;
use crate::GameEngine;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum AutoCondition {
    Always,
    SupplyNearCap,
    UnitCountLessThan { unit_type: String, count: usize },
    UnitCountGreaterThan { unit_type: String, count: usize },
    MineralsGreaterThan { amount: f32 },
    EnergyGreaterThan { amount: f32 },
    ReactorPerConverterRatio,
    BuildingCountLessThan { building_type: String, count: usize },
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum AutoAction {
    TrainUnit {
        unit_type: String,
        count: u32,
    },
    BuildStructure {
        building_type: String,
        count: u32,
    },
    ResearchTech {
        tech_id: String,
        use_priority_queue: bool,
    },
    AttackWave,
    Delay {
        seconds: f32,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AutoCard {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub conditions: Vec<AutoCondition>,
    pub action: AutoAction,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AutoLoop {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub current_card_index: usize,
    pub current_card_progress: u32,
    #[serde(default)]
    pub card_timer: f32,
    pub cards: Vec<AutoCard>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TurretDefenseLine {
    pub enabled: bool,
    pub p1: (f32, f32), // Điểm bắt đầu
    pub p2: (f32, f32), // Điểm kết thúc
    pub spacing: f32,   // Khoảng cách giữa các tháp (mặc định 90.0)
    pub rows: u32,      // Số hàng tháp (mặc định 1)
    pub row_spacing: f32, // Khoảng cách giữa các hàng (mặc định 80.0)
}

impl TurretDefenseLine {
    pub fn for_team_base(base: (f32, f32)) -> Self {
        let center_x = crate::map::MAP_WIDTH * 0.5;
        let center_y = crate::map::MAP_HEIGHT * 0.5;
        let dx = center_x - base.0;
        let dy = center_y - base.1;
        let len = (dx * dx + dy * dy).sqrt().max(1.0);
        let dir_x = dx / len;
        let dir_y = dy / len;
        // Điểm giữa tuyến phòng thủ cách base 400px hướng về tâm map
        let mid_x = base.0 + dir_x * 400.0;
        let mid_y = base.1 + dir_y * 400.0;
        // Vector vuông góc
        let perp_x = -dir_y;
        let perp_y = dir_x;
        Self {
            enabled: true,
            p1: (mid_x - perp_x * 200.0, mid_y - perp_y * 200.0),
            p2: (mid_x + perp_x * 200.0, mid_y + perp_y * 200.0),
            spacing: 90.0,
            rows: 1,
            row_spacing: 80.0,
        }
    }
}

impl Default for TurretDefenseLine {
    fn default() -> Self {
        Self::for_team_base((550.0, 4250.0))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AutoPlayConfig {
    pub enabled: bool,
    pub priority_techs: Vec<String>,
    #[serde(default)]
    pub turret_defense: TurretDefenseLine,
    pub loops: Vec<AutoLoop>,
}

impl Default for AutoPlayConfig {
    fn default() -> Self {
        Self::default_for_team(Team::Player)
    }
}

impl AutoPlayConfig {
    pub fn default_for_enemy() -> Self {
        Self::default_for_team(Team::Enemy)
    }

    pub fn default_for_team(team: Team) -> Self {
        let base = crate::map::get_spawn_positions().get(team.index()).copied().unwrap_or((550.0, 4250.0));
        let turret_defense = TurretDefenseLine::for_team_base(base);
        // Loop 1: Sản xuất (Production)
        let production_loop = AutoLoop {
            id: "loop_production".to_string(),
            name: "Sản Xuất".to_string(),
            enabled: true,
            current_card_index: 0,
            current_card_progress: 0,
            card_timer: 0.0,
            cards: vec![
                AutoCard {
                    id: "card_prod_worker".to_string(),
                    name: "Mua 2 Thợ Mỏ".to_string(),
                    enabled: true,
                    conditions: vec![AutoCondition::UnitCountLessThan {
                        unit_type: "Engineer".to_string(),
                        count: 7,
                    }],
                    action: AutoAction::TrainUnit {
                        unit_type: "Engineer".to_string(),
                        count: 2,
                    },
                },
                AutoCard {
                    id: "card_prod_reactor".to_string(),
                    name: "Xây 2 PowerReactor".to_string(),
                    enabled: true,
                    conditions: vec![AutoCondition::BuildingCountLessThan {
                        building_type: "PowerReactor".to_string(),
                        count: 10,
                    }],
                    action: AutoAction::BuildStructure {
                        building_type: "PowerReactor".to_string(),
                        count: 2,
                    },
                },
                AutoCard {
                    id: "card_prod_turret".to_string(),
                    name: "Xây 2 TurretTower".to_string(),
                    enabled: true,
                    conditions: vec![AutoCondition::BuildingCountLessThan {
                        building_type: "TurretTower".to_string(),
                        count: 8,
                    }],
                    action: AutoAction::BuildStructure {
                        building_type: "TurretTower".to_string(),
                        count: 2,
                    },
                },
                AutoCard {
                    id: "card_prod_tech".to_string(),
                    name: "Nâng cấp Công Nghệ".to_string(),
                    enabled: true,
                    conditions: vec![AutoCondition::Always],
                    action: AutoAction::ResearchTech {
                        tech_id: "dark_matter_rockets".to_string(),
                        use_priority_queue: true,
                    },
                },
                AutoCard {
                    id: "card_prod_barracks".to_string(),
                    name: "Xây 1 Barracks".to_string(),
                    enabled: true,
                    conditions: vec![AutoCondition::BuildingCountLessThan {
                        building_type: "Barracks".to_string(),
                        count: 6,
                    }],
                    action: AutoAction::BuildStructure {
                        building_type: "Barracks".to_string(),
                        count: 1,
                    },
                },
                AutoCard {
                    id: "card_prod_converter".to_string(),
                    name: "Xây EnergyConverter (Tỷ lệ 4:1)".to_string(),
                    enabled: true,
                    conditions: vec![AutoCondition::ReactorPerConverterRatio],
                    action: AutoAction::BuildStructure {
                        building_type: "EnergyConverter".to_string(),
                        count: 1,
                    },
                },
                AutoCard {
                    id: "card_prod_delay".to_string(),
                    name: "Tạm nghỉ công trình (Delay 5s)".to_string(),
                    enabled: true,
                    conditions: vec![AutoCondition::Always],
                    action: AutoAction::Delay { seconds: 5.0 },
                },
            ],
        };

        // Loop 2: Quân sự (Military)
        let military_loop = AutoLoop {
            id: "loop_military".to_string(),
            name: "Quân Sự".to_string(),
            enabled: true,
            current_card_index: 0,
            current_card_progress: 0,
            card_timer: 0.0,
            cards: vec![
                AutoCard {
                    id: "card_mil_supply".to_string(),
                    name: "Xây SupplyDepot tránh kẹt".to_string(),
                    enabled: true,
                    conditions: vec![AutoCondition::SupplyNearCap],
                    action: AutoAction::BuildStructure {
                        building_type: "SupplyDepot".to_string(),
                        count: 1,
                    },
                },
                AutoCard {
                    id: "card_mil_shock".to_string(),
                    name: "Mua ShockTrooper (>150 Khoáng)".to_string(),
                    enabled: true,
                    conditions: vec![AutoCondition::MineralsGreaterThan { amount: 150.0 }],
                    action: AutoAction::TrainUnit {
                        unit_type: "ShockTrooper".to_string(),
                        count: 1,
                    },
                },
                AutoCard {
                    id: "card_mil_rifle".to_string(),
                    name: "Mua Rifleman (>150 Khoáng)".to_string(),
                    enabled: true,
                    conditions: vec![AutoCondition::MineralsGreaterThan { amount: 150.0 }],
                    action: AutoAction::TrainUnit {
                        unit_type: "Rifleman".to_string(),
                        count: 1,
                    },
                },
                AutoCard {
                    id: "card_mil_attack".to_string(),
                    name: "Tự động tấn công (>20 Lính)".to_string(),
                    enabled: true,
                    conditions: vec![AutoCondition::UnitCountGreaterThan {
                        unit_type: "Combat".to_string(),
                        count: 20,
                    }],
                    action: AutoAction::AttackWave,
                },
            ],
        };

        Self {
            enabled: false,
            priority_techs: Vec::new(),
            turret_defense,
            loops: vec![production_loop, military_loop],
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AutoPlayState {
    pub config: AutoPlayConfig,
    pub timer: f32,
    pub last_action_msg: String,
}

impl Default for AutoPlayState {
    fn default() -> Self {
        Self {
            config: AutoPlayConfig::default(),
            timer: 0.0,
            last_action_msg: "".to_string(),
        }
    }
}

impl AutoPlayState {
    pub fn default_for_enemy() -> Self {
        Self {
            config: AutoPlayConfig::default_for_enemy(),
            timer: 0.0,
            last_action_msg: "".to_string(),
        }
    }
}

pub fn update_autoplay(engine: &mut GameEngine, dt: f32) {
    update_team_autoplay(engine, Team::Player, dt);
}

pub fn update_team_autoplay(engine: &mut GameEngine, team: Team, dt: f32) {
    let enabled = engine.team_autoplay(team).config.enabled;
    if !enabled {
        return;
    }

    let timer = &mut engine.team_autoplay_mut(team).timer;
    *timer += dt;
    if *timer < 0.25 {
        return;
    }
    let elapsed = *timer;
    *timer = 0.0;

    let num_loops = engine.team_autoplay(team).config.loops.len();

    for loop_idx in 0..num_loops {
        let loop_enabled = engine.team_autoplay(team).config.loops[loop_idx].enabled;
        if !loop_enabled {
            continue;
        }
        step_team_loop(engine, team, loop_idx, elapsed);
    }
}

fn step_team_loop(engine: &mut GameEngine, team: Team, loop_idx: usize, dt: f32) {
    let loop_data = &engine.team_autoplay(team).config.loops[loop_idx];
    if loop_data.cards.is_empty() {
        return;
    }

    let current_idx = loop_data.current_card_index % loop_data.cards.len();
    let card = loop_data.cards[current_idx].clone();

    if !card.enabled {
        advance_team_loop_card(engine, team, loop_idx);
        return;
    }

    let all_conditions_met = if card.conditions.is_empty() {
        true
    } else {
        card.conditions.iter().all(|c| evaluate_condition_for_team(engine, team, c))
    };

    if !all_conditions_met {
        advance_team_loop_card(engine, team, loop_idx);
        return;
    }

    let action_completed = execute_action_for_team(engine, team, loop_idx, &card.action, dt);
    if action_completed {
        advance_team_loop_card(engine, team, loop_idx);
    }
}

fn advance_team_loop_card(engine: &mut GameEngine, team: Team, loop_idx: usize) {
    let loop_data = &mut engine.team_autoplay_mut(team).config.loops[loop_idx];
    loop_data.current_card_progress = 0;
    loop_data.card_timer = 0.0;
    if !loop_data.cards.is_empty() {
        loop_data.current_card_index = (loop_data.current_card_index + 1) % loop_data.cards.len();
    }
}

fn evaluate_condition_for_team(engine: &GameEngine, team: Team, condition: &AutoCondition) -> bool {
    let eco = engine.team_eco(team);
    match condition {
        AutoCondition::Always => true,
        AutoCondition::SupplyNearCap => {
            if eco.max_supply >= 200 {
                return false;
            }
            let depot_under_construction = engine
                .buildings
                .iter()
                .filter(|b| {
                    b.team == team
                        && b.building_type == BuildingType::SupplyDepot
                        && b.build_progress < 1.0
                })
                .count();

            if depot_under_construction >= 2 {
                return false;
            }

            (eco.supply + 2 >= eco.max_supply)
                || (eco.minerals >= 350.0 && eco.supply + 6 >= eco.max_supply)
        }
        AutoCondition::UnitCountLessThan { unit_type, count } => {
            let actual = count_team_units(engine, team, unit_type);
            actual < *count
        }
        AutoCondition::UnitCountGreaterThan { unit_type, count } => {
            let actual = count_team_units(engine, team, unit_type);
            actual > *count
        }
        AutoCondition::MineralsGreaterThan { amount } => eco.minerals > *amount,
        AutoCondition::EnergyGreaterThan { amount } => eco.energy > *amount,
        AutoCondition::ReactorPerConverterRatio => {
            let reactors = engine
                .buildings
                .iter()
                .filter(|b| {
                    b.team == team
                        && b.building_type == BuildingType::PowerReactor
                        && b.build_progress >= 1.0
                })
                .count();
            let converters = engine
                .buildings
                .iter()
                .filter(|b| {
                    b.team == team && b.building_type == BuildingType::EnergyConverter
                })
                .count();
            (reactors / 4) > converters
        }
        AutoCondition::BuildingCountLessThan {
            building_type,
            count,
        } => {
            let b_type = parse_building_type(building_type);
            if let Some(bt) = b_type {
                let actual = engine
                    .buildings
                    .iter()
                    .filter(|b| b.team == team && b.building_type == bt)
                    .count();
                actual < *count
            } else {
                false
            }
        }
    }
}

pub fn can_train_unit(tech: &TechTreeState, u_type: UnitType) -> bool {
    match u_type {
        UnitType::Engineer | UnitType::ShockTrooper | UnitType::Rifleman => true,
        UnitType::Vanguard => tech.energy_shielding,
        UnitType::LaserMarksman => tech.focused_laser,
        UnitType::DarkRocketeer => tech.dark_matter_rockets,
        UnitType::TaoistSwordmaster => tech.celestial_flying_swords,
        UnitType::VoidWarlock => tech.void_cataclysm,
        UnitType::BehemothTitan => tech.behemoth_mutation,
        UnitType::AcidBaneling => tech.acid_banelings,
    }
}

#[allow(dead_code)]
fn execute_action(engine: &mut GameEngine, loop_idx: usize, action: &AutoAction, dt: f32) -> bool {
    execute_action_for_team(engine, Team::Player, loop_idx, action, dt)
}

fn execute_action_for_team(
    engine: &mut GameEngine,
    team: Team,
    loop_idx: usize,
    action: &AutoAction,
    dt: f32,
) -> bool {
    match action {
        AutoAction::Delay { seconds } => {
            let loop_data = &mut engine.team_autoplay_mut(team).config.loops[loop_idx];
            let timer = loop_data.card_timer + dt;
            loop_data.card_timer = timer;
            if timer >= *seconds {
                return true;
            }
            false
        }
        AutoAction::TrainUnit { unit_type, count } => {
            let u_type = parse_unit_type(unit_type);
            let u_type = match u_type {
                Some(t) => t,
                None => return true,
            };

            let tech = engine.team_tech(team);
            if !can_train_unit(tech, u_type) {
                return true;
            }

            let (mut min_cost, mut en_cost) = engine.config.get_unit_cost(u_type);
            if tech.hive_proliferation {
                min_cost *= 0.65;
                en_cost *= 0.65;
            }

            let eco = engine.team_eco(team);
            if eco.supply >= eco.max_supply {
                return false;
            }
            if eco.minerals < min_cost || eco.energy < en_cost {
                return false;
            }

            let target_btype = match u_type {
                UnitType::Engineer => BuildingType::CommandCenter,
                _ => BuildingType::Barracks,
            };

            let has_building = engine
                .buildings
                .iter()
                .any(|b| b.team == team && b.building_type == target_btype && b.build_progress >= 1.0);
            if !has_building {
                return true;
            }

            let maybe_b_id = engine
                .buildings
                .iter()
                .filter(|b| {
                    b.team == team
                        && b.building_type == target_btype
                        && b.build_progress >= 1.0
                        && b.training_queue.len() < 5
                })
                .min_by_key(|b| b.training_queue.len())
                .map(|b| b.id);

            if let Some(bid) = maybe_b_id {
                let eco = engine.team_eco_mut(team);
                eco.minerals -= min_cost;
                eco.energy -= en_cost;

                if let Some(b) = engine.buildings.iter_mut().find(|b| b.id == bid) {
                    b.training_queue.push(u_type);
                    let loop_data = &mut engine.team_autoplay_mut(team).config.loops[loop_idx];
                    let progress = loop_data.current_card_progress + 1;
                    loop_data.current_card_progress = progress;
                    if progress >= *count {
                        return true;
                    }
                }
            }
            false
        }
        AutoAction::BuildStructure {
            building_type,
            count,
        } => {
            let b_type = parse_building_type(building_type);
            let b_type = match b_type {
                Some(t) => t,
                None => return true,
            };

            let (min_cost, en_cost) = engine.config.get_building_cost(b_type);
            let eco = engine.team_eco(team);
            if eco.minerals < min_cost || eco.energy < en_cost {
                return false;
            }

            let spot = find_team_build_spot(engine, team, b_type);

            if let Some((bx, by)) = spot {
                if try_team_auto_build(engine, team, b_type, bx, by) {
                    let loop_data = &mut engine.team_autoplay_mut(team).config.loops[loop_idx];
                    let progress = loop_data.current_card_progress + 1;
                    loop_data.current_card_progress = progress;
                    if progress >= *count {
                        return true;
                    }
                }
                false
            } else {
                let msg = format!("Khu vực đã kín chỗ xây {}", building_type);
                engine.team_autoplay_mut(team).last_action_msg = msg;
                true
            }
        }
        AutoAction::ResearchTech {
            tech_id,
            use_priority_queue,
        } => {
            let mut target_tech = tech_id.clone();
            let priority_list = engine.team_autoplay(team).config.priority_techs.clone();
            let team_tech = engine.team_tech(team);

            if *use_priority_queue {
                for p_tech in priority_list {
                    if !crate::simulation::is_tech_unlocked(team_tech, &p_tech)
                        && !team_tech.active_researches.iter().any(|r| r.tech_id == p_tech)
                    {
                        if let Some(req) = get_tech_prerequisite(&p_tech) {
                            if !crate::simulation::is_tech_unlocked(team_tech, &req) {
                                target_tech = req;
                                break;
                            }
                        }
                        target_tech = p_tech;
                        break;
                    }
                }
            }

            let team_tech = engine.team_tech(team);
            if crate::simulation::is_tech_unlocked(team_tech, &target_tech) {
                return true;
            }
            if team_tech.active_researches.iter().any(|r| r.tech_id == target_tech) {
                return true;
            }

            let (tech_state, eco_state, cfg) = match team {
                Team::Player => (&mut engine.player_tech, &mut engine.player_eco, &engine.config),
                Team::Enemy => (&mut engine.enemy_tech, &mut engine.enemy_eco, &engine.config),
                _ => {
                    let idx = team.index() - 2;
                    (&mut engine.extra_techs[idx], &mut engine.extra_ecos[idx], &engine.config)
                }
            };

            if crate::simulation::process_start_research(
                tech_state,
                eco_state,
                cfg,
                &target_tech,
            ) {
                if team == Team::Player {
                    engine.outgoing_events.push(crate::bridge::EngineToReactEvent::PlaySound {
                        sound: "research_start".to_string(),
                    });
                }
                return true;
            }

            false
        }
        AutoAction::AttackWave => {
            let target_pos = engine
                .buildings
                .iter()
                .find(|b| engine.are_hostile(team, b.team) && b.building_type == BuildingType::CommandCenter)
                .map(|b| (b.x, b.y))
                .or_else(|| {
                    engine
                        .buildings
                        .iter()
                        .find(|b| engine.are_hostile(team, b.team))
                        .map(|b| (b.x, b.y))
                })
                .or_else(|| {
                    engine
                        .units
                        .iter()
                        .find(|u| engine.are_hostile(team, u.team))
                        .map(|u| (u.x, u.y))
                })
                .unwrap_or_else(|| {
                    (crate::map::MAP_WIDTH * 0.5, crate::map::MAP_HEIGHT * 0.5)
                });

            let combat_units: Vec<u32> = engine
                .units
                .iter()
                .filter(|u| u.team == team && u.unit_type != UnitType::Engineer)
                .map(|u| u.id)
                .collect();

            for (i, uid) in combat_units.iter().enumerate() {
                if let Some(u) = engine.units.iter_mut().find(|u| u.id == *uid) {
                    let offset_x = ((i % 6) as f32 - 2.5) * 25.0;
                    let offset_y = ((i / 6) as f32) * 25.0;
                    u.order = UnitOrder::AttackMove {
                        x: target_pos.0 + offset_x,
                        y: target_pos.1 + offset_y,
                    };
                }
            }

            if team == Team::Player {
                engine.outgoing_events.push(crate::bridge::EngineToReactEvent::PlaySound {
                    sound: "attack_order".to_string(),
                });
            }
            true
        }
    }
}

pub fn try_player_auto_build(
    engine: &mut GameEngine,
    b_type: BuildingType,
    wx: f32,
    wy: f32,
) -> bool {
    try_team_auto_build(engine, Team::Player, b_type, wx, wy)
}

pub fn try_team_auto_build(
    engine: &mut GameEngine,
    team: Team,
    b_type: BuildingType,
    wx: f32,
    wy: f32,
) -> bool {
    let (cost_min, cost_en) = engine.config.get_building_cost(b_type);
    let eco = engine.team_eco_mut(team);
    if eco.minerals < cost_min || eco.energy < cost_en {
        return false;
    }

    eco.minerals -= cost_min;
    eco.energy -= cost_en;
    let bid = engine.alloc_id();

    let mut building = engine
        .config
        .create_building(bid, team, b_type, wx, wy, false);
    building.build_progress = 0.05;
    building.hp = 1.0;
    building.rally_x = if team == Team::Player { wx + 40.0 } else { wx - 40.0 };
    building.rally_y = if team == Team::Player { wy + 40.0 } else { wy - 40.0 };

    let tech = engine.team_tech(team);

    if tech.titan_alloy_plating {
        building.resistances.physical = (building.resistances.physical + 0.20).min(0.80);
    }
    let mut hp_mult = 1.0;
    if tech.fortified_blueprints {
        hp_mult += 0.3;
    }
    if tech.titan_alloy_plating {
        hp_mult += 0.25;
    }
    building.max_hp *= hp_mult;

    let is_armed = engine
        .config
        .get_building(b_type)
        .map(|c| c.turret_damage > 0.0)
        .unwrap_or(false);
    if is_armed && tech.dark_matter_rockets {
        building.damage_type = crate::components::DamageType::Explosive;
    }

    engine.buildings.push(building);

    let nearest_worker_id = engine
        .units
        .iter()
        .filter(|u| u.team == team && u.unit_type == UnitType::Engineer)
        .min_by(|a, b| {
            let da = (a.x - wx).powi(2) + (a.y - wy).powi(2);
            let db = (b.x - wx).powi(2) + (b.y - wy).powi(2);
            da.partial_cmp(&db).unwrap()
        })
        .map(|u| u.id);

    if let Some(wid) = nearest_worker_id {
        if let Some(w) = engine.units.iter_mut().find(|u| u.id == wid) {
            w.order = UnitOrder::Build {
                target_building_id: bid,
            };
        }
    }

    if team == Team::Player {
        engine.outgoing_events.push(crate::bridge::EngineToReactEvent::PlaySound {
            sound: "place_building".to_string(),
        });
    }

    true
}

fn count_team_units(engine: &GameEngine, team: Team, unit_type: &str) -> usize {
    if unit_type.eq_ignore_ascii_case("combat") {
        engine
            .units
            .iter()
            .filter(|u| u.team == team && u.unit_type != UnitType::Engineer)
            .count()
    } else if let Some(ut) = parse_unit_type(unit_type) {
        engine
            .units
            .iter()
            .filter(|u| u.team == team && u.unit_type == ut)
            .count()
    } else {
        0
    }
}

#[allow(dead_code)]
fn count_player_units(engine: &GameEngine, unit_type: &str) -> usize {
    if unit_type.eq_ignore_ascii_case("combat") {
        engine
            .units
            .iter()
            .filter(|u| u.team == Team::Player && u.unit_type != UnitType::Engineer)
            .count()
    } else if let Some(ut) = parse_unit_type(unit_type) {
        engine
            .units
            .iter()
            .filter(|u| u.team == Team::Player && u.unit_type == ut)
            .count()
    } else {
        0
    }
}

fn parse_unit_type(s: &str) -> Option<UnitType> {
    match s.to_lowercase().as_str() {
        "engineer" | "worker" => Some(UnitType::Engineer),
        "shocktrooper" | "striker" => Some(UnitType::ShockTrooper),
        "rifleman" | "rifle" => Some(UnitType::Rifleman),
        "vanguard" | "shield" => Some(UnitType::Vanguard),
        "lasermarksman" | "laser" => Some(UnitType::LaserMarksman),
        "darkrocketeer" | "rocket" => Some(UnitType::DarkRocketeer),
        "taoistswordmaster" | "sword" => Some(UnitType::TaoistSwordmaster),
        "voidwarlock" | "warlock" => Some(UnitType::VoidWarlock),
        "behemothtitan" | "titan" => Some(UnitType::BehemothTitan),
        "acidbaneling" | "baneling" => Some(UnitType::AcidBaneling),
        _ => None,
    }
}

fn parse_building_type(s: &str) -> Option<BuildingType> {
    match s.to_lowercase().as_str() {
        "commandcenter" | "cc" | "hq" => Some(BuildingType::CommandCenter),
        "supplydepot" | "supply" => Some(BuildingType::SupplyDepot),
        "powerreactor" | "reactor" => Some(BuildingType::PowerReactor),
        "barracks" => Some(BuildingType::Barracks),
        "turrettower" | "turret" => Some(BuildingType::TurretTower),
        "energyconverter" | "converter" => Some(BuildingType::EnergyConverter),
        _ => None,
    }
}

fn get_tech_prerequisite(tech_id: &str) -> Option<String> {
    match tech_id {
        "energy_shielding" => Some("focused_laser".to_string()),
        "titan_alloy_plating" => Some("dark_matter_rockets".to_string()),
        "golden_bell_shield" => Some("celestial_flying_swords".to_string()),
        "void_cataclysm" => Some("corrosive_curse".to_string()),
        "behemoth_mutation" => Some("primeval_scales".to_string()),
        "hive_proliferation" => Some("acid_banelings".to_string()),
        "overcharged_reactors" => Some("optical_drills".to_string()),
        "fortified_blueprints" => Some("overcharged_reactors".to_string()),
        "exo_boosters" => Some("magnetic_rounds".to_string()),
        _ => None,
    }
}

/// Tính danh sách tất cả các điểm đặt tháp dọc theo hàng rào phòng thủ
pub fn compute_defense_tower_spots(defense_line: &TurretDefenseLine) -> Vec<(f32, f32)> {
    let mut spots = Vec::new();
    let (x1, y1) = defense_line.p1;
    let (x2, y2) = defense_line.p2;

    let dx = x2 - x1;
    let dy = y2 - y1;
    let len = (dx * dx + dy * dy).sqrt();

    if len < 10.0 {
        return vec![(x1, y1)];
    }

    let ux = dx / len;
    let uy = dy / len;

    // Vector pháp tuyến vuông góc hướng ra ngoài (về phía địch / Đông Bắc)
    // Nếu trục line chạy từ Tây Bắc sang Đông Nam, pháp tuyến (-uy, ux) sẽ hướng về phía Đông Bắc
    let nx = -uy;
    let ny = ux;

    let spacing = defense_line.spacing.max(45.0);
    let num_towers_per_row = ((len / spacing).floor() as usize) + 1;
    let row_spacing = defense_line.row_spacing.max(40.0);
    let rows = defense_line.rows.max(1) as usize;

    for r in 0..rows {
        let row_offset_x = (r as f32) * row_spacing * nx;
        let row_offset_y = (r as f32) * row_spacing * ny;

        for i in 0..num_towers_per_row {
            let t = if num_towers_per_row > 1 {
                (i as f32) * spacing
            } else {
                len * 0.5
            };
            if t <= len + 5.0 {
                let sx = x1 + t * ux + row_offset_x;
                let sy = y1 + t * uy + row_offset_y;
                spots.push((sx, sy));
            }
        }
    }

    spots
}

/// Tìm điểm còn trống trên hàng rào chưa có TurretTower và hợp lệ để đặt móng
pub fn find_defense_line_spot(
    engine: &GameEngine,
    defense_line: &TurretDefenseLine,
) -> Option<(f32, f32)> {
    find_defense_line_spot_for_team(engine, Team::Player, defense_line)
}

pub fn find_defense_line_spot_for_team(
    engine: &GameEngine,
    team: Team,
    defense_line: &TurretDefenseLine,
) -> Option<(f32, f32)> {
    let spots = compute_defense_tower_spots(defense_line);
    let (bw, bh) = engine
        .config
        .get_building(BuildingType::TurretTower)
        .map(|c| (c.width, c.height))
        .unwrap_or((60.0, 60.0));

    for (sx, sy) in spots {
        let already_has_turret = engine.buildings.iter().any(|b| {
            b.team == team
                && b.building_type == BuildingType::TurretTower
                && (b.x - sx).powi(2) + (b.y - sy).powi(2) < 45.0_f32.powi(2)
        });
        if already_has_turret {
            continue;
        }

        let collides_building = engine.buildings.iter().any(|b| {
            let clearance_x = (bw + b.width) * 0.5 + 15.0;
            let clearance_y = (bh + b.height) * 0.5 + 15.0;
            (sx - b.x).abs() < clearance_x && (sy - b.y).abs() < clearance_y
        });
        if collides_building {
            continue;
        }

        let collides_minerals = engine.minerals.iter().any(|m| {
            let dist = ((sx - m.x).powi(2) + (sy - m.y).powi(2)).sqrt();
            dist < 75.0
        });
        if collides_minerals {
            continue;
        }

        let collides_obstacles = engine.obstacles.iter().any(|obs| {
            let obs_cx = obs.x + obs.width * 0.5;
            let obs_cy = obs.y + obs.height * 0.5;
            let clearance_x = (bw + obs.width) * 0.5 + 12.0;
            let clearance_y = (bh + obs.height) * 0.5 + 12.0;
            (sx - obs_cx).abs() < clearance_x && (sy - obs_cy).abs() < clearance_y
        });
        if collides_obstacles {
            continue;
        }

        // Không thể đặt móng công trình đè lên đầu quân địch đang giao chiến
        let collides_enemies = engine.units.iter().any(|u| {
            u.team != team && ((sx - u.x).powi(2) + (sy - u.y).powi(2)).sqrt() < 80.0
        });
        if collides_enemies {
            continue;
        }

        if !(120.0..=2280.0).contains(&sx) || !(120.0..=2280.0).contains(&sy) {
            continue;
        }

        return Some((sx, sy));
    }

    None
}

pub fn find_player_build_spot(
    engine: &GameEngine,
    b_type: BuildingType,
) -> Option<(f32, f32)> {
    find_team_build_spot(engine, Team::Player, b_type)
}

pub fn find_team_build_spot(
    engine: &GameEngine,
    team: Team,
    b_type: BuildingType,
) -> Option<(f32, f32)> {
    let (bw, bh) = engine
        .config
        .get_building(b_type)
        .map(|c| (c.width, c.height))
        .unwrap_or((70.0, 70.0));

    let defense_line = &engine.team_autoplay(team).config.turret_defense;

    if b_type == BuildingType::TurretTower && defense_line.enabled {
        if let Some(spot) = find_defense_line_spot_for_team(engine, team, defense_line) {
            return Some(spot);
        }

        let mut extended_line = defense_line.clone();
        if extended_line.rows < 3 {
            extended_line.rows = (extended_line.rows + 1).min(3);
            if let Some(spot) = find_defense_line_spot_for_team(engine, team, &extended_line) {
                return Some(spot);
            }
        }
    }

    if b_type == BuildingType::CommandCenter {
        let mut candidate_minerals: Vec<(f32, f32)> = Vec::new();
        for m in &engine.minerals {
            let has_cc_nearby = engine.buildings.iter().any(|b| {
                b.building_type == BuildingType::CommandCenter
                    && (b.x - m.x).powi(2) + (b.y - m.y).powi(2) < 380.0_f32.powi(2)
            });
            if !has_cc_nearby {
                candidate_minerals.push((m.x, m.y));
            }
        }

        let base_ref = engine.base_pos(team);
        candidate_minerals.sort_by(|a, b| {
            let da = (a.0 - base_ref.0).powi(2) + (a.1 - base_ref.1).powi(2);
            let db = (b.0 - base_ref.0).powi(2) + (b.1 - base_ref.1).powi(2);
            da.partial_cmp(&db).unwrap()
        });

        for (mx, my) in candidate_minerals {
            for ang_deg in [0, 45, 90, 135, 180, 225, 270, 315] {
                let rad = (ang_deg as f32).to_radians();
                let cand_x = mx + 115.0 * rad.cos();
                let cand_y = my + 115.0 * rad.sin();

                if !(120.0..=(crate::map::MAP_WIDTH - 120.0)).contains(&cand_x)
                    || !(120.0..=(crate::map::MAP_HEIGHT - 120.0)).contains(&cand_y)
                {
                    continue;
                }
                let collides_b = engine.buildings.iter().any(|b| {
                    (cand_x - b.x).abs() < (bw + b.width) * 0.5 + 20.0
                        && (cand_y - b.y).abs() < (bh + b.height) * 0.5 + 20.0
                });
                if collides_b {
                    continue;
                }
                let collides_m = engine.minerals.iter().any(|m| {
                    ((cand_x - m.x).powi(2) + (cand_y - m.y).powi(2)).sqrt() < 85.0
                });
                if collides_m {
                    continue;
                }
                let collides_obs = engine.obstacles.iter().any(|obs| {
                    let obs_cx = obs.x + obs.width * 0.5;
                    let obs_cy = obs.y + obs.height * 0.5;
                    (cand_x - obs_cx).abs() < (bw + obs.width) * 0.5 + 15.0
                        && (cand_y - obs_cy).abs() < (bh + obs.height) * 0.5 + 15.0
                });
                if collides_obs {
                    continue;
                }

                return Some((cand_x, cand_y));
            }
        }
    }

    let mut team_ccs: Vec<&crate::components::Building> = engine
        .buildings
        .iter()
        .filter(|b| b.team == team && b.building_type == BuildingType::CommandCenter)
        .collect();

    team_ccs.sort_by(|a, b| b.id.cmp(&a.id));

    let default_pos = engine.base_pos(team);
    let newest_cc_pos = team_ccs.first().map(|b| (b.x, b.y)).unwrap_or(default_pos);

    let mut anchors = Vec::new();
    anchors.push(newest_cc_pos);

    for b in &engine.buildings {
        if b.team == team && b.building_type != BuildingType::CommandCenter {
            let dist_sq = (b.x - newest_cc_pos.0).powi(2) + (b.y - newest_cc_pos.1).powi(2);
            if dist_sq < 600.0_f32.powi(2) {
                anchors.push((b.x, b.y));
            }
        }
    }

    for cc in team_ccs.iter().skip(1) {
        anchors.push((cc.x, cc.y));
    }

    for b in &engine.buildings {
        if b.team == team && !anchors.contains(&(b.x, b.y)) {
            anchors.push((b.x, b.y));
        }
    }

    for (a_idx, anchor) in anchors.iter().enumerate() {
        let dy: f32 = (crate::map::MAP_HEIGHT * 0.5) - anchor.1;
        let dx: f32 = (crate::map::MAP_WIDTH * 0.5) - anchor.0;
        let to_center_ang = dy.atan2(dx);
        let angle_bias = match b_type {
            BuildingType::TurretTower => Some(to_center_ang + 0.1),
            BuildingType::PowerReactor => Some(to_center_ang - 0.5),
            BuildingType::Barracks => Some(to_center_ang),
            _ => Some(to_center_ang),
        };

        let mut current_radius = 80.0;
        let max_radius = if a_idx == 0 { 1400.0 } else { 550.0 };
        let radius_step = 65.0;

        while current_radius <= max_radius {
            let num_angles = ((2.0 * std::f32::consts::PI * current_radius) / 70.0).max(8.0) as usize;
            for i in 0..num_angles {
                let angle = if let Some(base_ang) = angle_bias {
                    let offset = if i % 2 == 0 {
                        (i / 2) as f32
                    } else {
                        -(i.div_ceil(2) as f32)
                    };
                    base_ang + offset * (std::f32::consts::PI * 2.0 / num_angles as f32)
                } else {
                    (i as f32) * (std::f32::consts::PI * 2.0 / num_angles as f32)
                };

                let cand_x = anchor.0 + current_radius * angle.cos();
                let cand_y = anchor.1 + current_radius * angle.sin();

                if !(120.0..=(crate::map::MAP_WIDTH - 120.0)).contains(&cand_x)
                    || !(120.0..=(crate::map::MAP_HEIGHT - 120.0)).contains(&cand_y)
                {
                    continue;
                }

                let collides_building = engine.buildings.iter().any(|b| {
                    let clearance_x = (bw + b.width) * 0.5 + 16.0;
                    let clearance_y = (bh + b.height) * 0.5 + 16.0;
                    (cand_x - b.x).abs() < clearance_x && (cand_y - b.y).abs() < clearance_y
                });
                if collides_building {
                    continue;
                }

                let collides_minerals = engine.minerals.iter().any(|m| {
                    let dist = ((cand_x - m.x).powi(2) + (cand_y - m.y).powi(2)).sqrt();
                    dist < 75.0
                });
                if collides_minerals {
                    continue;
                }

                let collides_obstacles = engine.obstacles.iter().any(|obs| {
                    let obs_cx = obs.x + obs.width * 0.5;
                    let obs_cy = obs.y + obs.height * 0.5;
                    let clearance_x = (bw + obs.width) * 0.5 + 12.0;
                    let clearance_y = (bh + obs.height) * 0.5 + 12.0;
                    (cand_x - obs_cx).abs() < clearance_x && (cand_y - obs_cy).abs() < clearance_y
                });
                if collides_obstacles {
                    continue;
                }

                let collides_enemies = engine.units.iter().any(|u| {
                    u.team != team && ((cand_x - u.x).powi(2) + (cand_y - u.y).powi(2)).sqrt() < 80.0
                });
                if collides_enemies {
                    continue;
                }

                return Some((cand_x, cand_y));
            }
            current_radius += radius_step;
        }
    }

    None
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutoTrainingArchetype {
    ScienceArtillery = 0,
    MagicCelestialVoid = 1,
    MonsterSwarmBehemoth = 2,
    BioBlitzRush = 3,
    MacroFortress = 4,
}

impl AutoTrainingArchetype {
    pub fn name(&self) -> &'static str {
        match self {
            Self::ScienceArtillery => "Science Laser & Rocket Siege",
            Self::MagicCelestialVoid => "Magic Sword & Void Cataclysm",
            Self::MonsterSwarmBehemoth => "Monster Swarm & Behemoth Titan",
            Self::BioBlitzRush => "Bio Infantry Blitz Rush",
            Self::MacroFortress => "Macro Eco & Multi-row Fortress",
        }
    }
}

pub fn create_training_autoplay_config(team: Team, archetype: AutoTrainingArchetype) -> AutoPlayConfig {
    let mut config = AutoPlayConfig::default_for_team(team);
    config.enabled = true;

    match archetype {
        AutoTrainingArchetype::ScienceArtillery => {
            config.priority_techs = vec![
                "focused_laser".to_string(),
                "energy_shielding".to_string(),
                "dark_matter_rockets".to_string(),
                "titan_alloy_plating".to_string(),
                "overload_strike".to_string(),
            ];
            config.turret_defense.rows = 2;
        }
        AutoTrainingArchetype::MagicCelestialVoid => {
            config.priority_techs = vec![
                "celestial_flying_swords".to_string(),
                "golden_bell_shield".to_string(),
                "corrosive_curse".to_string(),
                "void_cataclysm".to_string(),
            ];
        }
        AutoTrainingArchetype::MonsterSwarmBehemoth => {
            config.priority_techs = vec![
                "optical_drills".to_string(),
                "acid_banelings".to_string(),
                "hive_proliferation".to_string(),
                "primeval_scales".to_string(),
                "behemoth_mutation".to_string(),
            ];
            // Swarm tăng số thợ mỏ ban đầu
            if let Some(prod_loop) = config.loops.iter_mut().find(|l| l.id == "loop_production") {
                if let Some(w_card) = prod_loop.cards.iter_mut().find(|c| c.id == "card_prod_worker") {
                    w_card.conditions = vec![AutoCondition::UnitCountLessThan {
                        unit_type: "Engineer".to_string(),
                        count: 10,
                    }];
                }
            }
        }
        AutoTrainingArchetype::BioBlitzRush => {
            config.priority_techs = vec![
                "magnetic_rounds".to_string(),
                "exo_boosters".to_string(),
                "superheated_blades".to_string(),
            ];
            // Rush sớm với ngưỡng quân thấp hơn
            if let Some(mil_loop) = config.loops.iter_mut().find(|l| l.id == "loop_military") {
                if let Some(atk_card) = mil_loop.cards.iter_mut().find(|c| c.id == "card_mil_attack") {
                    atk_card.conditions = vec![AutoCondition::UnitCountGreaterThan {
                        unit_type: "Combat".to_string(),
                        count: 12,
                    }];
                }
            }
        }
        AutoTrainingArchetype::MacroFortress => {
            config.priority_techs = vec![
                "optical_drills".to_string(),
                "overcharged_reactors".to_string(),
                "fortified_blueprints".to_string(),
                "dark_matter_rockets".to_string(),
            ];
            config.turret_defense.rows = 2;
            config.turret_defense.spacing = 75.0;
        }
    }

    config
}

/// Tùy biến giữa trận (Mid-Match Adaptation) cho Auto:
/// Dựa theo công nghệ đã nghiên cứu xong, binh chủng mới mở khóa, thời gian trận đấu,
/// và tình hình quân địch để tự động cập nhật hoặc thêm thẻ bài vào vòng lặp Auto.
pub fn adapt_training_autoplay_mid_match(engine: &mut GameEngine, team: Team) {
    let tech = engine.team_tech(team).clone();

    let enemy_combat_count = engine
        .units
        .iter()
        .filter(|u| engine.are_hostile(team, u.team) && u.unit_type != UnitType::Engineer)
        .count();

    let my_combat_count = engine
        .units
        .iter()
        .filter(|u| u.team == team && u.unit_type != UnitType::Engineer)
        .count();

    let game_time = engine.game_time;
    let eco_minerals = engine.team_eco(team).minerals;

    let config = &mut engine.team_autoplay_mut(team).config;

    // 1. Tùy biến Vòng Lặp Sản Xuất (Loop Production)
    if let Some(prod_loop) = config.loops.iter_mut().find(|l| l.id == "loop_production") {
        // Nâng cấp thợ mỏ theo thời gian và kinh tế
        if let Some(w_card) = prod_loop.cards.iter_mut().find(|c| c.id == "card_prod_worker") {
            let target_workers = if tech.optical_drills || game_time > 150.0 {
                16
            } else if game_time > 60.0 {
                10
            } else {
                7
            };
            w_card.conditions = vec![AutoCondition::UnitCountLessThan {
                unit_type: "Engineer".to_string(),
                count: target_workers,
            }];
        }

        // Tự động thêm thẻ mở rộng CommandCenter phụ nếu có nhiều khoáng sản và đủ phát triển
        let has_expand_card = prod_loop.cards.iter().any(|c| c.id == "card_prod_expand_cc");
        if !has_expand_card && (game_time > 120.0 || eco_minerals >= 380.0) {
            prod_loop.cards.insert(
                1,
                AutoCard {
                    id: "card_prod_expand_cc".to_string(),
                    name: "Mở Rộng Căn Cứ Phụ (CommandCenter)".to_string(),
                    enabled: true,
                    conditions: vec![
                        AutoCondition::BuildingCountLessThan {
                            building_type: "CommandCenter".to_string(),
                            count: 2,
                        },
                        AutoCondition::MineralsGreaterThan { amount: 380.0 },
                    ],
                    action: AutoAction::BuildStructure {
                        building_type: "CommandCenter".to_string(),
                        count: 1,
                    },
                },
            );
        }

        // Tăng trần Barracks và Reactor khi bước vào Mid/Late Game
        if let Some(r_card) = prod_loop.cards.iter_mut().find(|c| c.id == "card_prod_reactor") {
            let max_reactors = if tech.overcharged_reactors { 8 } else { 12 };
            r_card.conditions = vec![AutoCondition::BuildingCountLessThan {
                building_type: "PowerReactor".to_string(),
                count: max_reactors,
            }];
        }
        if let Some(b_card) = prod_loop.cards.iter_mut().find(|c| c.id == "card_prod_barracks") {
            let max_barracks = if game_time > 120.0 { 8 } else { 5 };
            b_card.conditions = vec![AutoCondition::BuildingCountLessThan {
                building_type: "Barracks".to_string(),
                count: max_barracks,
            }];
        }
    }

    // 2. Tùy biến Vòng Lặp Quân Sự (Loop Military) khi mở khóa Binh Chủng Mới
    if let Some(mil_loop) = config.loops.iter_mut().find(|l| l.id == "loop_military") {
        // Danh sách các binh chủng công nghệ cao kèm điều kiện mở khóa
        let advanced_units = [
            (
                "Vanguard",
                tech.energy_shielding,
                "card_mil_vanguard",
                "Mua Vanguard (Lá Chắn Tiên Phong)",
                120.0,
            ),
            (
                "LaserMarksman",
                tech.focused_laser,
                "card_mil_laser",
                "Mua LaserMarksman (Bắn Tỉa Laser)",
                140.0,
            ),
            (
                "DarkRocketeer",
                tech.dark_matter_rockets,
                "card_mil_rocket",
                "Mua DarkRocketeer (Rocket Hắc Khoa Kỹ AoE)",
                130.0,
            ),
            (
                "TaoistSwordmaster",
                tech.celestial_flying_swords,
                "card_mil_sword",
                "Mua TaoistSwordmaster (Tu Tiên Ngự Kiếm)",
                150.0,
            ),
            (
                "VoidWarlock",
                tech.void_cataclysm,
                "card_mil_warlock",
                "Mua VoidWarlock (Phù Thủy Hư Không AoE)",
                160.0,
            ),
            (
                "AcidBaneling",
                tech.acid_banelings,
                "card_mil_baneling",
                "Mua AcidBaneling (Bọ Nổ Axit Cảm Tử)",
                50.0,
            ),
            (
                "BehemothTitan",
                tech.behemoth_mutation,
                "card_mil_titan",
                "Mua BehemothTitan (Chiến Thú Khổng Lồ)",
                200.0,
            ),
        ];

        let mut unlocked_advanced_count = 0;

        for (u_name, is_unlocked, card_id, card_name, min_threshold) in advanced_units {
            if is_unlocked {
                unlocked_advanced_count += 1;
                let exists = mil_loop.cards.iter().any(|c| c.id == card_id);
                if !exists {
                    // Chèn trước thẻ AttackWave (thường là thẻ cuối)
                    let insert_pos = mil_loop.cards.len().saturating_sub(1);
                    mil_loop.cards.insert(
                        insert_pos,
                        AutoCard {
                            id: card_id.to_string(),
                            name: card_name.to_string(),
                            enabled: true,
                            conditions: vec![AutoCondition::MineralsGreaterThan {
                                amount: min_threshold,
                            }],
                            action: AutoAction::TrainUnit {
                                unit_type: u_name.to_string(),
                                count: if tech.hive_proliferation { 2 } else { 1 },
                            },
                        },
                    );
                }
            }
        }

        // Chuyển giao cơ cấu quân đội (Transition):
        // Khi đã có binh chủng cao cấp và bước sang mid-game, hạn chế sản xuất lính cơ bản
        let should_throttle_basics = unlocked_advanced_count >= 1 && (game_time > 110.0 || my_combat_count >= 12);
        if let Some(shock_card) = mil_loop.cards.iter_mut().find(|c| c.id == "card_mil_shock") {
            if should_throttle_basics {
                shock_card.conditions = vec![
                    AutoCondition::UnitCountLessThan {
                        unit_type: "ShockTrooper".to_string(),
                        count: 6,
                    },
                    AutoCondition::MineralsGreaterThan { amount: 200.0 },
                ];
            }
        }
        if let Some(rifle_card) = mil_loop.cards.iter_mut().find(|c| c.id == "card_mil_rifle") {
            if should_throttle_basics {
                rifle_card.conditions = vec![
                    AutoCondition::UnitCountLessThan {
                        unit_type: "Rifleman".to_string(),
                        count: 6,
                    },
                    AutoCondition::MineralsGreaterThan { amount: 200.0 },
                ];
            }
        }

        // Tùy biến ngưỡng Tổng Tấn Công (AttackWave):
        // Nếu bị đối phương dồn ép hoặc đã có lượng quân áp đảo -> điều chỉnh ngưỡng tấn công
        if let Some(atk_card) = mil_loop.cards.iter_mut().find(|c| c.id == "card_mil_attack") {
            let required_units = if enemy_combat_count >= 18 {
                // Địch quá đông, phải gom đủ lực lượng phản kích hoặc xả quân giữ nhà
                16
            } else if tech.dark_matter_rockets || tech.focused_laser || tech.behemoth_mutation {
                // Đã có binh chủng công thành hạng nặng -> tấn công áp đảo sớm hơn
                15
            } else {
                20
            };
            atk_card.conditions = vec![AutoCondition::UnitCountGreaterThan {
                unit_type: "Combat".to_string(),
                count: required_units,
            }];
        }
    }

    // 3. Tùy biến Phòng Tuyến Tháp Pháo (Turret Defense)
    if enemy_combat_count >= 14 && config.turret_defense.rows < 2 {
        config.turret_defense.rows = 2;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_autoplay_config() {
        let config = AutoPlayConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.loops.len(), 2);
        assert_eq!(config.loops[0].name, "Sản Xuất");
        assert_eq!(config.loops[0].cards.len(), 7);
        assert_eq!(config.loops[1].name, "Quân Sự");
        assert_eq!(config.loops[1].cards.len(), 4);
    }

    #[test]
    fn test_can_train_unit_tech_requirements() {
        let mut tech = TechTreeState::default();
        assert!(can_train_unit(&tech, UnitType::Engineer));
        assert!(can_train_unit(&tech, UnitType::ShockTrooper));
        assert!(can_train_unit(&tech, UnitType::Rifleman));
        assert!(!can_train_unit(&tech, UnitType::Vanguard));

        tech.energy_shielding = true;
        assert!(can_train_unit(&tech, UnitType::Vanguard));
    }

    #[test]
    fn test_converter_condition() {
        let cond = AutoCondition::ReactorPerConverterRatio;
        // Kiểm tra đúng định dạng enum
        assert_eq!(cond, AutoCondition::ReactorPerConverterRatio);
    }

    #[test]
    fn test_compute_defense_tower_spots() {
        let line = TurretDefenseLine {
            enabled: true,
            p1: (0.0, 0.0),
            p2: (300.0, 0.0),
            spacing: 100.0,
            rows: 2,
            row_spacing: 50.0,
        };
        let spots = compute_defense_tower_spots(&line);
        // Hàng 1: x = 0, 100, 200, 300 (4 tháp)
        // Hàng 2: x = 0, 100, 200, 300 (4 tháp)
        // Tổng: 8 tháp
        assert_eq!(spots.len(), 8);
        assert_eq!(spots[0], (0.0, 0.0));
        assert_eq!(spots[1], (100.0, 0.0));
        assert_eq!(spots[2], (200.0, 0.0));
        assert_eq!(spots[3], (300.0, 0.0));
    }

    #[test]
    fn test_parse_building_and_tech_prerequisite() {
        assert_eq!(parse_building_type("PowerReactor"), Some(BuildingType::PowerReactor));
        assert_eq!(parse_building_type("TurretTower"), Some(BuildingType::TurretTower));
        assert_eq!(parse_building_type("Barracks"), Some(BuildingType::Barracks));
        assert_eq!(parse_building_type("EnergyConverter"), Some(BuildingType::EnergyConverter));
        assert_eq!(get_tech_prerequisite("titan_alloy_plating"), Some("dark_matter_rockets".to_string()));
    }

    #[test]
    fn test_create_and_adapt_training_autoplay() {
        let mut engine = GameEngine::new_headless();
        let config = create_training_autoplay_config(Team::Player, AutoTrainingArchetype::ScienceArtillery);
        assert!(config.enabled);
        assert_eq!(config.priority_techs.len(), 5);
        assert_eq!(config.turret_defense.rows, 2);

        engine.autoplay.config = config;
        // Giả lập mở khóa công nghệ
        engine.player_tech.energy_shielding = true;
        engine.player_tech.focused_laser = true;
        engine.game_time = 120.0;

        adapt_training_autoplay_mid_match(&mut engine, Team::Player);

        let mil_loop = engine.autoplay.config.loops.iter().find(|l| l.id == "loop_military").unwrap();
        // Kiểm tra đã tự động chèn thẻ mua Vanguard và LaserMarksman
        assert!(mil_loop.cards.iter().any(|c| c.id == "card_mil_vanguard"));
        assert!(mil_loop.cards.iter().any(|c| c.id == "card_mil_laser"));
    }
}
