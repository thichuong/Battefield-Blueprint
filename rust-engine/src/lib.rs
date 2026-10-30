use std::cell::RefCell;
use std::collections::VecDeque;
use wasm_bindgen::prelude::*;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

pub mod ai;
pub mod bridge;
pub mod components;
pub mod fog;
pub mod map;
pub mod renderer;
pub mod resources;
pub mod simulation;
pub mod spatial_grid;

use bridge::*;
use components::*;
use fog::FogOfWar;
use resources::*;
use spatial_grid::SpatialGrid;

thread_local! {
    static ENGINE_STATE: RefCell<Option<GameEngine>> = const { RefCell::new(None) };
}

pub struct GameEngine {
    pub context: CanvasRenderingContext2d,
    pub canvas: HtmlCanvasElement,
    pub camera: CameraState,
    pub fog: FogOfWar,
    pub settings: GameSettings,

    // Thực thể ECS trong game
    pub units: Vec<Unit>,
    pub buildings: Vec<Building>,
    pub minerals: Vec<MineralNode>,
    pub obstacles: Vec<Obstacle>,
    pub projectiles: Vec<Projectile>,
    pub floating_texts: Vec<FloatingText>,
    pub spatial_grid: SpatialGrid,

    // Trạng thái kinh tế & công nghệ 2 phe
    pub player_eco: PlayerEconomy,
    pub enemy_eco: PlayerEconomy,
    pub player_tech: TechTreeState,
    pub enemy_tech: TechTreeState,

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
    pub sync_timer: f32,
    pub minerals_mined_accumulator: f32,
    pub income_calc_timer: f32,
    pub difficulty: String,

    // Kênh thông điệp
    pub pending_commands: VecDeque<ReactToEngineCommand>,
    pub outgoing_events: Vec<EngineToReactEvent>,
}

