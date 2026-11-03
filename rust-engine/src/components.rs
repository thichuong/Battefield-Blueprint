use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

pub const MAX_PLAYERS: usize = 8;

#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Team {
    Player = 0,
    Enemy = 1,
    Player2 = 2,
    Player3 = 3,
    Player4 = 4,
    Player5 = 5,
    Player6 = 6,
    Player7 = 7,
}

impl Team {
    pub const ALL: [Team; MAX_PLAYERS] = [
        Team::Player,
        Team::Enemy,
        Team::Player2,
        Team::Player3,
        Team::Player4,
        Team::Player5,
        Team::Player6,
        Team::Player7,
    ];

    #[inline]
    pub fn index(self) -> usize {
        self as usize
    }

    #[inline]
    pub fn from_index(index: usize) -> Self {
        match index {
            0 => Team::Player,
            1 => Team::Enemy,
            2 => Team::Player2,
            3 => Team::Player3,
            4 => Team::Player4,
            5 => Team::Player5,
            6 => Team::Player6,
            _ => Team::Player7,
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct PlayerSlotConfig {
    pub slot_index: usize, // 0..7
    pub active: bool,      // Có tham gia trận đấu không
    pub faction: u8,       // ID Phe (0 = Phe Player/Đồng minh, 1..7 = Các phe khác)
    pub is_human: bool,    // true cho slot 0
}

#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DamageType {
    Physical = 0,
    Energy = 1,
    Explosive = 2,
    Arcane = 3,
}

impl DamageType {
    pub fn display_name(&self) -> &'static str {
        match self {
            DamageType::Physical => "PHYSICAL",
            DamageType::Energy => "ENERGY",
            DamageType::Explosive => "EXPLOSIVE",
            DamageType::Arcane => "ARCANE",
        }
    }

    pub fn color_code(&self) -> &'static str {
        match self {
            DamageType::Physical => "#f8fafc",
            DamageType::Energy => "#38bdf8",
            DamageType::Explosive => "#fb923c",
            DamageType::Arcane => "#c084fc",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Resistances {
    pub physical: f32, // 0.0 -> 0.8 (ví dụ 0.2 là giảm 20% sát thương)
    pub energy: f32,
    pub explosive: f32,
    pub arcane: f32,
}

impl Default for Resistances {
    fn default() -> Self {
        Self {
            physical: 0.0,
            energy: 0.0,
            explosive: 0.0,
            arcane: 0.0,
        }
    }
}

#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnitType {
    Engineer = 0,          // Khai thác, xây dựng, cờ lê/máy hàn
    ShockTrooper = 1,      // Kiếm sĩ xung kích (Plasma Blade)
    Rifleman = 2,          // Xạ thủ bắn súng trường
    Vanguard = 3,          // Vệ binh khiên chắn năng lượng
    LaserMarksman = 4,     // Pháo thủ Laser tầm xa
    DarkRocketeer = 5,     // Khoa học - Hắc khoa kỹ: Lính bệ phóng rocket nổ AoE
    TaoistSwordmaster = 6, // Ma thuật - Tiên pháp: Tu tiên giả ngự phi kiếm chân khí
    VoidWarlock = 7,       // Ma thuật - Hắc ám: Vu sư hư không, cầu ma thuật bộc phá
    BehemothTitan = 8,     // Quái vật - Khổng lồ: Chiến thú titan dậm chấn động
    AcidBaneling = 9,      // Quái vật - Bọ: Bọ nổ axit cảm tử AoE
}

impl UnitType {
    pub fn training_time(&self) -> f32 {
        match self {
            UnitType::Engineer => 5.0,
            UnitType::ShockTrooper => 7.0,
            UnitType::Rifleman => 8.0,
            UnitType::Vanguard => 12.0,
            UnitType::LaserMarksman => 15.0,
            UnitType::DarkRocketeer => 14.0,
            UnitType::TaoistSwordmaster => 15.0,
            UnitType::VoidWarlock => 16.0,
            UnitType::BehemothTitan => 20.0,
            UnitType::AcidBaneling => 6.0,
        }
    }

    pub fn cost(&self) -> (f32, f32) {
        match self {
            UnitType::Engineer => (50.0, 0.0),
            UnitType::ShockTrooper => (75.0, 0.0),
            UnitType::Rifleman => (100.0, 20.0),
            UnitType::Vanguard => (150.0, 50.0),
            UnitType::LaserMarksman => (175.0, 100.0),
            UnitType::DarkRocketeer => (160.0, 80.0),
            UnitType::TaoistSwordmaster => (180.0, 90.0),
            UnitType::VoidWarlock => (200.0, 120.0),
            UnitType::BehemothTitan => (250.0, 100.0),
            UnitType::AcidBaneling => (60.0, 30.0),
        }
    }

    pub fn button_label(&self) -> &'static str {
        match self {
            UnitType::Engineer => "WORKER",
            UnitType::ShockTrooper => "STRIKER",
            UnitType::Rifleman => "RIFLE",
            UnitType::Vanguard => "SHIELD",
            UnitType::LaserMarksman => "LASER",
            UnitType::DarkRocketeer => "ROCKET",
            UnitType::TaoistSwordmaster => "SWORD",
            UnitType::VoidWarlock => "WARLOCK",
            UnitType::BehemothTitan => "TITAN",
            UnitType::AcidBaneling => "BANELING",
        }
    }

    pub fn cost_label(&self) -> &'static str {
        match self {
            UnitType::Engineer => "50M",
            UnitType::ShockTrooper => "75M",
            UnitType::Rifleman => "100M 20E",
            UnitType::Vanguard => "150M 50E",
            UnitType::LaserMarksman => "175M 100E",
            UnitType::DarkRocketeer => "160M 80E",
            UnitType::TaoistSwordmaster => "180M 90E",
            UnitType::VoidWarlock => "200M 120E",
            UnitType::BehemothTitan => "250M 100E",
            UnitType::AcidBaneling => "60M 30E",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            UnitType::Engineer => "ENGINEER // WORKER",
            UnitType::ShockTrooper => "STRIKER // SHOCK TROOPER",
            UnitType::Rifleman => "MARKSMAN // RIFLEMAN",
            UnitType::Vanguard => "VANGUARD // SHIELDER",
            UnitType::LaserMarksman => "SIEGE // LASER MARKSMAN",
            UnitType::DarkRocketeer => "DARK TECH // ROCKETEER",
            UnitType::TaoistSwordmaster => "CELESTIAL // SWORDMASTER",
            UnitType::VoidWarlock => "VOID // SHADOW WARLOCK",
            UnitType::BehemothTitan => "BEHEMOTH // TITAN",
            UnitType::AcidBaneling => "SWARM // ACID BANELING",
        }
    }

