use std::cell::RefCell;
use std::collections::VecDeque;
use wasm_bindgen::prelude::*;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

pub mod ai;
pub mod autoplay;
pub mod bridge;
pub mod components;
pub mod config;
pub mod fog;
pub mod formation;
pub mod map;
pub mod renderer;
pub mod resources;
pub mod rl;
pub mod simulation;
pub mod spatial_grid;

use bridge::*;
use components::*;
use config::GameConfig;
use fog::FogOfWar;
use resources::*;
use spatial_grid::SpatialGrid;

thread_local! {
    static ENGINE_STATE: RefCell<Option<GameEngine>> = const { RefCell::new(None) };
}

pub struct GameEngine {
    pub context: Option<CanvasRenderingContext2d>,
    pub canvas: Option<HtmlCanvasElement>,
    pub is_headless: bool,
    pub camera: CameraState,
    pub fog: FogOfWar,
    pub settings: GameSettings,
    pub config: GameConfig,

    // Thực thể ECS trong game
    pub units: Vec<Unit>,
    pub buildings: Vec<Building>,
    pub minerals: Vec<MineralNode>,
    pub obstacles: Vec<Obstacle>,
    pub projectiles: Vec<Projectile>,
    pub magic_circles: Vec<MagicCircle>,
    pub floating_texts: Vec<FloatingText>,
    pub spatial_grid: SpatialGrid,

    // Trạng thái kinh tế & công nghệ 2 phe chính (giữ nguyên để tương thích ngược)
    pub player_eco: PlayerEconomy,
    pub enemy_eco: PlayerEconomy,
    pub player_tech: TechTreeState,
    pub enemy_tech: TechTreeState,
    pub autoplay: crate::autoplay::AutoPlayState,
    pub enemy_autoplay: crate::autoplay::AutoPlayState,

    // Trạng thái mở rộng cho tối đa 8 Player & Đa Phe
    pub player_slots: [crate::components::PlayerSlotConfig; 8],
    pub extra_ecos: Vec<PlayerEconomy>, // Slot 2..7
    pub extra_techs: Vec<TechTreeState>, // Slot 2..7
    pub extra_autoplays: Vec<crate::autoplay::AutoPlayState>, // Slot 2..7
    pub extra_ai_states: Vec<crate::ai::AiState>, // Slot 2..7
    pub team_formations: [crate::components::FormationType; 8],

    // Đang chọn & Thao tác
    pub selected_unit_ids: Vec<u32>,
    pub selected_building_id: Option<u32>,
    pub selection_box: Option<(f32, f32, f32, f32)>, // x1, y1, x2, y2 (screen)
    pub build_mode: Option<BuildingType>,
    pub is_attack_move_mode: bool,
    pub is_minimap_dragging: bool,
    pub hovered_command_button: Option<usize>,

    // Điều khiển & Vòng lặp
    pub next_id: u32,
    pub game_time: f32,
    pub is_paused: bool,
    pub game_over: bool,
    pub mouse_screen_x: f32,
    pub mouse_screen_y: f32,
    pub is_mouse_down: bool,
    pub mouse_down_start: (f32, f32),
    pub keys_down: std::collections::HashSet<String>,

    // AI timer & Income tracker
    pub ai_think_timer: f32,
    pub player_ai_state: crate::ai::AiState,
    pub ai_state: crate::ai::AiState,
    pub sync_timer: f32,
    pub minerals_mined_accumulator: f32,
    pub income_calc_timer: f32,
    pub difficulty: String,

    // Kênh thông điệp
    pub pending_commands: VecDeque<ReactToEngineCommand>,
    pub outgoing_events: Vec<EngineToReactEvent>,
}

fn default_player_slots() -> [crate::components::PlayerSlotConfig; 8] {
    [
        crate::components::PlayerSlotConfig { slot_index: 0, active: true, faction: 0, is_human: true },
        crate::components::PlayerSlotConfig { slot_index: 1, active: true, faction: 1, is_human: false },
        crate::components::PlayerSlotConfig { slot_index: 2, active: false, faction: 1, is_human: false },
        crate::components::PlayerSlotConfig { slot_index: 3, active: false, faction: 1, is_human: false },
        crate::components::PlayerSlotConfig { slot_index: 4, active: false, faction: 1, is_human: false },
        crate::components::PlayerSlotConfig { slot_index: 5, active: false, faction: 1, is_human: false },
        crate::components::PlayerSlotConfig { slot_index: 6, active: false, faction: 1, is_human: false },
        crate::components::PlayerSlotConfig { slot_index: 7, active: false, faction: 1, is_human: false },
    ]
}