impl GameEngine {
    pub fn new(canvas: HtmlCanvasElement, ctx: CanvasRenderingContext2d) -> Self {
        let (minerals, obstacles) = map::generate_map_elements();
        let mut engine = Self {
            context: ctx,
            canvas,
            camera: CameraState::default(),
            fog: FogOfWar::new(),
            settings: GameSettings::default(),

            units: Vec::new(),
            buildings: Vec::new(),
            minerals,
            obstacles,
            projectiles: Vec::new(),
            floating_texts: Vec::new(),
            spatial_grid: SpatialGrid::new(),

            player_eco: PlayerEconomy::default(),
            enemy_eco: PlayerEconomy::default(),
            player_tech: TechTreeState::default(),
            enemy_tech: TechTreeState::default(),

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
        self.floating_texts.clear();
        self.spatial_grid.clear();

        // 3. Reset toàn bộ sương mù chiến trận (Fog of War)
        self.fog = FogOfWar::new();

        // 4. Reset tài nguyên kinh tế & cây công nghệ về ban đầu
        self.player_eco = PlayerEconomy::default();
        self.enemy_eco = PlayerEconomy::default();
        self.player_tech = TechTreeState::default();
        self.enemy_tech = TechTreeState::default();

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

        // 7. Reset bộ đếm thời gian và ID
        self.next_id = 100;
        self.game_time = 0.0;
        self.is_paused = false;
        self.game_over = false;
        self.ai_think_timer = 0.0;
        self.minerals_mined_accumulator = 0.0;
        self.income_calc_timer = 0.0;

        // 8. Dọn dẹp sự kiện cũ còn sót lại trong hàng đợi
        self.outgoing_events.clear();

        // 9. Sinh lại Căn cứ chính và 3 Thợ mỏ khởi đầu cho Player & AI
        self.init_base();

        // 10. Cập nhật lại sương mù cho căn cứ ban đầu ngay tức thì
        simulation::update_fog(self);
    }

    pub fn init_base(&mut self) {
        // Căn cứ Player (Góc Tây-Nam)
        let p_cc_id = self.alloc_id();
        self.buildings.push(Building {
            id: p_cc_id,
            team: Team::Player,
            building_type: BuildingType::CommandCenter,
            x: 250.0,
            y: 2050.0,
            width: 90.0,
            height: 90.0,
            hp: 2000.0,
            max_hp: 2000.0,
            damage_type: DamageType::Physical,
            resistances: Resistances {
                physical: 0.25,
                energy: 0.15,
                explosive: 0.20,
                arcane: 0.10,
            },
            build_progress: 1.0,
            attack_cooldown: 0.0,
            current_cooldown: 0.0,
            rally_x: 360.0,
            rally_y: 2050.0,
            training_queue: Vec::new(),
            training_timer: 0.0,
        });

        // 3 Thợ mỏ khởi đầu cho Player (Tự động đi đào mỏ ngay khi vào trận)
        for i in 0..3 {
            let u_id = self.alloc_id();
            let mineral_target_id = (i as u32 % 3) + 1;
            self.units.push(Unit {
                id: u_id,
                team: Team::Player,
                unit_type: UnitType::Engineer,
                x: 230.0 + (i as f32 * 35.0),
                y: 1980.0,
                target_x: 230.0 + (i as f32 * 35.0),
                target_y: 1980.0,
                speed: 120.0,
                hp: 90.0,
                max_hp: 90.0,
                attack_range: 30.0,
                attack_damage: 5.0,
                damage_type: DamageType::Physical,
                aoe_radius: 0.0,
                resistances: Resistances {
                    physical: 0.05,
                    energy: 0.0,
                    explosive: 0.0,
                    arcane: 0.0,
                },
                attack_cooldown: 1.0,
                current_cooldown: 0.0,
                walk_cycle: 0.0,
                facing_right: true,
                order: UnitOrder::Harvest {
                    target_mineral_id: mineral_target_id,
                    phase: HarvestPhase::MovingToNode,
                },
                carried_minerals: 0.0,
            });
        }

        // Căn cứ Enemy (Góc Đông-Bắc)
        let e_cc_id = self.alloc_id();
        self.buildings.push(Building {
            id: e_cc_id,
            team: Team::Enemy,
            building_type: BuildingType::CommandCenter,
            x: 2150.0,
            y: 350.0,
            width: 90.0,
            height: 90.0,
            hp: 2000.0,
            max_hp: 2000.0,
            damage_type: DamageType::Physical,
            resistances: Resistances {
                physical: 0.25,
                energy: 0.15,
                explosive: 0.20,
                arcane: 0.10,
            },
            build_progress: 1.0,
            attack_cooldown: 0.0,
            current_cooldown: 0.0,
            rally_x: 2050.0,
            rally_y: 350.0,
            training_queue: Vec::new(),
            training_timer: 0.0,
        });

        // 3 Thợ mỏ cho Enemy
        for i in 0..3 {
            let u_id = self.alloc_id();
            self.units.push(Unit {
                id: u_id,
                team: Team::Enemy,
                unit_type: UnitType::Engineer,
                x: 2120.0 - (i as f32 * 35.0),
                y: 420.0,
                target_x: 2120.0 - (i as f32 * 35.0),
                target_y: 420.0,
                speed: 120.0,
                hp: 90.0,
                max_hp: 90.0,
                attack_range: 30.0,
                attack_damage: 5.0,
                damage_type: DamageType::Physical,
                aoe_radius: 0.0,
                resistances: Resistances {
                    physical: 0.05,
                    energy: 0.0,
                    explosive: 0.0,
                    arcane: 0.0,
                },
                attack_cooldown: 1.0,
                current_cooldown: 0.0,
                walk_cycle: 0.0,
                facing_right: false,
                order: UnitOrder::Idle,
                carried_minerals: 0.0,
            });
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
            engine.canvas.set_width(width as u32);
            engine.canvas.set_height(height as u32);
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