    pub fn formation_role_tier(&self) -> u8 {
        match self {
            // Tier 0: Tuyến đầu / Chống chịu / Tiên phong / Cảm tử
            UnitType::Vanguard
            | UnitType::BehemothTitan
            | UnitType::ShockTrooper
            | UnitType::AcidBaneling => 0,
            // Tier 1: Trung tuyến / Cận chiến tầm trung / Xạ thủ
            UnitType::Rifleman | UnitType::TaoistSwordmaster => 1,
            // Tier 2: Hậu tuyến / Pháo binh / Pháp sư / Thợ mỏ
            UnitType::LaserMarksman
            | UnitType::DarkRocketeer
            | UnitType::VoidWarlock
            | UnitType::Engineer => 2,
        }
    }
}

#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum FormationType {
    #[default]
    Line = 0, // Hàng Ngang (Phalanx)
    Wedge = 1,   // Mũi Tên (Spearhead V)
    Box = 2,     // Ô Vuông Hộ Vệ (Protect Square)
    Scatter = 3, // Phân Tán (Chống AoE)
    Circle = 4,  // Vòng Cung Vây Bọc (Crescent Arc)
    Column = 5,  // Cột Dọc Đột Kích (Blitz Column)
}

impl FormationType {
    pub const ALL: [FormationType; 6] = [
        FormationType::Line,
        FormationType::Wedge,
        FormationType::Box,
        FormationType::Scatter,
        FormationType::Circle,
        FormationType::Column,
    ];

