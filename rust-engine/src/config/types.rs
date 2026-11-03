use crate::components::{DamageType, Resistances};
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
    #[serde(default)]
    pub percent_max_hp_dps: f32,
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

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TechItemConfig {
    pub name: String,
    pub branch: String, // "science" | "magic" | "monster" | "logistics"
    pub cost_minerals: f32,
    pub cost_energy: f32,
    pub research_time: f32,
}

pub const DEFAULT_PLAYER_COLORS: [(&str, &str); 8] = [
    ("#00e5ff", "rgba(0, 229, 255, 0.12)"),  // Slot 0: Cyan neon (Player)
    ("#f43f5e", "rgba(244, 63, 94, 0.12)"),  // Slot 1: Crimson / Rose red
    ("#a855f7", "rgba(168, 85, 247, 0.12)"), // Slot 2: Arcane purple
    ("#f59e0b", "rgba(245, 158, 11, 0.12)"), // Slot 3: Amber orange
    ("#10b981", "rgba(16, 185, 129, 0.12)"), // Slot 4: Emerald green
    ("#ec4899", "rgba(236, 72, 153, 0.12)"), // Slot 5: Hot pink / Magenta
    ("#3b82f6", "rgba(59, 130, 246, 0.12)"), // Slot 6: Electric royal blue
    ("#eab308", "rgba(234, 179, 8, 0.12)"),  // Slot 7: Gold yellow
];

fn default_player_palette() -> Vec<(String, String)> {
    DEFAULT_PLAYER_COLORS
        .iter()
        .map(|(s, f)| (s.to_string(), f.to_string()))
        .collect()
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
    #[serde(default = "default_player_palette")]
    pub player_palette: Vec<(String, String)>,
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
                (
                    "#f59e0b".to_string(),
                    "rgba(245, 158, 11, 0.12)".to_string(),
                ),
                (
                    "#a855f7".to_string(),
                    "rgba(168, 85, 247, 0.12)".to_string(),
                ),
                ("#eab308".to_string(), "rgba(234, 179, 8, 0.12)".to_string()),
                (
                    "#ec4899".to_string(),
                    "rgba(236, 72, 153, 0.12)".to_string(),
                ),
                (
                    "#f97316".to_string(),
                    "rgba(249, 115, 22, 0.12)".to_string(),
                ),
            ],
            player_palette: default_player_palette(),
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
