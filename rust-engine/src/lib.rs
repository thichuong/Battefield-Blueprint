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

    let engine = GameEngine::new(canvas, ctx);
    ENGINE_STATE.with(|state| {
        *state.borrow_mut() = Some(engine);
    });

    Ok(())
}

fn get_performance_now() -> f64 {
    web_sys::window()
        .and_then(|w| w.performance())
        .map(|p| p.now())
        .unwrap_or(0.0)
}

/// Vòng lặp 1: Cập nhật mô phỏng logic (AI, kinh tế, di chuyển, va chạm, đạn)
#[wasm_bindgen]
pub fn engine_tick(dt: f32) {
    ENGINE_STATE.with(|state| {
        if let Some(ref mut engine) = *state.borrow_mut() {
            let t0 = get_performance_now();
            simulation::step(engine, dt);
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
            if engine.last_render_time > 0.0 {
                let dt_ms = (now - engine.last_render_time).max(1.0);
                let instant_fps = (1000.0 / dt_ms).min(360.0) as f32;
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
            if let Some(ref canvas) = engine.canvas {
                canvas.set_width((width * dpr_val) as u32);
                canvas.set_height((height * dpr_val) as u32);
            }
            engine.camera.viewport_w = width;
            engine.camera.viewport_h = height;
            engine.camera.dpr = dpr_val;
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
            if key == "F3" || key == "`" || key == "~" {
                engine.show_debug_overlay = !engine.show_debug_overlay;
            }
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