impl GameEngine {
    pub fn new(canvas: HtmlCanvasElement, ctx: CanvasRenderingContext2d) -> Self {
        let (minerals, obstacles) = map::generate_map_elements();
        let config = GameConfig::default();
        let player_eco = PlayerEconomy {
            minerals: config.economy.starting_minerals,
            energy: config.economy.starting_energy,
            supply: config.economy.starting_supply,
            max_supply: config.economy.base_max_supply,
            minerals_per_sec: 0.0,
            energy_per_sec: config.economy.base_energy_per_sec,
        };
        let enemy_eco = player_eco.clone();

        let mut extra_ecos = Vec::with_capacity(6);
        let mut extra_techs = Vec::with_capacity(6);
        let mut extra_autoplays = Vec::with_capacity(6);
        let mut extra_ai_states = Vec::with_capacity(6);
        for _ in 2..8 {
            extra_ecos.push(player_eco.clone());
            extra_techs.push(TechTreeState::default());
            extra_autoplays.push(crate::autoplay::AutoPlayState::default_for_enemy());
            extra_ai_states.push(crate::ai::AiState::default());
        }

        let mut engine = Self {
            context: Some(ctx),
            canvas: Some(canvas),
            is_headless: false,
            camera: CameraState::default(),
            fog: FogOfWar::new(),
            settings: GameSettings::default(),
            config,

            units: Vec::new(),
            buildings: Vec::new(),
            minerals,
            obstacles,
            projectiles: Vec::new(),
            magic_circles: Vec::new(),
            floating_texts: Vec::new(),
            spatial_grid: SpatialGrid::new(),

            player_eco,
            enemy_eco,
            player_tech: TechTreeState::default(),
            enemy_tech: TechTreeState::default(),
            autoplay: crate::autoplay::AutoPlayState::default(),
            enemy_autoplay: crate::autoplay::AutoPlayState::default_for_enemy(),

            player_slots: default_player_slots(),
            extra_ecos,
            extra_techs,
            extra_autoplays,
            extra_ai_states,
            team_formations: [crate::components::FormationType::Line; 8],

            selected_unit_ids: Vec::new(),
            selected_building_id: None,
            selection_box: None,
            build_mode: None,
            is_attack_move_mode: false,
            is_minimap_dragging: false,
            hovered_command_button: None,

            next_id: 100,
            game_time: 0.0,
            is_paused: true,
            game_over: false,
            mouse_screen_x: 0.0,
            mouse_screen_y: 0.0,
            is_mouse_down: false,
            mouse_down_start: (0.0, 0.0),
            keys_down: std::collections::HashSet::new(),

            ai_think_timer: 0.0,
            player_ai_state: crate::ai::AiState::default(),
            ai_state: crate::ai::AiState::default(),
            sync_timer: 0.0,
            minerals_mined_accumulator: 0.0,
            income_calc_timer: 0.0,
            difficulty: "normal".to_string(),
            pending_commands: VecDeque::new(),
            outgoing_events: Vec::new(),
        };

        engine.init_base();
        simulation::update_fog(&mut engine);
        engine
    }

