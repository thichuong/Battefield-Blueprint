use std::cell::RefCell;
use wasm_bindgen::prelude::*;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

pub mod ai;
pub mod autoplay;
pub mod bridge;
pub mod components;
pub mod config;
pub mod cuda_accel;
pub mod engine;
pub mod fog;
pub mod formation;
pub mod map;
pub mod renderer;
pub mod resources;
pub mod rl;
pub mod simulation;
pub mod spatial_grid;

use bridge::*;
use config::{
    COMMAND_CARD_HEIGHT, COMMAND_CARD_WIDTH, MINIMAP_SIZE, SELECTION_PANEL_HEIGHT,
    SELECTION_PANEL_WIDTH,
};
pub use engine::GameEngine;

thread_local! {
    static ENGINE_STATE: RefCell<Option<GameEngine>> = const { RefCell::new(None) };
}

#[wasm_bindgen]
pub fn init_engine(canvas: HtmlCanvasElement) -> Result<(), JsValue> {
    let ctx = canvas
        .get_context("2d")?
        .unwrap()
        .dyn_into::<CanvasRenderingContext2d>()?;

    let mut engine = GameEngine::new(canvas, ctx);
    engine.gpu_cache = crate::renderer::gpu_cache::GpuCacheManager::new(&engine.config);
    ENGINE_STATE.with(|state| {
        *state.borrow_mut() = Some(engine);
    });

    Ok(())
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub async fn init_engine_webgpu(
    world_canvas: HtmlCanvasElement,
    ui_canvas: Option<HtmlCanvasElement>,
    top_bar_canvas: Option<HtmlCanvasElement>,
    minimap_canvas: Option<HtmlCanvasElement>,
    selection_canvas: Option<HtmlCanvasElement>,
    command_card_canvas: Option<HtmlCanvasElement>,
    overlay_canvas: Option<HtmlCanvasElement>,
) -> Result<bool, JsValue> {
    let config = crate::config::GameConfig::default();
    let cache = match crate::renderer::gpu_cache::GpuCacheManager::new(&config) {
        Some(c) => c,
        None => return Ok(false),
    };

    let wgpu_renderer =
        match crate::renderer::gpu::WgpuRenderer::new(world_canvas.clone(), &cache).await {
            Ok(r) => Some(r),
            Err(err) => {
                web_sys::console::warn_1(
                    &format!("WebGPU init failed, falling back to Canvas 2D: {}", err).into(),
                );
                None
            }
        };

    let is_webgpu = wgpu_renderer.is_some();

    let get_2d = |c: &Option<HtmlCanvasElement>| -> Option<CanvasRenderingContext2d> {
        c.as_ref()
            .and_then(|cv| cv.get_context("2d").ok().flatten())
            .and_then(|ctx| ctx.dyn_into::<CanvasRenderingContext2d>().ok())
    };

    let ui_ctx = get_2d(&ui_canvas);
    let top_bar_ctx = get_2d(&top_bar_canvas);
    let minimap_ctx = get_2d(&minimap_canvas);
    let selection_ctx = get_2d(&selection_canvas);
    let command_card_ctx = get_2d(&command_card_canvas);
    let overlay_ctx = get_2d(&overlay_canvas);

    let fallback_ctx = if !is_webgpu {
        world_canvas
            .get_context("2d")
            .ok()
            .flatten()
            .and_then(|c| c.dyn_into::<CanvasRenderingContext2d>().ok())
    } else {
        None
    };

    let mut engine = if let Some(fctx) = fallback_ctx {
        GameEngine::new(world_canvas.clone(), fctx)
    } else {
        let dummy_canvas: HtmlCanvasElement = web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .create_element("canvas")
            .unwrap()
            .dyn_into()
            .unwrap();
        let dummy_ctx: CanvasRenderingContext2d = dummy_canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into()
            .unwrap();
        GameEngine::new(world_canvas.clone(), dummy_ctx)
    };

    engine.wgpu_renderer = wgpu_renderer;
    engine.ui_context = ui_ctx;
    engine.top_bar_ctx = top_bar_ctx;
    engine.minimap_ctx = minimap_ctx;
    engine.selection_ctx = selection_ctx;
    engine.command_card_ctx = command_card_ctx;
    engine.overlay_ctx = overlay_ctx;

    engine.top_bar_canvas = top_bar_canvas;
    engine.minimap_canvas = minimap_canvas;
    engine.selection_canvas = selection_canvas;
    engine.command_card_canvas = command_card_canvas;
    engine.overlay_canvas = overlay_canvas;

    engine.gpu_cache = Some(cache);

    ENGINE_STATE.with(|state| {
        *state.borrow_mut() = Some(engine);
    });

    Ok(is_webgpu)
}

#[cfg(not(target_arch = "wasm32"))]
#[wasm_bindgen]
pub async fn init_engine_webgpu(
    _world_canvas: HtmlCanvasElement,
    _ui_canvas: Option<HtmlCanvasElement>,
    _top_bar_canvas: Option<HtmlCanvasElement>,
    _minimap_canvas: Option<HtmlCanvasElement>,
    _selection_canvas: Option<HtmlCanvasElement>,
    _command_card_canvas: Option<HtmlCanvasElement>,
    _overlay_canvas: Option<HtmlCanvasElement>,
) -> Result<bool, JsValue> {
    Ok(false)
}

fn get_performance_now() -> f64 {
    web_sys::window()
        .and_then(|w| w.performance())
        .map(|p| p.now())
        .unwrap_or(0.0)
}

/// Vòng lặp 1 (Độc lập): Cập nhật mô phỏng logic cố định 60Hz theo thời gian thực đơn điệu `now_ms`
/// Hoàn toàn tách rời khỏi vòng lặp Render — tốc độ game luôn chuẩn 1x ở mọi mức FPS.
#[wasm_bindgen]
pub fn engine_sim_step(now_ms: f64) -> u32 {
    ENGINE_STATE.with(|state| {
        let mut borrowed = state.borrow_mut();
        let engine = match borrowed.as_mut() {
            Some(e) => e,
            None => return 0,
        };

        // Luôn xử lý hàng đợi lệnh từ React ngay lập tức (kể cả khi đang tạm dừng)
        if !engine.is_headless {
            simulation::process_commands(engine);
        }

        if engine.is_paused || engine.game_over {
            engine.sim_accumulator = 0.0;
            engine.last_frame_time = now_ms;
            return if engine.outgoing_events.is_empty() {
                0
            } else {
                1
            };
        }

        // Tính Delta Time đơn điệu chuẩn xác (TUYỆT ĐỐI không ép clamp dương khi dt <= 0)
        let delta_sec = if engine.last_frame_time > 0.0 {
            if now_ms > engine.last_frame_time {
                ((now_ms - engine.last_frame_time) / 1000.0).min(0.1) as f32
            } else {
                0.0
            }
        } else {
            1.0 / 60.0
        };

        if engine.last_frame_time <= 0.0 || now_ms > engine.last_frame_time {
            engine.last_frame_time = now_ms;
        }

        const FIXED_TIMESTEP: f32 = 1.0 / 60.0;
        const MAX_SUB_STEPS: usize = 5;
        engine.sim_accumulator += delta_sec;

        let measure_debug = engine.show_debug_overlay;
        let t0 = if measure_debug {
            get_performance_now()
        } else {
            0.0
        };

        let mut sub_steps = 0;
        while engine.sim_accumulator >= FIXED_TIMESTEP && sub_steps < MAX_SUB_STEPS {
            simulation::step(engine, FIXED_TIMESTEP);
            engine.sim_accumulator -= FIXED_TIMESTEP;
            sub_steps += 1;
        }
        if sub_steps == MAX_SUB_STEPS {
            engine.sim_accumulator = 0.0;
        }

        if sub_steps > 0 {
            engine.top_bar_dirty = true;
            engine.minimap_dirty = true;
            engine.selection_dirty = true;
            engine.command_card_dirty = true;
            engine.hud_dirty = true;
        }

        if measure_debug && sub_steps > 0 {
            let t1 = get_performance_now();
            engine.sim_time_ms = (t1 - t0) as f32;
        }

        if engine.outgoing_events.is_empty() {
            0
        } else {
            1
        }
    })
}

/// Vòng lặp 2 (Độc lập): Chỉ dựng hình (Render) và nội suy khung hình mượt mà theo FPS
/// Hoàn toàn không can thiệp hay cộng dồn vào bộ đếm thời gian mô phỏng!
#[wasm_bindgen]
pub fn engine_render_frame(now_ms: f64, mouse_x: f32, mouse_y: f32) -> u32 {
    ENGINE_STATE.with(|state| {
        let mut borrowed = state.borrow_mut();
        let engine = match borrowed.as_mut() {
            Some(e) => e,
            None => return 0,
        };

        // 1. Gom tọa độ chuột phục vụ hover, kéo khung chọn, minimap mượt ở FPS cao
        if (engine.mouse_screen_x - mouse_x).abs() > 0.5
            || (engine.mouse_screen_y - mouse_y).abs() > 0.5
        {
            let old_hover = engine.hovered_command_button;
            simulation::handle_mouse_move(engine, mouse_x, mouse_y);
            if engine.hovered_command_button != old_hover {
                engine.command_card_dirty = true;
            }
            if engine.selection_box.is_some() || engine.is_attack_move_mode {
                engine.overlay_dirty = true;
            }
            if engine.is_minimap_dragging {
                engine.minimap_dirty = true;
            }
            engine.hud_dirty = true;
        }

        // 2. Đo FPS thực tế của vòng lặp Render
        if engine.last_render_time > 0.0 && now_ms > engine.last_render_time {
            let dt_ms = (now_ms - engine.last_render_time).max(0.1);
            let instant_fps = (1000.0 / dt_ms).min(2000.0) as f32;
            engine.fps = engine.fps * 0.90 + instant_fps * 0.10;
        }
        if engine.last_render_time <= 0.0 || now_ms > engine.last_render_time {
            engine.last_render_time = now_ms;
        }

        // 3. Tính hệ số nội suy khung hình (alpha) từ thời gian trôi qua kể từ tick 60Hz gần nhất
        const FIXED_TIMESTEP: f32 = 1.0 / 60.0;
        let extra_sec = if !engine.is_paused
            && !engine.game_over
            && engine.last_frame_time > 0.0
            && now_ms > engine.last_frame_time
        {
            (((now_ms - engine.last_frame_time) / 1000.0) as f32).min(FIXED_TIMESTEP)
        } else {
            0.0
        };
        let alpha = ((engine.sim_accumulator + extra_sec) / FIXED_TIMESTEP).clamp(0.0, 1.0);
        engine.render_alpha = alpha;

        // 4. Dựng hình WebGPU / Canvas 2D
        let measure_debug = engine.show_debug_overlay;
        let rt0 = if measure_debug {
            get_performance_now()
        } else {
            0.0
        };
        renderer::render(engine);
        if measure_debug {
            let rt1 = get_performance_now();
            engine.render_time_ms = (rt1 - rt0) as f32;
        }

        if engine.outgoing_events.is_empty() {
            0
        } else {
            1
        }
    })
}

/// Hàm kết hợp giữ tương thích ngược (đã sửa lỗi clamp dương gây tăng tốc theo FPS)
#[wasm_bindgen]
pub fn engine_frame(now_ms: f64, mouse_x: f32, mouse_y: f32, should_render: bool) -> u32 {
    let sim_flags = engine_sim_step(now_ms);
    let render_flags = if should_render {
        engine_render_frame(now_ms, mouse_x, mouse_y)
    } else {
        0
    };
    sim_flags | render_flags
}

/// Vòng lặp 1: Cập nhật mô phỏng logic (AI, kinh tế, di chuyển, va chạm, đạn)
#[wasm_bindgen]
pub fn engine_tick(dt: f32) {
    ENGINE_STATE.with(|state| {
        if let Some(ref mut engine) = *state.borrow_mut() {
            let t0 = get_performance_now();
            simulation::step(engine, dt);
            engine.top_bar_dirty = true;
            engine.minimap_dirty = true;
            engine.selection_dirty = true;
            engine.command_card_dirty = true;
            engine.hud_dirty = true;
            let t1 = get_performance_now();
            engine.sim_time_ms = (t1 - t0) as f32;
        }
    });
}

/// Vòng lặp 2: Vẽ khung hình lên Canvas (chạy theo requestAnimationFrame / VSync)
/// Nhận hệ số `alpha` [0.0..1.0] để nội suy vị trí mượt mà giữa 2 tick mô phỏng
#[wasm_bindgen]
pub fn engine_render(alpha: Option<f32>) {
    let a = alpha.unwrap_or(1.0).clamp(0.0, 1.0);
    ENGINE_STATE.with(|state| {
        if let Some(ref mut engine) = *state.borrow_mut() {
            let now = get_performance_now();
            if engine.last_render_time > 0.0 && now > engine.last_render_time {
                let dt_ms = (now - engine.last_render_time).max(0.1);
                let instant_fps = (1000.0 / dt_ms).min(2000.0) as f32;
                engine.fps = engine.fps * 0.90 + instant_fps * 0.10;
            }
            engine.last_render_time = now;

            let t0 = get_performance_now();
            engine.render_alpha = a;
            renderer::render(engine);
            let t1 = get_performance_now();
            engine.render_time_ms = (t1 - t0) as f32;
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
pub fn get_game_config_json() -> String {
    ENGINE_STATE.with(|state| {
        if let Some(ref engine) = *state.borrow() {
            engine.config.to_json()
        } else {
            crate::config::GameConfig::default().to_json()
        }
    })
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
pub fn resize_canvas(width: f32, height: f32, dpr: Option<f32>) {
    let dpr_val = dpr.unwrap_or(1.0).max(0.5);
    ENGINE_STATE.with(|state| {
        if let Some(ref mut engine) = *state.borrow_mut() {
            let pixel_w = (width * dpr_val) as u32;
            let pixel_h = (height * dpr_val) as u32;
            if let Some(ref canvas) = engine.canvas {
                canvas.set_width(pixel_w);
                canvas.set_height(pixel_h);
            }
            #[cfg(target_arch = "wasm32")]
            if let Some(ref mut wgpu_rend) = engine.wgpu_renderer {
                wgpu_rend.resize(pixel_w, pixel_h);
            }
            engine.camera.viewport_w = width;
            engine.camera.viewport_h = height;
            engine.camera.dpr = dpr_val;

            // Resize và set DPR scale cho các sub-canvas
            if let Some(ref c) = engine.top_bar_canvas {
                c.set_width(pixel_w);
                c.set_height((48.0 * dpr_val) as u32);
            }
            if let Some(ref c) = engine.minimap_canvas {
                c.set_width((MINIMAP_SIZE * dpr_val) as u32);
                c.set_height((MINIMAP_SIZE * dpr_val) as u32);
            }
            if let Some(ref c) = engine.selection_canvas {
                c.set_width((SELECTION_PANEL_WIDTH * dpr_val) as u32);
                c.set_height((SELECTION_PANEL_HEIGHT * dpr_val) as u32);
            }
            if let Some(ref c) = engine.command_card_canvas {
                c.set_width((COMMAND_CARD_WIDTH * dpr_val) as u32);
                c.set_height((COMMAND_CARD_HEIGHT * dpr_val) as u32);
            }
            if let Some(ref c) = engine.overlay_canvas {
                c.set_width(pixel_w);
                c.set_height(pixel_h);
            }

            engine.top_bar_dirty = true;
            engine.minimap_dirty = true;
            engine.selection_dirty = true;
            engine.command_card_dirty = true;
            engine.overlay_dirty = true;
            engine.hud_dirty = true;
        }
    });
}

#[wasm_bindgen]
pub fn handle_mouse_down(button: i32, client_x: f32, client_y: f32) {
    ENGINE_STATE.with(|state| {
        if let Some(ref mut engine) = *state.borrow_mut() {
            simulation::handle_mouse_down(engine, button, client_x, client_y);
            engine.top_bar_dirty = true;
            engine.minimap_dirty = true;
            engine.selection_dirty = true;
            engine.command_card_dirty = true;
            engine.overlay_dirty = true;
            engine.hud_dirty = true;
        }
    });
}

#[wasm_bindgen]
pub fn handle_mouse_up(button: i32, client_x: f32, client_y: f32) {
    ENGINE_STATE.with(|state| {
        if let Some(ref mut engine) = *state.borrow_mut() {
            simulation::handle_mouse_up(engine, button, client_x, client_y);
            engine.top_bar_dirty = true;
            engine.minimap_dirty = true;
            engine.selection_dirty = true;
            engine.command_card_dirty = true;
            engine.overlay_dirty = true;
            engine.hud_dirty = true;
        }
    });
}

#[wasm_bindgen]
pub fn handle_mouse_move(client_x: f32, client_y: f32) {
    ENGINE_STATE.with(|state| {
        if let Some(ref mut engine) = *state.borrow_mut() {
            let old_hover = engine.hovered_command_button;
            simulation::handle_mouse_move(engine, client_x, client_y);
            if engine.hovered_command_button != old_hover {
                engine.command_card_dirty = true;
            }
            if engine.selection_box.is_some() || engine.is_attack_move_mode {
                engine.overlay_dirty = true;
            }
            if engine.is_minimap_dragging {
                engine.minimap_dirty = true;
            }
            engine.hud_dirty = true;
        }
    });
}

#[wasm_bindgen]
pub fn handle_mouse_wheel(delta_y: f32) {
    ENGINE_STATE.with(|state| {
        if let Some(ref mut engine) = *state.borrow_mut() {
            let zoom_change = if delta_y > 0.0 { -0.1 } else { 0.1 };
            engine.camera.zoom = (engine.camera.zoom + zoom_change).clamp(0.5, 2.0);
            engine.minimap_dirty = true;
            engine.hud_dirty = true;
        }
    });
}

#[wasm_bindgen]
pub fn handle_key_down(key: &str) {
    ENGINE_STATE.with(|state| {
        if let Some(ref mut engine) = *state.borrow_mut() {
            if key == "F3" || key == "`" || key == "~" {
                engine.show_debug_overlay = !engine.show_debug_overlay;
            }
            engine.keys_down.insert(key.to_string());
            simulation::handle_hotkeys(engine, key);
            engine.top_bar_dirty = true;
            engine.minimap_dirty = true;
            engine.selection_dirty = true;
            engine.command_card_dirty = true;
            engine.overlay_dirty = true;
            engine.hud_dirty = true;
        }
    });
}

#[wasm_bindgen]
pub fn handle_key_up(key: &str) {
    ENGINE_STATE.with(|state| {
        if let Some(ref mut engine) = *state.borrow_mut() {
            engine.keys_down.remove(key);
            engine.hud_dirty = true;
        }
    });
}
