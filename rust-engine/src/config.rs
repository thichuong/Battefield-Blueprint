use crate::components::{Building, DamageType, Resistances, Team, Unit, UnitOrder};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct UnitStatsConfig {
    pub name: String,
    #[serde(default)]
    pub short_name: String,
    #[serde(default)]
    pub description: String,
    pub hp: f32,
    pub speed: f32,
    pub attack_range: f32,
    pub attack_damage: f32,
    pub attack_cooldown: f32,
    pub damage_type: DamageType,
    pub aoe_radius: f32,
    pub resistances: Resistances,
    pub cost_minerals: f32,
    pub cost_energy: f32,
    pub training_time: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BuildingStatsConfig {
    pub name: String,
    #[serde(default)]
    pub short_name: String,
    #[serde(default)]
    pub description: String,
    pub hp: f32,
    pub cost_minerals: f32,
    pub cost_energy: f32,
    pub width: f32,
    pub height: f32,
    pub build_time: f32,
    pub resistances: Resistances,
    // Thuộc tính riêng cho từng loại nhà
    pub supply_bonus: u32,
    pub energy_per_sec: f32,
    pub turret_range: f32,
    pub turret_damage: f32,
    pub turret_cooldown: f32,
    pub turret_damage_type: DamageType,
    pub turret_projectile_speed: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct EconomyConfig {
    pub starting_minerals: f32,
    pub starting_energy: f32,
    pub starting_supply: u32,
    pub base_max_supply: u32,
    pub base_energy_per_sec: f32,
    pub worker_carry_capacity: f32,
    pub worker_mining_time: f32,
    pub worker_carry_capacity_upgraded: f32,
    pub worker_mining_time_upgraded: f32,
    pub converter_threshold: f32,
    pub converter_energy_per_sec: f32,
    pub converter_minerals_per_100_energy: f32,
}

impl Default for EconomyConfig {
    fn default() -> Self {
        Self {
            starting_minerals: 200.0,
            starting_energy: 50.0,
            starting_supply: 4,
            base_max_supply: 10,
            base_energy_per_sec: 2.0,
            worker_carry_capacity: 10.0,
            worker_mining_time: 2.0,
            worker_carry_capacity_upgraded: 15.0,
            worker_mining_time_upgraded: 1.4,
            converter_threshold: 1000.0,
            converter_energy_per_sec: 100.0,
            converter_minerals_per_100_energy: 50.0,
        }
    }
}

// =========================================================================
// HẰNG SỐ CẤU HÌNH HỆ THỐNG (CENTRALIZED CONSTANTS)
// =========================================================================

// === BẢN ĐỒ & CHIẾN TRƯỜNG (MAP & ENVIRONMENT) ===
pub const MAP_WIDTH: f32 = 4800.0;
pub const MAP_HEIGHT: f32 = 4800.0;

// === SƯƠNG MÙ CHIẾN TRANH (FOG OF WAR) ===
pub const FOG_COLS: usize = 96;
pub const FOG_ROWS: usize = 96;
pub const CELL_W: f32 = MAP_WIDTH / (FOG_COLS as f32);
pub const CELL_H: f32 = MAP_HEIGHT / (FOG_ROWS as f32);

// === LƯỚI KHÔNG GIAN TÌM KIẾM ĐƠN VỊ (SPATIAL GRID) ===
pub const CELL_SIZE: f32 = 150.0;
pub const COLS: usize = ((MAP_WIDTH / CELL_SIZE) as usize) + 1; // 33
pub const ROWS: usize = ((MAP_HEIGHT / CELL_SIZE) as usize) + 1; // 33

// === KINH TẾ & CHUYỂN ĐỔI NĂNG LƯỢNG (ECONOMY & CONVERSION) ===
pub const ENERGY_CONVERSION_THRESHOLD: f32 = 1000.0;
pub const CONVERT_ENERGY_PER_SEC: f32 = 30.0;
pub const CONVERT_MINERALS_PER_SEC: f32 = 10.0;

// === TỐC ĐỘ XÂY DỰNG & HIỆU ỨNG BỊ TẤN CÔNG ===
pub const BUILD_SPEED_BASE: f32 = 0.05; // Tốc độ xây tự động cơ bản khi không có thợ (1/20s)
pub const BUILD_SPEED_WORKER: f32 = 0.20; // Tốc độ xây khi có Engineer ở gần (1/5s)
pub const WORKER_BUILD_RANGE: f32 = 90.0; // Khoảng cách thợ hỗ trợ xây dựng
pub const BUILDING_UNDER_ATTACK_DURATION: f32 = 5.0; // Thời gian duy trì hiệu ứng bị tấn công (5s)
pub const BUILDING_UNDER_ATTACK_SPEED_MULTIPLIER: f32 = 0.20; // Giảm còn 30% tốc độ xây khi bị tấn công

// === NGHIÊN CỨU ĐA NHÁNH CÔNG NGHỆ ===
pub const MAX_CONCURRENT_RESEARCH_PER_BRANCH: usize = 1; // 1 công nghệ mỗi nhánh cùng lúc
pub const MAX_TOTAL_CONCURRENT_RESEARCH: usize = 4; // Tối đa 4 nhánh chạy đồng thời

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TechItemConfig {
    pub name: String,
    pub branch: String, // "science" | "magic" | "monster" | "logistics"
    pub cost_minerals: f32,
    pub cost_energy: f32,
    pub research_time: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct RenderConfig {
    pub bg_color: String,
    pub grid_line_color: String,
    pub grid_size: f32,
    pub map_border_color: String,
    pub selection_color: String,
    pub player_color: String,
    pub player_fill: String,
    pub ally_color: String,
    pub ally_fill: String,
    pub enemy_palette: Vec<(String, String)>,
    pub hp_high_color: String,
    pub hp_low_color: String,
    pub build_progress_color: String,
    pub build_under_attack_color: String,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            bg_color: "#030d1a".to_string(),
            grid_line_color: "#0a223e".to_string(),
            grid_size: 80.0,
            map_border_color: "#00e5ff".to_string(),
            selection_color: "#ffffff".to_string(),
            player_color: "#00e5ff".to_string(),
            player_fill: "rgba(0, 229, 255, 0.12)".to_string(),
            ally_color: "#10b981".to_string(),
            ally_fill: "rgba(16, 185, 129, 0.12)".to_string(),
            enemy_palette: vec![
                ("#f43f5e".to_string(), "rgba(244, 63, 94, 0.12)".to_string()),
                ("#f59e0b".to_string(), "rgba(245, 158, 11, 0.12)".to_string()),
                ("#a855f7".to_string(), "rgba(168, 85, 247, 0.12)".to_string()),
                ("#eab308".to_string(), "rgba(234, 179, 8, 0.12)".to_string()),
                ("#ec4899".to_string(), "rgba(236, 72, 153, 0.12)".to_string()),
                ("#f97316".to_string(), "rgba(249, 115, 22, 0.12)".to_string()),
            ],
            hp_high_color: "#22c55e".to_string(),
            hp_low_color: "#ef4444".to_string(),
            build_progress_color: "#fbbf24".to_string(),
            build_under_attack_color: "#ef4444".to_string(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct GameConfig {
    pub units: HashMap<String, UnitStatsConfig>,
    pub buildings: HashMap<String, BuildingStatsConfig>,
    pub economy: EconomyConfig,
    pub techs: HashMap<String, TechItemConfig>,
    #[serde(default)]
    pub render: RenderConfig,
}

impl Default for GameConfig {
    fn default() -> Self {
        let mut units = HashMap::new();
        let mut buildings = HashMap::new();
        let mut techs = HashMap::new();

        // 1. UNITS
        units.insert(
            "Engineer".to_string(),
            UnitStatsConfig {
                name: "ENGINEER // WORKER".to_string(),
                short_name: "WORKER".to_string(),
                description: "Huấn luyện thợ mỏ / kỹ sư xây dựng.".to_string(),
                hp: 90.0,
                speed: 120.0,
                attack_range: 30.0,
                attack_damage: 5.0,
                attack_cooldown: 1.0,
                damage_type: DamageType::Physical,
                aoe_radius: 0.0,
                resistances: Resistances {
                    physical: 0.05,
                    energy: 0.0,
                    explosive: 0.0,
                    arcane: 0.0,
                },
                cost_minerals: 50.0,
                cost_energy: 0.0,
                training_time: 5.0,
            },
        );

        units.insert(
            "ShockTrooper".to_string(),
            UnitStatsConfig {
                name: "STRIKER // SHOCK TROOPER".to_string(),
                short_name: "STRIKER".to_string(),
                description: "Kiếm sĩ Plasma, chạy nhanh, chém mạnh.".to_string(),
                hp: 160.0,
                speed: 140.0,
                attack_range: 40.0,
                attack_damage: 22.0,
                attack_cooldown: 0.8,
                damage_type: DamageType::Energy,
                aoe_radius: 0.0,
                resistances: Resistances {
                    physical: 0.15,
                    energy: 0.20,
                    explosive: 0.05,
                    arcane: 0.10,
                },
                cost_minerals: 75.0,
                cost_energy: 0.0,
                training_time: 7.0,
            },
        );

        units.insert(
            "Rifleman".to_string(),
            UnitStatsConfig {
                name: "MARKSMAN // RIFLEMAN".to_string(),
                short_name: "RIFLE".to_string(),
                description: "Xạ thủ súng trường tầm xa.".to_string(),
                hp: 120.0,
                speed: 110.0,
                attack_range: 160.0,
                attack_damage: 14.0,
                attack_cooldown: 1.1,
                damage_type: DamageType::Physical,
                aoe_radius: 0.0,
                resistances: Resistances {
                    physical: 0.10,
                    energy: 0.05,
                    explosive: 0.05,
                    arcane: 0.05,
                },
                cost_minerals: 100.0,
                cost_energy: 20.0,
                training_time: 8.0,
            },
        );

        units.insert(
            "Vanguard".to_string(),
            UnitStatsConfig {
                name: "VANGUARD // SHIELDER".to_string(),
                short_name: "SHIELD".to_string(),
                description: "Vệ binh khiên chắn năng lượng.".to_string(),
                hp: 320.0,
                speed: 90.0,
                attack_range: 45.0,
                attack_damage: 18.0,
                attack_cooldown: 1.2,
                damage_type: DamageType::Physical,
                aoe_radius: 0.0,
                resistances: Resistances {
                    physical: 0.40,
                    energy: 0.50,
                    explosive: 0.40,
                    arcane: 0.30,
                },
                cost_minerals: 150.0,
                cost_energy: 50.0,
                training_time: 12.0,
            },
        );

        units.insert(
            "LaserMarksman".to_string(),
            UnitStatsConfig {
                name: "SIEGE // LASER MARKSMAN".to_string(),
                short_name: "LASER".to_string(),
                description: "Pháo Laser siêu sát thương.".to_string(),
                hp: 110.0,
                speed: 95.0,
                attack_range: 220.0,
                attack_damage: 25.0,
                attack_cooldown: 0.5,
                damage_type: DamageType::Energy,
                aoe_radius: 0.0,
                resistances: Resistances {
                    physical: 0.05,
                    energy: 0.30,
                    explosive: 0.0,
                    arcane: 0.15,
                },
                cost_minerals: 175.0,
                cost_energy: 100.0,
                training_time: 15.0,
            },
        );

        units.insert(
            "DarkRocketeer".to_string(),
            UnitStatsConfig {
                name: "DARK TECH // ROCKETEER".to_string(),
                short_name: "ROCKET".to_string(),
                description: "Bệ phóng Rocket nổ AoE tầm xa.".to_string(),
                hp: 140.0,
                speed: 95.0,
                attack_range: 200.0,
                attack_damage: 50.0,
                attack_cooldown: 1.5,
                damage_type: DamageType::Explosive,
                aoe_radius: 50.0,
                resistances: Resistances {
                    physical: 0.20,
                    energy: 0.10,
                    explosive: 0.35,
                    arcane: 0.05,
                },
                cost_minerals: 160.0,
                cost_energy: 80.0,
                training_time: 14.0,
            },
        );

        units.insert(
            "TaoistSwordmaster".to_string(),
            UnitStatsConfig {
                name: "CELESTIAL // SWORDMASTER".to_string(),
                short_name: "SWORD".to_string(),
                description: "Tu tiên giả ngự phi kiếm chân khí Arcane.".to_string(),
                hp: 170.0,
                speed: 130.0,
                attack_range: 140.0,
                attack_damage: 40.0,
                attack_cooldown: 1.0,
                damage_type: DamageType::Arcane,
                aoe_radius: 10.0,
                resistances: Resistances {
                    physical: 0.15,
                    energy: 0.35,
                    explosive: 0.15,
                    arcane: 0.45,
                },
                cost_minerals: 180.0,
                cost_energy: 90.0,
                training_time: 15.0,
            },
        );

        units.insert(
            "VoidWarlock".to_string(),
            UnitStatsConfig {
                name: "VOID // SHADOW WARLOCK".to_string(),
                short_name: "WARLOCK".to_string(),
                description: "Vu sư hư không, cầu ma thuật nổ AoE.".to_string(),
                hp: 135.0,
                speed: 100.0,
                attack_range: 175.0,
                attack_damage: 35.0,
                attack_cooldown: 1.0,
                damage_type: DamageType::Arcane,
                aoe_radius: 35.0,
                resistances: Resistances {
                    physical: 0.10,
                    energy: 0.25,
                    explosive: 0.20,
                    arcane: 0.50,
                },
                cost_minerals: 200.0,
                cost_energy: 120.0,
                training_time: 16.0,
            },
        );

        units.insert(
            "BehemothTitan".to_string(),
            UnitStatsConfig {
                name: "BEHEMOTH // TITAN".to_string(),
                short_name: "TITAN".to_string(),
                description: "Cự thú HP khủng, dậm chấn động diện rộng.".to_string(),
                hp: 650.0,
                speed: 60.0,
                attack_range: 55.0,
                attack_damage: 40.0,
                attack_cooldown: 1.5,
                damage_type: DamageType::Physical,
                aoe_radius: 35.0,
                resistances: Resistances {
                    physical: 0.50,
                    energy: 0.20,
                    explosive: 0.20,
                    arcane: 0.10,
                },
                cost_minerals: 250.0,
                cost_energy: 100.0,
                training_time: 20.0,
            },
        );

        units.insert(
            "AcidBaneling".to_string(),
            UnitStatsConfig {
                name: "SWARM // ACID BANELING".to_string(),
                short_name: "BANELING".to_string(),
                description: "Bọ cảm tử lao vào tự nổ axit AoE cực lớn.".to_string(),
                hp: 100.0,
                speed: 160.0,
                attack_range: 25.0,
                attack_damage: 90.0,
                attack_cooldown: 0.5,
                damage_type: DamageType::Explosive,
                aoe_radius: 70.0,
                resistances: Resistances {
                    physical: 0.05,
                    energy: 0.05,
                    explosive: 0.20,
                    arcane: 0.20,
                },
                cost_minerals: 60.0,
                cost_energy: 30.0,
                training_time: 6.0,
            },
        );

        // 2. BUILDINGS
        buildings.insert(
            "CommandCenter".to_string(),
            BuildingStatsConfig {
                name: "HQ COMMAND CENTER".to_string(),
                short_name: "HQ CC".to_string(),
                description: "Trung tâm chỉ huy mở rộng.".to_string(),
                hp: 2000.0,
                cost_minerals: 400.0,
                cost_energy: 0.0,
                width: 90.0,
                height: 90.0,
                build_time: 15.0,
                resistances: Resistances {
                    physical: 0.80,
                    energy: 0.60,
                    explosive: 0.25,
                    arcane: 0.20,
                },
                supply_bonus: 10,
                energy_per_sec: 0.0,
                turret_range: 250.0,
                turret_damage: 100.0,
                turret_cooldown: 1.0,
                turret_damage_type: DamageType::Physical,
                turret_projectile_speed: 900.0,
            },
        );

        buildings.insert(
            "SupplyDepot".to_string(),
            BuildingStatsConfig {
                name: "SUPPLY DEPOT".to_string(),
                short_name: "SUPPLY".to_string(),
                description: "Tăng Giới hạn dân số (Supply).".to_string(),
                hp: 800.0,
                cost_minerals: 100.0,
                cost_energy: 0.0,
                width: 60.0,
                height: 60.0,
                build_time: 8.0,
                resistances: Resistances {
                    physical: 0.20,
                    energy: 0.15,
                    explosive: 0.10,
                    arcane: 0.05,
                },
                supply_bonus: 8,
                energy_per_sec: 0.0,
                turret_range: 0.0,
                turret_damage: 0.0,
                turret_cooldown: 0.0,
                turret_damage_type: DamageType::Physical,
                turret_projectile_speed: 900.0,
            },
        );

        buildings.insert(
            "PowerReactor".to_string(),
            BuildingStatsConfig {
                name: "POWER REACTOR".to_string(),
                short_name: "REACTOR".to_string(),
                description: "Tăng tốc độ sản sinh Năng lượng.".to_string(),
                hp: 700.0,
                cost_minerals: 120.0,
                cost_energy: 0.0,
                width: 60.0,
                height: 60.0,
                build_time: 10.0,
                resistances: Resistances {
                    physical: 0.15,
                    energy: 0.25,
                    explosive: 0.05,
                    arcane: 0.15,
                },
                supply_bonus: 0,
                energy_per_sec: 8.0,
                turret_range: 0.0,
                turret_damage: 0.0,
                turret_cooldown: 0.0,
                turret_damage_type: DamageType::Physical,
                turret_projectile_speed: 900.0,
            },
        );

        buildings.insert(
            "Barracks".to_string(),
            BuildingStatsConfig {
                name: "BARRACKS".to_string(),
                short_name: "BARRACKS".to_string(),
                description: "Trại huấn luyện lính chiến đấu.".to_string(),
                hp: 1200.0,
                cost_minerals: 150.0,
                cost_energy: 0.0,
                width: 80.0,
                height: 70.0,
                build_time: 12.0,
                resistances: Resistances {
                    physical: 0.35,
                    energy: 0.20,
                    explosive: 0.25,
                    arcane: 0.15,
                },
                supply_bonus: 0,
                energy_per_sec: 0.0,
                turret_range: 0.0,
                turret_damage: 0.0,
                turret_cooldown: 0.0,
                turret_damage_type: DamageType::Physical,
                turret_projectile_speed: 900.0,
            },
        );

        buildings.insert(
            "TurretTower".to_string(),
            BuildingStatsConfig {
                name: "TURRET TOWER".to_string(),
                short_name: "TURRET".to_string(),
                description: "Tháp phòng thủ tự động bắn tỉa.".to_string(),
                hp: 900.0,
                cost_minerals: 125.0,
                cost_energy: 50.0,
                width: 50.0,
                height: 50.0,
                build_time: 10.0,
                resistances: Resistances {
                    physical: 0.85,
                    energy: 0.60,
                    explosive: 0.35,
                    arcane: 0.30,
                },
                supply_bonus: 0,
                energy_per_sec: 0.0,
                turret_range: 250.0,
                turret_damage: 25.0,
                turret_cooldown: 1.0,
                turret_damage_type: DamageType::Energy,
                turret_projectile_speed: 900.0,
            },
        );

        buildings.insert(
            "EnergyConverter".to_string(),
            BuildingStatsConfig {
                name: "ENERGY CONVERTER".to_string(),
                short_name: "CONVERT".to_string(),
                description: "Tự động chuyển Năng lượng dư thừa thành Vàng.".to_string(),
                hp: 850.0,
                cost_minerals: 150.0,
                cost_energy: 50.0,
                width: 65.0,
                height: 65.0,
                build_time: 12.0,
                resistances: Resistances {
                    physical: 0.25,
                    energy: 0.35,
                    explosive: 0.10,
                    arcane: 0.25,
                },
                supply_bonus: 0,
                energy_per_sec: 0.0,
                turret_range: 0.0,
                turret_damage: 0.0,
                turret_cooldown: 0.0,
                turret_damage_type: DamageType::Physical,
                turret_projectile_speed: 900.0,
            },
        );

        // 3. TECHS
        techs.insert(
            "focused_laser".to_string(),
            TechItemConfig {
                name: "Focused Laser".to_string(),
                branch: "science".to_string(),
                cost_minerals: 250.0,
                cost_energy: 200.0,
                research_time: 20.0,
            },
        );
        techs.insert(
            "energy_shielding".to_string(),
            TechItemConfig {
                name: "Energy Shielding".to_string(),
                branch: "science".to_string(),
                cost_minerals: 180.0,
                cost_energy: 150.0,
                research_time: 15.0,
            },
        );
        techs.insert(
            "dark_matter_rockets".to_string(),
            TechItemConfig {
                name: "Dark Matter Rockets".to_string(),
                branch: "science".to_string(),
                cost_minerals: 220.0,
                cost_energy: 120.0,
                research_time: 18.0,
            },
        );
        techs.insert(
            "titan_alloy_plating".to_string(),
            TechItemConfig {
                name: "Titan Alloy Plating".to_string(),
                branch: "science".to_string(),
                cost_minerals: 300.0,
                cost_energy: 180.0,
                research_time: 25.0,
            },
        );
        techs.insert(
            "celestial_flying_swords".to_string(),
            TechItemConfig {
                name: "Celestial Flying Swords".to_string(),
                branch: "magic".to_string(),
                cost_minerals: 260.0,
                cost_energy: 160.0,
                research_time: 22.0,
            },
        );
        techs.insert(
            "golden_bell_shield".to_string(),
            TechItemConfig {
                name: "Golden Bell Shield".to_string(),
                branch: "magic".to_string(),
                cost_minerals: 200.0,
                cost_energy: 140.0,
                research_time: 16.0,
            },
        );
        techs.insert(
            "corrosive_curse".to_string(),
            TechItemConfig {
                name: "Corrosive Curse".to_string(),
                branch: "magic".to_string(),
                cost_minerals: 180.0,
                cost_energy: 160.0,
                research_time: 16.0,
            },
        );
        techs.insert(
            "void_cataclysm".to_string(),
            TechItemConfig {
                name: "Void Cataclysm".to_string(),
                branch: "magic".to_string(),
                cost_minerals: 320.0,
                cost_energy: 240.0,
                research_time: 28.0,
            },
        );
        techs.insert(
            "primeval_scales".to_string(),
            TechItemConfig {
                name: "Primeval Scales".to_string(),
                branch: "monster".to_string(),
                cost_minerals: 220.0,
                cost_energy: 120.0,
                research_time: 18.0,
            },
        );
        techs.insert(
            "behemoth_mutation".to_string(),
            TechItemConfig {
                name: "Behemoth Mutation".to_string(),
                branch: "monster".to_string(),
                cost_minerals: 320.0,
                cost_energy: 180.0,
                research_time: 26.0,
            },
        );
        techs.insert(
            "acid_banelings".to_string(),
            TechItemConfig {
                name: "Acid Banelings".to_string(),
                branch: "monster".to_string(),
                cost_minerals: 160.0,
                cost_energy: 90.0,
                research_time: 14.0,
            },
        );
        techs.insert(
            "hive_proliferation".to_string(),
            TechItemConfig {
                name: "Hive Proliferation".to_string(),
                branch: "monster".to_string(),
                cost_minerals: 260.0,
                cost_energy: 160.0,
                research_time: 20.0,
            },
        );
        techs.insert(
            "optical_drills".to_string(),
            TechItemConfig {
                name: "Optical Drills".to_string(),
                branch: "logistics".to_string(),
                cost_minerals: 100.0,
                cost_energy: 50.0,
                research_time: 10.0,
            },
        );
        techs.insert(
            "overcharged_reactors".to_string(),
            TechItemConfig {
                name: "Overcharged Reactors".to_string(),
                branch: "logistics".to_string(),
                cost_minerals: 150.0,
                cost_energy: 100.0,
                research_time: 15.0,
            },
        );
        techs.insert(
            "fortified_blueprints".to_string(),
            TechItemConfig {
                name: "Fortified Blueprints".to_string(),
                branch: "logistics".to_string(),
                cost_minerals: 200.0,
                cost_energy: 150.0,
                research_time: 20.0,
            },
        );
        techs.insert(
            "magnetic_rounds".to_string(),
            TechItemConfig {
                name: "Magnetic Rounds".to_string(),
                branch: "logistics".to_string(),
                cost_minerals: 200.0,
                cost_energy: 150.0,
                research_time: 15.0,
            },
        );
        techs.insert(
            "exo_boosters".to_string(),
            TechItemConfig {
                name: "Exo Boosters".to_string(),
                branch: "logistics".to_string(),
                cost_minerals: 250.0,
                cost_energy: 200.0,
                research_time: 20.0,
            },
        );
        techs.insert(
            "superheated_blades".to_string(),
            TechItemConfig {
                name: "Superheated Blades".to_string(),
                branch: "logistics".to_string(),
                cost_minerals: 350.0,
                cost_energy: 300.0,
                research_time: 30.0,
            },
        );
        techs.insert(
            "overload_strike".to_string(),
            TechItemConfig {
                name: "Overload Strike".to_string(),
                branch: "logistics".to_string(),
                cost_minerals: 400.0,
                cost_energy: 350.0,
                research_time: 35.0,
            },
        );

        Self {
            units,
            buildings,
            economy: EconomyConfig::default(),
            techs,
            render: RenderConfig::default(),
        }
    }
}

impl GameConfig {
    pub fn unit_key(u_type: crate::components::UnitType) -> &'static str {
        match u_type {
            crate::components::UnitType::Engineer => "Engineer",
            crate::components::UnitType::ShockTrooper => "ShockTrooper",
            crate::components::UnitType::Rifleman => "Rifleman",
            crate::components::UnitType::Vanguard => "Vanguard",
            crate::components::UnitType::LaserMarksman => "LaserMarksman",
            crate::components::UnitType::DarkRocketeer => "DarkRocketeer",
            crate::components::UnitType::TaoistSwordmaster => "TaoistSwordmaster",
            crate::components::UnitType::VoidWarlock => "VoidWarlock",
            crate::components::UnitType::BehemothTitan => "BehemothTitan",
            crate::components::UnitType::AcidBaneling => "AcidBaneling",
        }
    }

    pub fn building_key(b_type: crate::components::BuildingType) -> &'static str {
        match b_type {
            crate::components::BuildingType::CommandCenter => "CommandCenter",
            crate::components::BuildingType::SupplyDepot => "SupplyDepot",
            crate::components::BuildingType::PowerReactor => "PowerReactor",
            crate::components::BuildingType::Barracks => "Barracks",
            crate::components::BuildingType::TurretTower => "TurretTower",
            crate::components::BuildingType::EnergyConverter => "EnergyConverter",
        }
    }

    #[inline]
    pub fn get_unit(&self, u_type: crate::components::UnitType) -> Option<&UnitStatsConfig> {
        self.units.get(Self::unit_key(u_type))
    }

    #[inline]
    pub fn get_building(
        &self,
        b_type: crate::components::BuildingType,
    ) -> Option<&BuildingStatsConfig> {
        self.buildings.get(Self::building_key(b_type))
    }

    #[inline]
    pub fn get_tech(&self, tech_id: &str) -> Option<&TechItemConfig> {
        self.techs.get(tech_id)
    }

    pub fn get_building_dimensions(&self, b_type: crate::components::BuildingType) -> (f32, f32) {
        if let Some(b) = self.get_building(b_type) {
            (b.width, b.height)
        } else {
            (60.0, 60.0)
        }
    }

    pub fn get_building_name(&self, b_type: crate::components::BuildingType) -> &str {
        if let Some(b) = self.get_building(b_type) {
            &b.name
        } else {
            "STRUCTURE"
        }
    }

    pub fn get_building_short_name(&self, b_type: crate::components::BuildingType) -> &str {
        if let Some(b) = self.get_building(b_type) {
            if !b.short_name.is_empty() {
                return &b.short_name;
            }
            &b.name
        } else {
            "BUILDING"
        }
    }

    pub fn get_building_description(&self, b_type: crate::components::BuildingType) -> &str {
        if let Some(b) = self.get_building(b_type) {
            &b.description
        } else {
            ""
        }
    }

    pub fn get_unit_name(&self, u_type: crate::components::UnitType) -> &str {
        if let Some(u) = self.get_unit(u_type) {
            &u.name
        } else {
            u_type.display_name()
        }
    }

    pub fn get_unit_short_name(&self, u_type: crate::components::UnitType) -> &str {
        if let Some(u) = self.get_unit(u_type) {
            if !u.short_name.is_empty() {
                return &u.short_name;
            }
            &u.name
        } else {
            u_type.button_label()
        }
    }

    pub fn get_unit_description(&self, u_type: crate::components::UnitType) -> &str {
        if let Some(u) = self.get_unit(u_type) {
            &u.description
        } else {
            ""
        }
    }

    pub fn get_unit_cost(&self, u_type: crate::components::UnitType) -> (f32, f32) {
        if let Some(u) = self.get_unit(u_type) {
            (u.cost_minerals, u.cost_energy)
        } else {
            u_type.cost()
        }
    }

    pub fn get_unit_training_time(&self, u_type: crate::components::UnitType) -> f32 {
        if let Some(u) = self.get_unit(u_type) {
            u.training_time
        } else {
            u_type.training_time()
        }
    }

    pub fn get_unit_cost_label(&self, u_type: crate::components::UnitType) -> String {
        let (min, en) = self.get_unit_cost(u_type);
        if en > 0.0 {
            format!("{:.0}M {:.0}E", min, en)
        } else {
            format!("{:.0}M", min)
        }
    }

    pub fn get_building_cost(&self, b_type: crate::components::BuildingType) -> (f32, f32) {
        if let Some(b) = self.get_building(b_type) {
            (b.cost_minerals, b.cost_energy)
        } else {
            (100.0, 0.0)
        }
    }

    pub fn get_building_cost_label(&self, b_type: crate::components::BuildingType) -> String {
        let (min, en) = self.get_building_cost(b_type);
        if en > 0.0 {
            format!("{:.0}M {:.0}E", min, en)
        } else {
            format!("{:.0}M", min)
        }
    }

    pub fn create_unit(
        &self,
        id: u32,
        team: Team,
        u_type: crate::components::UnitType,
        x: f32,
        y: f32,
    ) -> Unit {
        let default_cfg = UnitStatsConfig {
            name: "Unknown".to_string(),
            short_name: "UNK".to_string(),
            description: String::new(),
            hp: 100.0,
            speed: 100.0,
            attack_range: 40.0,
            attack_damage: 10.0,
            attack_cooldown: 1.0,
            damage_type: DamageType::Physical,
            aoe_radius: 0.0,
            resistances: Resistances::default(),
            cost_minerals: 50.0,
            cost_energy: 0.0,
            training_time: 5.0,
        };
        let cfg = self.get_unit(u_type).unwrap_or(&default_cfg);

        Unit {
            id,
            team,
            unit_type: u_type,
            x,
            y,
            target_x: x,
            target_y: y,
            speed: cfg.speed,
            hp: cfg.hp,
            max_hp: cfg.hp,
            attack_range: cfg.attack_range,
            attack_damage: cfg.attack_damage,
            damage_type: cfg.damage_type,
            aoe_radius: cfg.aoe_radius,
            resistances: cfg.resistances,
            attack_cooldown: cfg.attack_cooldown,
            current_cooldown: 0.0,
            walk_cycle: 0.0,
            facing_right: team == Team::Player,
            order: UnitOrder::Idle,
            carried_minerals: 0.0,
            formation_offset: (0.0, 0.0),
        }
    }

    pub fn create_building(
        &self,
        id: u32,
        team: Team,
        b_type: crate::components::BuildingType,
        x: f32,
        y: f32,
        is_completed: bool,
    ) -> Building {
        let default_cfg = BuildingStatsConfig {
            name: "Unknown".to_string(),
            short_name: "UNK".to_string(),
            description: String::new(),
            hp: 1000.0,
            cost_minerals: 100.0,
            cost_energy: 0.0,
            width: 60.0,
            height: 60.0,
            build_time: 10.0,
            resistances: Resistances::default(),
            supply_bonus: 0,
            energy_per_sec: 0.0,
            turret_range: 0.0,
            turret_damage: 0.0,
            turret_cooldown: 0.0,
            turret_damage_type: DamageType::Physical,
            turret_projectile_speed: 0.0,
        };
        let cfg = self.get_building(b_type).unwrap_or(&default_cfg);

        Building {
            id,
            team,
            building_type: b_type,
            x,
            y,
            width: cfg.width,
            height: cfg.height,
            hp: cfg.hp,
            max_hp: cfg.hp,
            damage_type: cfg.turret_damage_type,
            resistances: cfg.resistances,
            build_progress: if is_completed { 1.0 } else { 0.0 },
            under_attack_timer: 0.0,
            attack_cooldown: 0.0,
            current_cooldown: 0.0,
            rally_x: x,
            rally_y: y + cfg.height * 0.5 + 20.0,
            training_queue: Vec::new(),
            training_timer: 0.0,
        }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|_| "{}".to_string())
    }

    pub fn from_json(json_str: &str) -> Result<Self, String> {
        serde_json::from_str::<GameConfig>(json_str).map_err(|e| e.to_string())
    }

    pub fn update_from_json(&mut self, json_str: &str) -> Result<(), String> {
        let new_cfg = Self::from_json(json_str)?;
        *self = new_cfg;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::{BuildingType, UnitType};

    #[test]
    fn test_default_game_config_has_all_units_and_buildings() {
        let cfg = GameConfig::default();
        let unit_types = [
            UnitType::Engineer,
            UnitType::ShockTrooper,
            UnitType::Rifleman,
            UnitType::Vanguard,
            UnitType::LaserMarksman,
            UnitType::DarkRocketeer,
            UnitType::TaoistSwordmaster,
            UnitType::VoidWarlock,
            UnitType::BehemothTitan,
            UnitType::AcidBaneling,
        ];
        for u in unit_types {
            assert!(cfg.get_unit(u).is_some(), "Missing config for unit {:?}", u);
        }

        let building_types = [
            BuildingType::CommandCenter,
            BuildingType::SupplyDepot,
            BuildingType::PowerReactor,
            BuildingType::Barracks,
            BuildingType::TurretTower,
            BuildingType::EnergyConverter,
        ];
        for b in building_types {
            assert!(
                cfg.get_building(b).is_some(),
                "Missing config for building {:?}",
                b
            );
        }
    }

    #[test]
    fn test_game_config_json_roundtrip() {
        let cfg = GameConfig::default();
        let json = cfg.to_json();
        let parsed = GameConfig::from_json(&json).expect("Should parse json cleanly");
        assert_eq!(cfg, parsed);
    }

    #[test]
    fn test_custom_config_modifies_unit_creation() {
        let mut cfg = GameConfig::default();
        if let Some(u) = cfg.units.get_mut("Engineer") {
            u.hp = 999.0;
            u.speed = 333.0;
        }

        let unit = cfg.create_unit(1, Team::Player, UnitType::Engineer, 100.0, 100.0);
        assert_eq!(unit.hp, 999.0);
        assert_eq!(unit.speed, 333.0);
    }

    #[test]
    fn test_custom_config_modifies_building_creation() {
        let mut cfg = GameConfig::default();
        if let Some(b) = cfg.buildings.get_mut("CommandCenter") {
            b.hp = 5000.0;
        }

        let building = cfg.create_building(
            1,
            Team::Player,
            BuildingType::CommandCenter,
            200.0,
            200.0,
            true,
        );
        assert_eq!(building.hp, 5000.0);
        assert_eq!(building.build_progress, 1.0);
    }

    #[test]
    fn test_custom_config_modifies_building_weapon_stats() {
        let mut cfg = GameConfig::default();
        if let Some(b) = cfg.buildings.get_mut("Barracks") {
            b.turret_range = 220.0;
            b.turret_damage = 35.0;
            b.turret_cooldown = 0.8;
            b.turret_damage_type = DamageType::Arcane;
        }

        let b_cfg = cfg.get_building(BuildingType::Barracks).unwrap();
        assert_eq!(b_cfg.turret_range, 220.0);
        assert_eq!(b_cfg.turret_damage, 35.0);
        assert_eq!(b_cfg.turret_cooldown, 0.8);
        assert_eq!(b_cfg.turret_damage_type, DamageType::Arcane);
    }

    #[test]
    fn test_building_dimensions_and_labels_from_config() {
        let mut cfg = GameConfig::default();
        assert_eq!(cfg.get_building_dimensions(BuildingType::CommandCenter), (90.0, 90.0));
        assert_eq!(cfg.get_building_short_name(BuildingType::SupplyDepot), "SUPPLY");
        assert!(!cfg.get_building_description(BuildingType::PowerReactor).is_empty());

        // Custom config modification propagates
        if let Some(b) = cfg.buildings.get_mut("CommandCenter") {
            b.width = 120.0;
            b.height = 130.0;
            b.short_name = "MEGA CC".to_string();
        }
        assert_eq!(cfg.get_building_dimensions(BuildingType::CommandCenter), (120.0, 130.0));
        assert_eq!(cfg.get_building_short_name(BuildingType::CommandCenter), "MEGA CC");
    }

    #[test]
    fn test_unit_labels_and_descriptions_from_config() {
        let mut cfg = GameConfig::default();
        assert_eq!(cfg.get_unit_short_name(UnitType::ShockTrooper), "STRIKER");
        assert_eq!(cfg.get_unit_description(UnitType::Engineer), "Huấn luyện thợ mỏ / kỹ sư xây dựng.");

        // Custom modification
        if let Some(u) = cfg.units.get_mut("ShockTrooper") {
            u.short_name = "CYBER STRIKER".to_string();
            u.description = "Chiến binh plasma tương lai".to_string();
        }
        assert_eq!(cfg.get_unit_short_name(UnitType::ShockTrooper), "CYBER STRIKER");
        assert_eq!(cfg.get_unit_description(UnitType::ShockTrooper), "Chiến binh plasma tương lai");
    }

    #[test]
    fn test_render_config_customization() {
        let mut cfg = GameConfig::default();
        assert_eq!(cfg.render.grid_size, 80.0);
        assert_eq!(cfg.render.bg_color, "#030d1a");

        cfg.render.grid_size = 100.0;
        cfg.render.bg_color = "#000000".to_string();

        let json = cfg.to_json();
        let parsed = GameConfig::from_json(&json).expect("Roundtrip with custom render config");
        assert_eq!(parsed.render.grid_size, 100.0);
        assert_eq!(parsed.render.bg_color, "#000000");
    }
}