    pub fn new_headless() -> Self {
        let (minerals, obstacles) = map::generate_map_elements();
        let config = GameConfig::default();
        let player_eco = PlayerEconomy {
            minerals: config.economy.starting_minerals,
            energy: config.economy.starting_energy,
            supply: config.economy.starting_supply,
            max_supply: config.economy.base_max_supply,
            minerals_per_sec: 0.0,
            energy_per_sec: config.economy.base_energy_per_sec,
        };
        let enemy_eco = player_eco.clone();

        let mut extra_ecos = Vec::with_capacity(6);
        let mut extra_techs = Vec::with_capacity(6);
        let mut extra_autoplays = Vec::with_capacity(6);
        let mut extra_ai_states = Vec::with_capacity(6);
        for _ in 2..8 {
            extra_ecos.push(player_eco.clone());
            extra_techs.push(TechTreeState::default());
            extra_autoplays.push(crate::autoplay::AutoPlayState::default_for_enemy());
            extra_ai_states.push(crate::ai::AiState::default());
        }

        let mut engine = Self {
            context: None,
            canvas: None,
            is_headless: true,
            camera: CameraState::default(),
            fog: FogOfWar::new(),
            settings: GameSettings::default(),
            config,

            units: Vec::new(),
            buildings: Vec::new(),
            minerals,
            obstacles,
            projectiles: Vec::new(),
            magic_circles: Vec::new(),
            floating_texts: Vec::new(),
            spatial_grid: SpatialGrid::new(),

            player_eco,
            enemy_eco,
            player_tech: TechTreeState::default(),
            enemy_tech: TechTreeState::default(),
            autoplay: crate::autoplay::AutoPlayState::default(),
            enemy_autoplay: crate::autoplay::AutoPlayState::default_for_enemy(),

            player_slots: default_player_slots(),
            extra_ecos,
            extra_techs,
            extra_autoplays,
            extra_ai_states,
            team_formations: [crate::components::FormationType::Line; 8],

            selected_unit_ids: Vec::new(),
            selected_building_id: None,
            selection_box: None,
            build_mode: None,
            is_attack_move_mode: false,
            is_minimap_dragging: false,
            hovered_command_button: None,

            next_id: 100,
            game_time: 0.0,
            is_paused: false,
            game_over: false,
            mouse_screen_x: 0.0,
            mouse_screen_y: 0.0,
            is_mouse_down: false,
            mouse_down_start: (0.0, 0.0),
            keys_down: std::collections::HashSet::new(),

            ai_think_timer: 0.0,
            player_ai_state: crate::ai::AiState::default(),
            ai_state: crate::ai::AiState::default(),
            sync_timer: 0.0,
            minerals_mined_accumulator: 0.0,
            income_calc_timer: 0.0,
            difficulty: "normal".to_string(),
            pending_commands: VecDeque::new(),
            outgoing_events: Vec::new(),
        };

        engine.init_base();
        engine
    }

    #[inline]
    pub fn get_faction(&self, team: Team) -> u8 {
        self.player_slots[team.index() % 8].faction
    }

    #[inline]
    pub fn are_allies(&self, a: Team, b: Team) -> bool {
        a == b || self.get_faction(a) == self.get_faction(b)
    }

    #[inline]
    pub fn are_hostile(&self, a: Team, b: Team) -> bool {
        !self.are_allies(a, b)
    }

    #[inline]
    pub fn is_slot_active(&self, team: Team) -> bool {
        self.player_slots[team.index() % 8].active
    }

    #[inline]
    pub fn base_pos(&self, team: Team) -> (f32, f32) {
        let spawns = map::get_spawn_positions();
        spawns[team.index() % 8]
    }

    #[inline]
    pub fn team_factions(&self) -> [u8; 8] {
        let mut factions = [0u8; 8];
        for i in 0..8 {
            factions[i] = self.player_slots[i].faction;
        }
        factions
    }

    pub fn team_eco(&self, team: Team) -> &PlayerEconomy {
        match team {
            Team::Player => &self.player_eco,
            Team::Enemy => &self.enemy_eco,
            _ => {
                let idx = team.index().saturating_sub(2);
                if idx < self.extra_ecos.len() {
                    &self.extra_ecos[idx]
                } else {
                    &self.enemy_eco
                }
            }
        }
    }

    pub fn team_eco_mut(&mut self, team: Team) -> &mut PlayerEconomy {
        match team {
            Team::Player => &mut self.player_eco,
            Team::Enemy => &mut self.enemy_eco,
            _ => {
                let idx = team.index().saturating_sub(2);
                if idx < self.extra_ecos.len() {
                    &mut self.extra_ecos[idx]
                } else {
                    &mut self.enemy_eco
                }
            }
        }
    }

    pub fn team_tech(&self, team: Team) -> &TechTreeState {
        match team {
            Team::Player => &self.player_tech,
            Team::Enemy => &self.enemy_tech,
            _ => {
                let idx = team.index().saturating_sub(2);
                if idx < self.extra_techs.len() {
                    &self.extra_techs[idx]
                } else {
                    &self.enemy_tech
                }
            }
        }
    }

    pub fn team_tech_mut(&mut self, team: Team) -> &mut TechTreeState {
        match team {
            Team::Player => &mut self.player_tech,
            Team::Enemy => &mut self.enemy_tech,
            _ => {
                let idx = team.index().saturating_sub(2);
                if idx < self.extra_techs.len() {
                    &mut self.extra_techs[idx]
                } else {
                    &mut self.enemy_tech
                }
            }
        }
    }

    pub fn team_autoplay(&self, team: Team) -> &crate::autoplay::AutoPlayState {
        match team {
            Team::Player => &self.autoplay,
            Team::Enemy => &self.enemy_autoplay,
            _ => {
                let idx = team.index().saturating_sub(2);
                if idx < self.extra_autoplays.len() {
                    &self.extra_autoplays[idx]
                } else {
                    &self.enemy_autoplay
                }
            }
        }
    }