    pub fn from_index(idx: usize) -> Self {
        match idx % 6 {
            0 => FormationType::Line,
            1 => FormationType::Wedge,
            2 => FormationType::Box,
            3 => FormationType::Scatter,
            4 => FormationType::Circle,
            _ => FormationType::Column,
        }
    }

    pub fn to_index(&self) -> usize {
        *self as usize
    }

    pub fn next(&self) -> Self {
        Self::from_index(self.to_index() + 1)
    }

    pub fn button_label(&self) -> &'static str {
        match self {
            FormationType::Line => "LINE",
            FormationType::Wedge => "WEDGE",
            FormationType::Box => "BOX",
            FormationType::Scatter => "SCATTER",
            FormationType::Circle => "CIRCLE",
            FormationType::Column => "COLUMN",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            FormationType::Line => "HÀNG NGANG (PHALANX)",
            FormationType::Wedge => "MŨI TÊN (SPEARHEAD)",
            FormationType::Box => "Ô VUÔNG (PROTECT BOX)",
            FormationType::Scatter => "PHÂN TÁN (ANTI-AOE)",
            FormationType::Circle => "VÒNG CUNG (CRESCENT)",
            FormationType::Column => "CỘT DỌC (BLITZ COLUMN)",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            FormationType::Line => {
                "Dàn hàng ngang 2-3 lớp: Tiên phong đi trước, Xạ thủ tối đa góc hỏa lực."
            }
            FormationType::Wedge => {
                "Hình nêm mũi tên V: Đột phá xuyên tâm, tập trung xung lực cận chiến."
            }
            FormationType::Box => {
                "Khung vuông hộ vệ: Lính khiên bọc ngoài, pháo binh Laser/Rocket an toàn ở lõi."
            }
            FormationType::Scatter => {
                "Đội hình phân tán so le: Tản rộng >50px, giảm thiểu tối đa sát thương nổ AoE."
            }
            FormationType::Circle => {
                "Vòng cung bán nguyệt: Bao vây mục tiêu, hội tụ toàn bộ hỏa lực vào tâm."
            }
            FormationType::Column => {
                "Hai hàng dọc cơ động: Luồn lách chokepoint, luồn sâu đột kích mỏ quặng."
            }
        }
    }
}

#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BuildingType {
    CommandCenter = 0,   // Nhà chính (Nhận tài nguyên, xin Kỹ sư)
    SupplyDepot = 1,     // Trạm tiếp vận (+Pop cap)
    PowerReactor = 2,    // Lò phản ứng (+Energy/s)
    Barracks = 3,        // Trại huấn luyện lính
    TurretTower = 4,     // Tháp pháo phòng thủ tự động
    EnergyConverter = 5, // Trạm chuyển hóa năng lượng thành vàng
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum UnitOrder {
    Idle,
    MoveTo {
        x: f32,
        y: f32,
    },
    AttackMove {
        x: f32,
        y: f32,
    },
    Harvest {
        target_mineral_id: u32,
        phase: HarvestPhase,
    },
    Build {
        target_building_id: u32,
    },
    AttackTarget {
        target_id: u32,
        is_building: bool,
    },
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
    #[serde(default)]
    pub prev_x: f32,
    #[serde(default)]
    pub prev_y: f32,
    pub target_x: f32,
    pub target_y: f32,
    pub speed: f32,
    pub hp: f32,
    pub max_hp: f32,
    pub attack_range: f32,
    pub attack_damage: f32,
    pub damage_type: DamageType,
    pub aoe_radius: f32,
    pub resistances: Resistances,
    pub attack_cooldown: f32,
    pub current_cooldown: f32,
    pub walk_cycle: f32,
    pub facing_right: bool,
    pub order: UnitOrder,
    pub carried_minerals: f32,
    #[serde(default)]
    pub formation_offset: (f32, f32),
    #[serde(default)]
    pub shield: f32,
    #[serde(default)]
    pub max_shield: f32,
    #[serde(default)]
    pub shield_timer: f32,
    #[serde(default)]
    pub laser_target_id: Option<(u32, bool)>, // (target_id, is_building)
    #[serde(default)]
    pub laser_stacks: u32,
    #[serde(default)]
    pub flying_swords: u32,
    #[serde(default)]
    pub sword_regen_timer: f32,
    #[serde(default)]
    pub attack_anim_timer: f32,
    #[serde(default)]
    pub last_attack_pos: Option<(f32, f32)>,
    #[serde(default)]
    pub percent_max_hp_dps: f32,
    #[serde(default)]
    pub hit_flash_timer: f32,
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
    pub damage_type: DamageType,
    pub resistances: Resistances,
    pub build_progress: f32,     // 0.0 -> 1.0 (1.0 là hoàn tất)
    pub under_attack_timer: f32, // Thời gian còn lại chịu hiệu ứng bị tấn công (s)
    pub attack_cooldown: f32,
    pub current_cooldown: f32,
    pub rally_x: f32,
    pub rally_y: f32,
    // Hàng đợi sản xuất
    pub training_queue: Vec<UnitType>,
    pub training_timer: f32,
    #[serde(default)]
    pub hit_flash_timer: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ParticleColor {
    Blue,
    Cyan,
    Yellow,
    Orange,
    Red,
    Purple,
    Green,
    Slate,
    White,
}

impl std::str::FromStr for ParticleColor {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "#38bdf8" => ParticleColor::Blue,
            "#00e5ff" => ParticleColor::Cyan,
            "#facc15" | "#fef08a" | "#fbbf24" => ParticleColor::Yellow,
            "#fb923c" | "#ea580c" | "#f97316" => ParticleColor::Orange,
            "#ef4444" | "#f43f5e" => ParticleColor::Red,
            "#c084fc" | "#a855f7" => ParticleColor::Purple,
            "#84cc16" | "#a3e635" => ParticleColor::Green,
            "#475569" | "#94a3b8" => ParticleColor::Slate,
            _ => ParticleColor::White,
        })
    }
}

