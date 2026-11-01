use crate::components::{BuildingType, Team, UnitOrder, UnitType};
use crate::GameEngine;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum AutoCondition {
    Always,
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
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AutoCard {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub condition: AutoCondition,
    pub action: AutoAction,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AutoLoop {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub current_card_index: usize,
    pub current_card_progress: u32,
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

impl Default for TurretDefenseLine {
    fn default() -> Self {
        // Vị trí mặc định: Tuyến phòng thủ xéo chắn giữa căn cứ Player (250, 2050) và Enemy (2150, 350)
        // Cắt vuông góc trục Tây Nam -> Đông Bắc
        Self {
            enabled: true,
            p1: (520.0, 1680.0),
            p2: (860.0, 1980.0),
            spacing: 90.0,
            rows: 1,
            row_spacing: 80.0,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AutoPlayConfig {
    pub enabled: bool,
    pub auto_supply: bool,
    pub priority_techs: Vec<String>,
    #[serde(default)]
    pub turret_defense: TurretDefenseLine,
    pub loops: Vec<AutoLoop>,
}

impl Default for AutoPlayConfig {
    fn default() -> Self {
        let turret_defense = TurretDefenseLine::default();
        // Loop 1: Sản xuất (Production)
        let production_loop = AutoLoop {
            id: "loop_production".to_string(),
            name: "Sản Xuất".to_string(),
            enabled: true,
            current_card_index: 0,
            current_card_progress: 0,
            cards: vec![
                AutoCard {
                    id: "card_prod_worker".to_string(),
                    name: "Mua 2 Thợ Mỏ".to_string(),
                    enabled: true,
                    condition: AutoCondition::UnitCountLessThan {
                        unit_type: "Engineer".to_string(),
                        count: 7,
                    },
                    action: AutoAction::TrainUnit {
                        unit_type: "Engineer".to_string(),
                        count: 2,
                    },
                },
                AutoCard {
                    id: "card_prod_reactor".to_string(),
                    name: "Xây 2 PowerReactor".to_string(),
                    enabled: true,
                    condition: AutoCondition::Always,
                    action: AutoAction::BuildStructure {
                        building_type: "PowerReactor".to_string(),
                        count: 2,
                    },
                },
                AutoCard {
                    id: "card_prod_turret".to_string(),
                    name: "Xây 2 TurretTower".to_string(),
                    enabled: true,
                    condition: AutoCondition::Always,
                    action: AutoAction::BuildStructure {
                        building_type: "TurretTower".to_string(),
                        count: 2,
                    },
                },
                AutoCard {
                    id: "card_prod_tech".to_string(),
                    name: "Nâng cấp Công Nghệ".to_string(),
                    enabled: true,
                    condition: AutoCondition::Always,
                    action: AutoAction::ResearchTech {
                        tech_id: "dark_matter_rockets".to_string(),
                        use_priority_queue: true,
                    },
                },
                AutoCard {
                    id: "card_prod_barracks".to_string(),
                    name: "Xây 1 Barracks".to_string(),
                    enabled: true,
                    condition: AutoCondition::Always,
                    action: AutoAction::BuildStructure {
                        building_type: "Barracks".to_string(),
                        count: 1,
                    },
                },
                AutoCard {
                    id: "card_prod_converter".to_string(),
                    name: "Xây EnergyConverter (Tỷ lệ 4:1)".to_string(),
                    enabled: true,
                    condition: AutoCondition::ReactorPerConverterRatio,
                    action: AutoAction::BuildStructure {
                        building_type: "EnergyConverter".to_string(),
                        count: 1,
                    },
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
            cards: vec![
                AutoCard {
                    id: "card_mil_shock".to_string(),
                    name: "Mua ShockTrooper (>500 Khoáng)".to_string(),
                    enabled: true,
                    condition: AutoCondition::MineralsGreaterThan { amount: 500.0 },
                    action: AutoAction::TrainUnit {
                        unit_type: "ShockTrooper".to_string(),
                        count: 1,
                    },
                },
                AutoCard {
                    id: "card_mil_rifle".to_string(),
                    name: "Mua Rifleman (>500 Khoáng)".to_string(),
                    enabled: true,
                    condition: AutoCondition::MineralsGreaterThan { amount: 500.0 },
                    action: AutoAction::TrainUnit {
                        unit_type: "Rifleman".to_string(),
                        count: 1,
                    },
                },
                AutoCard {
                    id: "card_mil_attack".to_string(),
                    name: "Tự động tấn công (>40 Lính)".to_string(),
                    enabled: true,
                    condition: AutoCondition::UnitCountGreaterThan {
                        unit_type: "Combat".to_string(),
                        count: 40,
                    },
                    action: AutoAction::AttackWave,
                },
            ],
        };

        Self {
            enabled: false,
            auto_supply: true,
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

pub fn update_autoplay(engine: &mut GameEngine, dt: f32) {
    if !engine.autoplay.config.enabled {
        return;
    }

    engine.autoplay.timer += dt;
    if engine.autoplay.timer < 0.3 {
        return;
    }
    engine.autoplay.timer = 0.0;

    // 1. Tự động tránh kẹt dân số (Auto Supply Depot) nếu bật
    if engine.autoplay.config.auto_supply {
        check_and_build_supply(engine);
    }

    // 2. Chạy từng loop
    let num_loops = engine.autoplay.config.loops.len();
    for loop_idx in 0..num_loops {
        if !engine.autoplay.config.loops[loop_idx].enabled {
            continue;
        }
        step_loop(engine, loop_idx);
    }
}

fn check_and_build_supply(engine: &mut GameEngine) {
    if engine.player_eco.max_supply >= 200 {
        return;
    }
    let depot_under_construction = engine
        .buildings
        .iter()
        .filter(|b| {
            b.team == Team::Player
                && b.building_type == BuildingType::SupplyDepot
                && b.build_progress < 1.0
        })
        .count();

    if depot_under_construction >= 2 {
        return;
    }

    let need_depot = (engine.player_eco.supply + 2 >= engine.player_eco.max_supply)
        || (engine.player_eco.minerals >= 350.0
            && engine.player_eco.supply + 6 >= engine.player_eco.max_supply);

    let (cost_m, cost_e) = engine.config.get_building_cost(BuildingType::SupplyDepot);
    if need_depot
        && engine.player_eco.minerals >= cost_m
        && engine.player_eco.energy >= cost_e
    {
        let p_cc_pos = engine
            .buildings
            .iter()
            .find(|b| b.team == Team::Player && b.building_type == BuildingType::CommandCenter)
            .map(|b| (b.x, b.y))
            .unwrap_or((250.0, 2050.0));

        if let Some((bx, by)) = crate::ai::find_ai_build_spot(
            engine,
            BuildingType::SupplyDepot,
            p_cc_pos.0,
            p_cc_pos.1,
            550.0,
            Some(-0.8), // hướng Đông Bắc từ căn cứ người chơi
        ) {
            try_player_auto_build(engine, BuildingType::SupplyDepot, bx, by);
        }
    }
}

fn step_loop(engine: &mut GameEngine, loop_idx: usize) {
    let loop_data = &engine.autoplay.config.loops[loop_idx];
    if loop_data.cards.is_empty() {
        return;
    }

    let current_idx = loop_data.current_card_index % loop_data.cards.len();
    let card = loop_data.cards[current_idx].clone();

    if !card.enabled {
        // Skip card này
        advance_loop_card(engine, loop_idx);
        return;
    }

    // Kiểm tra điều kiện
    let condition_met = evaluate_condition(engine, &card.condition);
    if !condition_met {
        // Nếu điều kiện không đạt, bỏ qua thẻ để chuyển sang thẻ tiếp theo
        advance_loop_card(engine, loop_idx);
        return;
    }

    // Thực hiện hành động của thẻ
    let action_completed = execute_action(engine, loop_idx, &card.action);
    if action_completed {
        advance_loop_card(engine, loop_idx);
    }
}

fn advance_loop_card(engine: &mut GameEngine, loop_idx: usize) {
    let loop_data = &mut engine.autoplay.config.loops[loop_idx];
    loop_data.current_card_progress = 0;
    if !loop_data.cards.is_empty() {
        loop_data.current_card_index = (loop_data.current_card_index + 1) % loop_data.cards.len();
    }
}

fn evaluate_condition(engine: &GameEngine, condition: &AutoCondition) -> bool {
    match condition {
        AutoCondition::Always => true,
        AutoCondition::UnitCountLessThan { unit_type, count } => {
            let actual = count_player_units(engine, unit_type);
            actual < *count
        }
        AutoCondition::UnitCountGreaterThan { unit_type, count } => {
            let actual = count_player_units(engine, unit_type);
            actual > *count
        }
        AutoCondition::MineralsGreaterThan { amount } => engine.player_eco.minerals > *amount,
        AutoCondition::EnergyGreaterThan { amount } => engine.player_eco.energy > *amount,
        AutoCondition::ReactorPerConverterRatio => {
            let reactors = engine
                .buildings
                .iter()
                .filter(|b| {
                    b.team == Team::Player
                        && b.building_type == BuildingType::PowerReactor
                        && b.build_progress >= 1.0
                })
                .count();
            let converters = engine
                .buildings
                .iter()
                .filter(|b| {
                    b.team == Team::Player && b.building_type == BuildingType::EnergyConverter
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
                    .filter(|b| b.team == Team::Player && b.building_type == bt)
                    .count();
                actual < *count
            } else {
                false
            }
        }
    }
}

fn execute_action(engine: &mut GameEngine, loop_idx: usize, action: &AutoAction) -> bool {
    match action {
        AutoAction::TrainUnit { unit_type, count } => {
            let u_type = parse_unit_type(unit_type);
            let u_type = match u_type {
                Some(t) => t,
                None => return true, // Loại unit không hợp lệ, bỏ qua
            };

            // Nếu kịch trần dân số, chờ có supply hoặc xây supply
            if engine.player_eco.supply >= engine.player_eco.max_supply {
                return false;
            }

            let (min_cost, en_cost) = engine.config.get_unit_cost(u_type);
            if engine.player_eco.minerals < min_cost || engine.player_eco.energy < en_cost {
                return false; // Chờ đủ tiền
            }

            // Tìm nhà phù hợp để train
            let target_btype = match u_type {
                UnitType::Engineer => BuildingType::CommandCenter,
                _ => BuildingType::Barracks,
            };

            let maybe_b_id = engine
                .buildings
                .iter()
                .filter(|b| {
                    b.team == Team::Player
                        && b.building_type == target_btype
                        && b.build_progress >= 1.0
                        && b.training_queue.len() < 5
                })
                .min_by_key(|b| b.training_queue.len())
                .map(|b| b.id);

            if let Some(bid) = maybe_b_id {
                if let Some(b) = engine.buildings.iter_mut().find(|b| b.id == bid) {
                    engine.player_eco.minerals -= min_cost;
                    engine.player_eco.energy -= en_cost;
                    b.training_queue.push(u_type);

                    let progress = engine.autoplay.config.loops[loop_idx].current_card_progress + 1;
                    engine.autoplay.config.loops[loop_idx].current_card_progress = progress;
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
            if engine.player_eco.minerals < min_cost || engine.player_eco.energy < en_cost {
                return false; // Chờ đủ tiền
            }

            let p_cc_pos = engine
                .buildings
                .iter()
                .find(|b| b.team == Team::Player && b.building_type == BuildingType::CommandCenter)
                .map(|b| (b.x, b.y))
                .unwrap_or((250.0, 2050.0));

            let angle_bias = match b_type {
                BuildingType::TurretTower => Some(-0.4), // Hướng Đông Bắc phòng thủ
                BuildingType::PowerReactor => Some(-1.2),
                BuildingType::Barracks => Some(-0.6),
                _ => None,
            };

            let mut spot = None;
            if b_type == BuildingType::TurretTower && engine.autoplay.config.turret_defense.enabled {
                spot = find_defense_line_spot(engine, &engine.autoplay.config.turret_defense);
            }

            if spot.is_none() {
                spot = crate::ai::find_ai_build_spot(
                    engine,
                    b_type,
                    p_cc_pos.0,
                    p_cc_pos.1,
                    550.0,
                    angle_bias,
                );
            }

            if let Some((bx, by)) = spot {
                if try_player_auto_build(engine, b_type, bx, by) {
                    let progress =
                        engine.autoplay.config.loops[loop_idx].current_card_progress + 1;
                    engine.autoplay.config.loops[loop_idx].current_card_progress = progress;
                    if progress >= *count {
                        return true;
                    }
                }
            }
            false
        }
        AutoAction::ResearchTech {
            tech_id,
            use_priority_queue,
        } => {
            let mut target_tech = tech_id.clone();

            // Nếu bật hàng đợi ưu tiên, kiểm tra xem có tech ưu tiên nào chưa unlock không
            if *use_priority_queue {
                let priority_list = engine.autoplay.config.priority_techs.clone();
                for p_tech in priority_list {
                    if !crate::simulation::is_tech_unlocked(&engine.player_tech, &p_tech)
                        && !engine
                            .player_tech
                            .active_researches
                            .iter()
                            .any(|r| r.tech_id == p_tech)
                    {
                        // Kiểm tra tech này có yêu cầu tiên quyết requires không
                        if let Some(req) = get_tech_prerequisite(&p_tech) {
                            if !crate::simulation::is_tech_unlocked(&engine.player_tech, &req) {
                                target_tech = req;
                                break;
                            }
                        }
                        target_tech = p_tech;
                        break;
                    }
                }
            }

            // Kiểm tra tech mục tiêu đã mở hoặc đang chạy chưa
            if crate::simulation::is_tech_unlocked(&engine.player_tech, &target_tech) {
                return true; // Đã xong, qua card kế tiếp
            }
            if engine
                .player_tech
                .active_researches
                .iter()
                .any(|r| r.tech_id == target_tech)
            {
                return true; // Đang chạy rồi, qua card kế tiếp
            }

            // Thử khởi động research
            if crate::simulation::process_start_research(
                &mut engine.player_tech,
                &mut engine.player_eco,
                &engine.config,
                &target_tech,
            ) {
                engine.outgoing_events.push(crate::bridge::EngineToReactEvent::PlaySound {
                    sound: "research_start".to_string(),
                });
                return true;
            }

            false // Chờ đủ tiền hoặc nhánh rảnh
        }
        AutoAction::AttackWave => {
            // Ra lệnh tấn công toàn bộ lính chiến đấu Player
            let enemy_pos = engine
                .buildings
                .iter()
                .find(|b| b.team == Team::Enemy && b.building_type == BuildingType::CommandCenter)
                .map(|b| (b.x, b.y))
                .or_else(|| {
                    engine
                        .buildings
                        .iter()
                        .find(|b| b.team == Team::Enemy)
                        .map(|b| (b.x, b.y))
                })
                .or_else(|| {
                    engine
                        .units
                        .iter()
                        .find(|u| u.team == Team::Enemy)
                        .map(|u| (u.x, u.y))
                })
                .unwrap_or((2150.0, 350.0));

            let combat_units: Vec<u32> = engine
                .units
                .iter()
                .filter(|u| u.team == Team::Player && u.unit_type != UnitType::Engineer)
                .map(|u| u.id)
                .collect();

            for (i, uid) in combat_units.iter().enumerate() {
                if let Some(u) = engine.units.iter_mut().find(|u| u.id == *uid) {
                    let offset_x = ((i % 6) as f32 - 2.5) * 25.0;
                    let offset_y = ((i / 6) as f32) * 25.0;
                    u.order = UnitOrder::AttackMove {
                        x: enemy_pos.0 + offset_x,
                        y: enemy_pos.1 + offset_y,
                    };
                }
            }

            engine.outgoing_events.push(crate::bridge::EngineToReactEvent::PlaySound {
                sound: "attack_order".to_string(),
            });
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
    let (cost_min, cost_en) = engine.config.get_building_cost(b_type);
    if engine.player_eco.minerals < cost_min || engine.player_eco.energy < cost_en {
        return false;
    }

    engine.player_eco.minerals -= cost_min;
    engine.player_eco.energy -= cost_en;
    let bid = engine.alloc_id();

    let mut building = engine
        .config
        .create_building(bid, Team::Player, b_type, wx, wy, false);
    building.build_progress = 0.05;
    building.hp = 1.0;
    building.rally_x = wx + 40.0;
    building.rally_y = wy + 40.0;

    if engine.player_tech.titan_alloy_plating {
        building.resistances.physical = (building.resistances.physical + 0.20).min(0.80);
    }
    let mut hp_mult = 1.0;
    if engine.player_tech.fortified_blueprints {
        hp_mult += 0.3;
    }
    if engine.player_tech.titan_alloy_plating {
        hp_mult += 0.25;
    }
    building.max_hp *= hp_mult;

    let is_armed = engine
        .config
        .get_building(b_type)
        .map(|c| c.turret_damage > 0.0)
        .unwrap_or(false);
    if is_armed && engine.player_tech.dark_matter_rockets {
        building.damage_type = crate::components::DamageType::Explosive;
    }

    engine.buildings.push(building);

    // Tìm 1 thợ mỏ rảnh hoặc gần nhất để phân công xây
    let nearest_worker_id = engine
        .units
        .iter()
        .filter(|u| u.team == Team::Player && u.unit_type == UnitType::Engineer)
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

    engine.outgoing_events.push(crate::bridge::EngineToReactEvent::PlaySound {
        sound: "place_building".to_string(),
    });

    true
}

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
    let spots = compute_defense_tower_spots(defense_line);
    let (bw, bh) = engine
        .config
        .get_building(BuildingType::TurretTower)
        .map(|c| (c.width, c.height))
        .unwrap_or((60.0, 60.0));

    for (sx, sy) in spots {
        // Kiểm tra xem vị trí này đã có TurretTower của Player chưa
        let already_has_turret = engine.buildings.iter().any(|b| {
            b.team == Team::Player
                && b.building_type == BuildingType::TurretTower
                && (b.x - sx).powi(2) + (b.y - sy).powi(2) < 45.0_f32.powi(2)
        });
        if already_has_turret {
            continue;
        }

        // Kiểm tra va chạm với các công trình khác
        let collides_building = engine.buildings.iter().any(|b| {
            let clearance_x = (bw + b.width) * 0.5 + 15.0;
            let clearance_y = (bh + b.height) * 0.5 + 15.0;
            (sx - b.x).abs() < clearance_x && (sy - b.y).abs() < clearance_y
        });
        if collides_building {
            continue;
        }

        // Kiểm tra va chạm với mỏ quặng
        let collides_minerals = engine.minerals.iter().any(|m| {
            let dist = ((sx - m.x).powi(2) + (sy - m.y).powi(2)).sqrt();
            dist < 75.0
        });
        if collides_minerals {
            continue;
        }

        // Kiểm tra va chạm với chướng ngại vật
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

        // Kiểm tra biên giới bản đồ
        if !(120.0..=2280.0).contains(&sx) || !(120.0..=2280.0).contains(&sy) {
            continue;
        }

        return Some((sx, sy));
    }

    None
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
        assert_eq!(config.loops[0].cards.len(), 6);
        assert_eq!(config.loops[1].name, "Quân Sự");
        assert_eq!(config.loops[1].cards.len(), 3);
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
}