    pub fn team_autoplay_mut(&mut self, team: Team) -> &mut crate::autoplay::AutoPlayState {
        match team {
            Team::Player => &mut self.autoplay,
            Team::Enemy => &mut self.enemy_autoplay,
            _ => {
                let idx = team.index().saturating_sub(2);
                if idx < self.extra_autoplays.len() {
                    &mut self.extra_autoplays[idx]
                } else {
                    &mut self.enemy_autoplay
                }
            }
        }
    }

    pub fn team_ai(&self, team: Team) -> &crate::ai::AiState {
        match team {
            Team::Player => &self.player_ai_state,
            Team::Enemy => &self.ai_state,
            _ => {
                let idx = team.index().saturating_sub(2);
                if idx < self.extra_ai_states.len() {
                    &self.extra_ai_states[idx]
                } else {
                    &self.ai_state
                }
            }
        }
    }

    pub fn team_ai_mut(&mut self, team: Team) -> &mut crate::ai::AiState {
        match team {
            Team::Player => &mut self.player_ai_state,
            Team::Enemy => &mut self.ai_state,
            _ => {
                let idx = team.index().saturating_sub(2);
                if idx < self.extra_ai_states.len() {
                    &mut self.extra_ai_states[idx]
                } else {
                    &mut self.ai_state
                }
            }
        }
    }

    #[inline]
    pub fn team_formation(&self, team: Team) -> crate::components::FormationType {
        self.team_formations[team.index() % 8]
    }

    #[inline]
    pub fn set_team_formation(&mut self, team: Team, formation: crate::components::FormationType) {
        self.team_formations[team.index() % 8] = formation;
    }

    /// Dọn sạch toàn bộ thực thể và tài nguyên của màn chơi cũ, chuẩn bị cho màn chơi mới
    pub fn reset_game(&mut self) {
        // 1. Khởi tạo lại tài nguyên khoáng sản & chướng ngại vật mới tinh
        let (minerals, obstacles) = map::generate_map_elements();
        self.minerals = minerals;
        self.obstacles = obstacles;

        // 2. Dọn sạch toàn bộ đơn vị, công trình, đạn và hiệu ứng chữ nổi
        self.units.clear();
        self.buildings.clear();
        self.projectiles.clear();
        self.magic_circles.clear();
        self.floating_texts.clear();
        self.spatial_grid.clear();

        // 3. Reset toàn bộ sương mù chiến trận (Fog of War)
        self.fog = FogOfWar::new();

        // 4. Reset tài nguyên kinh tế & cây công nghệ về ban đầu
        let default_eco = PlayerEconomy {
            minerals: self.config.economy.starting_minerals,
            energy: self.config.economy.starting_energy,
            supply: self.config.economy.starting_supply,
            max_supply: self.config.economy.base_max_supply,
            minerals_per_sec: 0.0,
            energy_per_sec: self.config.economy.base_energy_per_sec,
        };
        self.player_eco = default_eco.clone();
        self.enemy_eco = default_eco.clone();
        for eco in self.extra_ecos.iter_mut() {
            *eco = default_eco.clone();
        }

        self.player_tech = TechTreeState::default();
        self.enemy_tech = TechTreeState::default();
        for tech in self.extra_techs.iter_mut() {
            *tech = TechTreeState::default();
        }

        for ap in self.extra_autoplays.iter_mut() {
            *ap = crate::autoplay::AutoPlayState::default_for_enemy();
        }

        // 5. Reset camera về góc căn cứ Player nhưng giữ nguyên kích thước viewport màn hình hiện tại
        let current_viewport_w = self.camera.viewport_w;
        let current_viewport_h = self.camera.viewport_h;
        self.camera = CameraState::default();
        self.camera.viewport_w = current_viewport_w;
        self.camera.viewport_h = current_viewport_h;

        // 6. Xóa toàn bộ trạng thái chọn đơn vị, thao tác chuột & phím
        self.selected_unit_ids.clear();
        self.selected_building_id = None;
        self.selection_box = None;
        self.build_mode = None;
        self.is_attack_move_mode = false;
        self.is_minimap_dragging = false;
        self.hovered_command_button = None;
        self.mouse_screen_x = 0.0;
        self.mouse_screen_y = 0.0;
        self.is_mouse_down = false;
        self.mouse_down_start = (0.0, 0.0);
        self.keys_down.clear();

        self.team_formations = [crate::components::FormationType::Line; 8];

        // 7. Reset bộ đếm thời gian và ID, nạp bộ não phù hợp theo chế độ chơi
        self.next_id = 100;
        self.game_time = 0.0;
        self.is_paused = false;
        self.game_over = false;
        self.ai_think_timer = 0.0;

        let mode = crate::rl::AiGameMode::detect_from_slots(&self.player_slots);
        let mode_brain = if self.ai_state.training_enabled {
            self.ai_state.rl_brain.clone()
        } else {
            crate::rl::RlBrain::load_for_mode(mode)
        };
        let training_enabled = self.ai_state.training_enabled;

        self.player_ai_state = crate::ai::AiState::default();
        self.player_ai_state.rl_brain = mode_brain.clone();
        self.player_ai_state.training_enabled = training_enabled;

        self.ai_state = crate::ai::AiState::default();
        self.ai_state.rl_brain = mode_brain.clone();
        self.ai_state.training_enabled = training_enabled;

        for ai in self.extra_ai_states.iter_mut() {
            *ai = crate::ai::AiState::default();
            ai.rl_brain = mode_brain.clone();
            ai.training_enabled = training_enabled;
        }

        self.minerals_mined_accumulator = 0.0;
        self.income_calc_timer = 0.0;
        self.enemy_autoplay = crate::autoplay::AutoPlayState::default_for_enemy();

        // 8. Dọn dẹp sự kiện cũ còn sót lại trong hàng đợi
        self.outgoing_events.clear();

        // 9. Sinh lại Căn cứ chính và 3 Thợ mỏ khởi đầu cho tất cả slot active
        self.init_base();

        // 10. Cập nhật lại sương mù cho căn cứ ban đầu ngay tức thì (bỏ qua nếu headless)
        if !self.is_headless {
            simulation::update_fog(self);
        }
    }