impl From<&str> for ParticleColor {
    fn from(s: &str) -> Self {
        use std::str::FromStr;
        Self::from_str(s).unwrap_or(ParticleColor::White)
    }
}

impl ParticleColor {
    pub fn from_hex(s: &str) -> Self {
        use std::str::FromStr;
        Self::from_str(s).unwrap_or(ParticleColor::White)
    }

    pub fn to_hex(&self) -> &'static str {
        match self {
            ParticleColor::Blue => "#38bdf8",
            ParticleColor::Cyan => "#00e5ff",
            ParticleColor::Yellow => "#facc15",
            ParticleColor::Orange => "#ea580c",
            ParticleColor::Red => "#ef4444",
            ParticleColor::Purple => "#c084fc",
            ParticleColor::Green => "#84cc16",
            ParticleColor::Slate => "#475569",
            ParticleColor::White => "#ffffff",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Particle {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub color: ParticleColor,
    pub size: f32,
    pub lifetime: f32,
    pub max_lifetime: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MineralNode {
    pub id: u32,
    pub x: f32,
    pub y: f32,
    pub amount: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ProjectileVisual {
    #[default]
    Bullet,
    Rocket,
    FlyingSword,
    Laser,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Projectile {
    pub id: u32,
    pub team: Team,
    pub x: f32,
    pub y: f32,
    #[serde(default)]
    pub prev_x: f32,
    #[serde(default)]
    pub prev_y: f32,
    pub target_x: f32,
    pub target_y: f32,
    pub speed: f32,
    pub damage: f32,
    pub damage_type: DamageType,
    pub aoe_radius: f32,
    pub is_laser: bool,
    pub lifetime: f32,
    #[serde(default)]
    pub is_melee: bool,
    #[serde(default = "default_mult_one")]
    pub building_damage_mult: f32,
    #[serde(default)]
    pub visual: ProjectileVisual,
    #[serde(default)]
    pub angle: f32,
}

fn default_mult_one() -> f32 {
    1.0
}

fn default_arcane() -> DamageType {
    DamageType::Arcane
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MagicCircle {
    pub id: u32,
    pub team: Team,
    pub x: f32,
    pub y: f32,
    pub radius: f32,
    pub lifetime: f32,
    pub max_lifetime: f32,
    pub tick_timer: f32,
    #[serde(default)]
    pub damage_pct_per_sec: f32,
    #[serde(default = "default_arcane")]
    pub damage_type: DamageType,
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
