use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type")]
pub enum ReactToEngineCommand {
    StartGame {
        difficulty: String,
        #[serde(default)]
        slots: Vec<crate::components::PlayerSlotConfig>,
    },
    SetPaused {
        paused: bool,
    },
    UpgradeTech {
        tech_id: String,
    },
    UpdateSettings {
        sound_volume: f32,
        camera_speed: f32,
        auto_pause: bool,
        show_grid: bool,
    },
    RestartGame,
    UpdateAutoPlayConfig {
        config_json: String,
    },
    TogglePriorityTech {
        tech_id: String,
    },
    SetFormation {
        formation: String,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type")]
pub enum EngineToReactEvent {
    SyncState {
        player_minerals: f32,
        player_energy: f32,
        player_supply: u32,
        player_max_supply: u32,
        tech_state: String, // serialized json
        autoplay_state: String, // serialized json
        player_formation: String,
        game_time: f32,
        is_paused: bool,
    },
    OpenTechTreeModal,
    OpenSettingsModal,
    OpenAutoPlayModal,
    PlaySound {
        sound: String,
    },
    GameOver {
        player_won: bool,
        stats_json: String,
    },
}