    pub fn init_base(&mut self) {
        let spawns = map::get_spawn_positions();

        for i in 0..8 {
            if !self.player_slots[i].active {
                continue;
            }
            let team = Team::from_index(i);
            let (sx, sy) = spawns[i];
            let rally = map::get_default_rally_point(i);

            // Sinh CommandCenter
            let cc_id = self.alloc_id();
            let mut cc = self.config.create_building(
                cc_id,
                team,
                BuildingType::CommandCenter,
                sx,
                sy,
                true,
            );
            cc.rally_x = rally.0;
            cc.rally_y = rally.1;
            self.buildings.push(cc);

            // Sinh 3 Thợ mỏ khởi đầu
            let mineral_base_id = (i as u32 + 1) * 10;
            for k in 0..3 {
                let u_id = self.alloc_id();
                let offset_x = ((k as f32) - 1.0) * 35.0;
                let offset_y = -35.0;
                let mut u = self.config.create_unit(
                    u_id,
                    team,
                    UnitType::Engineer,
                    sx + offset_x,
                    sy + offset_y,
                );
                u.facing_right = rally.0 >= sx;

                if team == Team::Player {
                    // Thợ mỏ của Player chính tự động đào mỏ ngay
                    let target_mineral_id = mineral_base_id + (k as u32 % 3) + 1;
                    u.order = UnitOrder::Harvest {
                        target_mineral_id,
                        phase: HarvestPhase::MovingToNode,
                    };
                } else {
                    u.order = UnitOrder::Idle;
                }
                self.units.push(u);
            }
        }
    }

    pub fn alloc_id(&mut self) -> u32 {
        self.next_id += 1;
        self.next_id
    }

    pub fn screen_to_world(&self, sx: f32, sy: f32) -> (f32, f32) {
        let wx = self.camera.x + (sx - self.camera.viewport_w * 0.5) / self.camera.zoom;
        let wy = self.camera.y + (sy - self.camera.viewport_h * 0.5) / self.camera.zoom;
        (wx, wy)
    }

    pub fn world_to_screen(&self, wx: f32, wy: f32) -> (f32, f32) {
        let sx = (wx - self.camera.x) * self.camera.zoom + self.camera.viewport_w * 0.5;
        let sy = (wy - self.camera.y) * self.camera.zoom + self.camera.viewport_h * 0.5;
        (sx, sy)
    }
}

