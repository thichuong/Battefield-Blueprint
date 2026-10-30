use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type")]
pub enum ReactToEngineCommand {
    StartGame {
        difficulty: String,
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
        game_time: f32,
        is_paused: bool,
    },
    OpenTechTreeModal,
    OpenSettingsModal,
    PlaySound {
        sound: String,
    },
    GameOver {
        player_won: bool,
        stats_json: String,
    },
}
