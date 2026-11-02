use crate::components::{FormationType, Team};
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
    AttackWave {
        #[serde(default)]
        formation: Option<String>,
    },
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
    pub p1: (f32, f32),   // Điểm bắt đầu
    pub p2: (f32, f32),   // Điểm kết thúc
    pub spacing: f32,     // Khoảng cách giữa các tháp (mặc định 90.0)
    pub rows: u32,        // Số hàng tháp (mặc định 1)
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
    pub preferred_formation: FormationType,
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
        let base = crate::map::get_spawn_positions()
            .get(team.index())
            .copied()
            .unwrap_or((550.0, 4250.0));
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
                    id: "card_prod_supply".to_string(),
                    name: "Xây SupplyDepot tránh kẹt".to_string(),
                    enabled: true,
                    conditions: vec![AutoCondition::SupplyNearCap],
                    action: AutoAction::BuildStructure {
                        building_type: "SupplyDepot".to_string(),
                        count: 1,
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
                        count: 12,
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
                    id: "card_prod_shock".to_string(),
                    name: "Mua ShockTrooper (>150 Khoáng)".to_string(),
                    enabled: true,
                    conditions: vec![AutoCondition::MineralsGreaterThan { amount: 150.0 }],
                    action: AutoAction::TrainUnit {
                        unit_type: "ShockTrooper".to_string(),
                        count: 1,
                    },
                },
                AutoCard {
                    id: "card_prod_rifle".to_string(),
                    name: "Mua Rifleman (>150 Khoáng)".to_string(),
                    enabled: true,
                    conditions: vec![AutoCondition::MineralsGreaterThan { amount: 150.0 }],
                    action: AutoAction::TrainUnit {
                        unit_type: "Rifleman".to_string(),
                        count: 1,
                    },
                },
                AutoCard {
                    id: "card_prod_delay".to_string(),
                    name: "Tạm nghỉ sản xuất (Delay 2s)".to_string(),
                    enabled: true,
                    conditions: vec![AutoCondition::Always],
                    action: AutoAction::Delay { seconds: 2.0 },
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
                    id: "card_mil_attack".to_string(),
                    name: "Tự động tấn công (>20 Lính)".to_string(),
                    enabled: true,
                    conditions: vec![AutoCondition::UnitCountGreaterThan {
                        unit_type: "Combat".to_string(),
                        count: 20,
                    }],
                    action: AutoAction::AttackWave { formation: None },
                },
                AutoCard {
                    id: "card_mil_delay".to_string(),
                    name: "Nghỉ lệnh hành quân (Delay 2s)".to_string(),
                    enabled: true,
                    conditions: vec![AutoCondition::Always],
                    action: AutoAction::Delay { seconds: 2.0 },
                },
            ],
        };

        Self {
            enabled: false,
            priority_techs: Vec::new(),
            preferred_formation: FormationType::Line,
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