#[wasm_bindgen]
pub fn init_engine(canvas: HtmlCanvasElement) -> Result<(), JsValue> {
    let ctx = canvas
        .get_context("2d")?
        .unwrap()
        .dyn_into::<CanvasRenderingContext2d>()?;

    let engine = GameEngine::new(canvas, ctx);
    ENGINE_STATE.with(|state| {
        *state.borrow_mut() = Some(engine);
    });

    Ok(())
}

/// Vòng lặp 1: Cập nhật mô phỏng logic (AI, kinh tế, di chuyển, va chạm, đạn)
#[wasm_bindgen]
pub fn engine_tick(dt: f32) {
    ENGINE_STATE.with(|state| {
        if let Some(ref mut engine) = *state.borrow_mut() {
            simulation::step(engine, dt);
        }
    });
}

/// Vòng lặp 2: Vẽ khung hình lên Canvas (chạy theo requestAnimationFrame / VSync)
#[wasm_bindgen]
pub fn engine_render() {
    ENGINE_STATE.with(|state| {
        if let Some(ref mut engine) = *state.borrow_mut() {
            renderer::render(engine);
        }
    });
}

/// Giữ lại hàm cũ để đảm bảo tính tương thích ngược
#[wasm_bindgen]
pub fn engine_step(dt: f32) {
    ENGINE_STATE.with(|state| {
        if let Some(ref mut engine) = *state.borrow_mut() {
            simulation::step(engine, dt);
            renderer::render(engine);
        }
    });
}

#[wasm_bindgen]
pub fn send_command_to_engine(cmd_json: &str) {
    if let Ok(cmd) = serde_json::from_str::<ReactToEngineCommand>(cmd_json) {
        ENGINE_STATE.with(|state| {
            if let Some(ref mut engine) = *state.borrow_mut() {
                engine.pending_commands.push_back(cmd);
            }
        });
    }
}

#[wasm_bindgen]
pub fn poll_events_from_engine() -> String {
    ENGINE_STATE.with(|state| {
        if let Some(ref mut engine) = *state.borrow_mut() {
            if engine.outgoing_events.is_empty() {
                "[]".to_string()
            } else {
                let evs = std::mem::take(&mut engine.outgoing_events);
                serde_json::to_string(&evs).unwrap_or_else(|_| "[]".to_string())
            }
        } else {
            "[]".to_string()
        }
    })
}

#[wasm_bindgen]
pub fn resize_canvas(width: f32, height: f32) {
    ENGINE_STATE.with(|state| {
        if let Some(ref mut engine) = *state.borrow_mut() {
            if let Some(ref canvas) = engine.canvas {
                canvas.set_width(width as u32);
                canvas.set_height(height as u32);
            }
            engine.camera.viewport_w = width;
            engine.camera.viewport_h = height;
        }
    });
}

#[wasm_bindgen]
pub fn handle_mouse_down(button: i32, client_x: f32, client_y: f32) {
    ENGINE_STATE.with(|state| {
        if let Some(ref mut engine) = *state.borrow_mut() {
            simulation::handle_mouse_down(engine, button, client_x, client_y);
        }
    });
}

#[wasm_bindgen]
pub fn handle_mouse_up(button: i32, client_x: f32, client_y: f32) {
    ENGINE_STATE.with(|state| {
        if let Some(ref mut engine) = *state.borrow_mut() {
            simulation::handle_mouse_up(engine, button, client_x, client_y);
        }
    });
}

#[wasm_bindgen]
pub fn handle_mouse_move(client_x: f32, client_y: f32) {
    ENGINE_STATE.with(|state| {
        if let Some(ref mut engine) = *state.borrow_mut() {
            simulation::handle_mouse_move(engine, client_x, client_y);
        }
    });
}

#[wasm_bindgen]
pub fn handle_mouse_wheel(delta_y: f32) {
    ENGINE_STATE.with(|state| {
        if let Some(ref mut engine) = *state.borrow_mut() {
            let zoom_change = if delta_y > 0.0 { -0.1 } else { 0.1 };
            engine.camera.zoom = (engine.camera.zoom + zoom_change).clamp(0.5, 2.0);
        }
    });
}

#[wasm_bindgen]
pub fn handle_key_down(key: &str) {
    ENGINE_STATE.with(|state| {
        if let Some(ref mut engine) = *state.borrow_mut() {
            engine.keys_down.insert(key.to_string());
            simulation::handle_hotkeys(engine, key);
        }
    });
}

#[wasm_bindgen]
pub fn handle_key_up(key: &str) {
    ENGINE_STATE.with(|state| {
        if let Some(ref mut engine) = *state.borrow_mut() {
            engine.keys_down.remove(key);
        }
    });
}
