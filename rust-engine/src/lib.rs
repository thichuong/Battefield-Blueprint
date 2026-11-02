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
