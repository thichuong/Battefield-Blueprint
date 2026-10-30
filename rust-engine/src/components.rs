use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};

#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Team {
    Player = 0,
    Enemy = 1,
}

#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnitType {
    Engineer = 0,     // Khai thác, xây dựng, cờ lê/máy hàn
    ShockTrooper = 1, // Kiếm sĩ xung kích (Plasma Blade)
    Rifleman = 2,     // Xạ thủ bắn súng trường
    Vanguard = 3,     // Vệ binh khiên chắn năng lượng
    LaserMarksman = 4,// Pháo thủ Laser tầm xa
}

#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BuildingType {
    CommandCenter = 0, // Nhà chính (Nhận tài nguyên, xin Kỹ sư)
    SupplyDepot = 1,   // Trạm tiếp vận (+Pop cap)
    PowerReactor = 2,  // Lò phản ứng (+Energy/s)
    Barracks = 3,      // Trại huấn luyện lính
    TurretTower = 4,   // Tháp pháo phòng thủ tự động
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum UnitOrder {
    Idle,
    MoveTo { x: f32, y: f32 },
    AttackMove { x: f32, y: f32 },
    Harvest { target_mineral_id: u32, phase: HarvestPhase },
    Build { target_building_id: u32 },
    AttackTarget { target_id: u32, is_building: bool },
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum HarvestPhase {
    MovingToNode,
    Mining(f32),
    ReturningToBase,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Unit {
    pub id: u32,
    pub team: Team,
    pub unit_type: UnitType,
    pub x: f32,
    pub y: f32,
    pub target_x: f32,
    pub target_y: f32,
    pub speed: f32,
    pub hp: f32,
    pub max_hp: f32,
    pub attack_range: f32,
    pub attack_damage: f32,
    pub attack_cooldown: f32,
    pub current_cooldown: f32,
    pub walk_cycle: f32,
    pub facing_right: bool,
    pub order: UnitOrder,
    pub carried_minerals: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Building {
    pub id: u32,
    pub team: Team,
    pub building_type: BuildingType,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub hp: f32,
    pub max_hp: f32,
    pub build_progress: f32, // 0.0 -> 1.0 (1.0 là hoàn tất)
    pub attack_cooldown: f32,
    pub current_cooldown: f32,
    pub rally_x: f32,
    pub rally_y: f32,
    // Hàng đợi sản xuất
    pub training_queue: Vec<UnitType>,
    pub training_timer: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MineralNode {
    pub id: u32,
    pub x: f32,
    pub y: f32,
    pub amount: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Projectile {
    pub id: u32,
    pub team: Team,
    pub x: f32,
    pub y: f32,
    pub target_x: f32,
    pub target_y: f32,
    pub speed: f32,
    pub damage: f32,
    pub is_laser: bool,
    pub lifetime: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Obstacle {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FloatingText {
    pub id: u32,
    pub text: String,
    pub x: f32,
    pub y: f32,
    pub color: String,
    pub lifetime: f32,
    pub max_lifetime: f32,
}
