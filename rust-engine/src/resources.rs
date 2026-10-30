use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlayerEconomy {
    pub minerals: f32,
    pub energy: f32,
    pub supply: u32,
    pub max_supply: u32,
    pub minerals_per_sec: f32,
    pub energy_per_sec: f32,
}

impl Default for PlayerEconomy {
    fn default() -> Self {
        Self {
            minerals: 200.0,
            energy: 50.0,
            supply: 4,
            max_supply: 10,
            minerals_per_sec: 0.0,
            energy_per_sec: 2.0,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct TechTreeState {
    // Nhánh Kinh tế
    pub optical_drills: bool,       // +30% tốc độ khai thác
    pub overcharged_reactors: bool, // +50% energy/s
    pub fortified_blueprints: bool, // +30% máu nhà

    // Nhánh Bộ binh
    pub energy_shielding: bool, // Mở Vanguard & +20% HP ShockTrooper
    pub magnetic_rounds: bool,  // +25% DMG Rifleman
    pub exo_boosters: bool,     // +20% tốc độ chạy lính

    // Nhánh Công nghệ cao
    pub focused_laser: bool,      // Mở LaserMarksman
    pub superheated_blades: bool, // Xuyên giáp kiếm sĩ
    pub overload_strike: bool,    // +40% tốc bắn Tháp canh & Laser

    // Đang nghiên cứu
    pub current_research: Option<String>,
    pub research_progress: f32, // 0.0 -> 1.0
    pub research_total_time: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameSettings {
    pub sound_volume: f32,
    pub camera_speed: f32,
    pub auto_pause_on_tech: bool,
    pub show_grid: bool,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            sound_volume: 0.7,
            camera_speed: 1.0,
            auto_pause_on_tech: true,
            show_grid: true,
        }
    }
}

pub struct CameraState {
    pub x: f32,
    pub y: f32,
    pub zoom: f32,
    pub viewport_w: f32,
    pub viewport_h: f32,
}

impl Default for CameraState {
    fn default() -> Self {
        Self {
            x: 200.0,
            y: 1800.0, // Nhìn vào căn cứ Player góc Tây-Nam
            zoom: 1.0,
            viewport_w: 1280.0,
            viewport_h: 720.0,
        }
    }
}
