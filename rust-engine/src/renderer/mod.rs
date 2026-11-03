pub mod buildings;
pub mod command_card;
pub mod effects;
pub mod environment;
#[cfg(target_arch = "wasm32")]
pub mod gpu;
pub mod gpu_cache;
pub mod hud;
pub mod units;

pub use buildings::*;
pub use command_card::*;
pub use effects::*;
pub use environment::*;
#[cfg(target_arch = "wasm32")]
pub use gpu::WgpuRenderer;
pub use gpu_cache::*;
pub use hud::*;
pub use units::*;

use crate::components::Team;
use crate::map::{MAP_HEIGHT, MAP_WIDTH};
use crate::GameEngine;

pub fn get_team_color(engine: &GameEngine, team: Team) -> (&str, &str) {
    if team == Team::Player {
        (
            &engine.config.render.player_color,
            &engine.config.render.player_fill,
        )
    } else if engine.are_allies(Team::Player, team) {
        (
            &engine.config.render.ally_color,
            &engine.config.render.ally_fill,
        )
    } else {
        let palette = &engine.config.render.enemy_palette;
        if !palette.is_empty() {
            let idx = team.index() % palette.len();
            (&palette[idx].0, &palette[idx].1)
        } else {
            ("#f43f5e", "rgba(244, 63, 94, 0.12)")
        }
    }
}

pub fn render_hud_layer(engine: &GameEngine, ctx: &web_sys::CanvasRenderingContext2d) {
    let vw = engine.camera.viewport_w as f64;
    let vh = engine.camera.viewport_h as f64;
    let dpr = engine.camera.dpr.max(0.5) as f64;
    let _ = ctx.set_transform(dpr, 0.0, 0.0, dpr, 0.0, 0.0);
    ctx.clear_rect(0.0, 0.0, vw, vh);

    // Hiệu ứng chữ nổi bay lên (Floating numbers e.g. +10 💎)
    render_floating_texts(engine);

    // Khung kéo chuột chọn quân (Box Select)
    if let Some((sx1, sy1, sx2, sy2)) = engine.selection_box {
        ctx.set_stroke_style_str(&engine.config.render.map_border_color);
        ctx.set_line_width(1.5);
        let rx = sx1.min(sx2);
        let ry = sy1.min(sy2);
        let rw = (sx2 - sx1).abs();
        let rh = (sy2 - sy1).abs();
        ctx.stroke_rect(rx as f64, ry as f64, rw as f64, rh as f64);
        ctx.set_fill_style_str(&engine.config.render.player_fill);
        ctx.fill_rect(rx as f64, ry as f64, rw as f64, rh as f64);
    }

    // Con trỏ chế độ Attack-Move
    if engine.is_attack_move_mode {
        render_attack_move_cursor(engine);
    }

    // Toàn bộ In-game HUD
    render_in_game_hud(engine);

    // Bảng thông số hiệu năng FPS & Thời gian xử lý (Debug Performance Overlay)
    render_debug_overlay(engine);
}

pub fn render(engine: &mut GameEngine) {
    if engine.is_headless {
        return;
    }

    // Đồng bộ selected_unit_set (nếu kích thước hoặc nội dung khác nhau)
    if engine.selected_unit_set.len() != engine.selected_unit_ids.len() {
        engine.selected_unit_set.clear();
        for &id in &engine.selected_unit_ids {
            engine.selected_unit_set.insert(id);
        }
    }

    // 1. Nếu có WebGPU Renderer: Render toàn bộ thế giới game bằng WebGPU!
    #[cfg(target_arch = "wasm32")]
    if let Some(mut wgpu_renderer) = engine.wgpu_renderer.take() {
        let _ = wgpu_renderer.render_frame(engine);
        engine.wgpu_renderer = Some(wgpu_renderer);

        // Vẽ HUD lên UI Canvas (nếu có)
        if let Some(ui_ctx) = engine.ui_context.clone() {
            render_hud_layer(engine, &ui_ctx);
        }
        return;
    }

    // 2. Fallback: Nếu không có WebGPU, chạy Canvas 2D truyền thống
    if engine.context.is_none() {
        return;
    }

    // Tối ưu hóa: Cập nhật GPU Fog Texture chỉ khi sương mù thay đổi
    if engine.fog_dirty {
        if let Some(ref cache) = engine.gpu_cache {
            cache.update_fog_texture(&engine.fog.grid);
        }
        engine.fog_dirty = false;
    }

    // Đồng bộ selected_unit_set (nếu kích thước hoặc nội dung khác nhau)
    if engine.selected_unit_set.len() != engine.selected_unit_ids.len() {
        engine.selected_unit_set.clear();
        for &id in &engine.selected_unit_ids {
            engine.selected_unit_set.insert(id);
        }
    }

    let ctx = engine.context.as_ref().unwrap().clone();
    let vw = engine.camera.viewport_w;
    let vh = engine.camera.viewport_h;

    // Chuẩn hóa HiDPI (DPR scaling) để Canvas 2D render siêu sắc nét trên màn hình Retina / 4K
    let dpr = engine.camera.dpr.max(0.5) as f64;
    let _ = ctx.set_transform(dpr, 0.0, 0.0, dpr, 0.0, 0.0);

    // 1. Xóa màn hình với màu nền Blueprint Dark
    ctx.set_fill_style_str(&engine.config.render.bg_color);
    ctx.fill_rect(0.0, 0.0, vw as f64, vh as f64);

    ctx.save();

    // 2. Thiết lập Camera Matrix (Translate & Scale)
    let cx = engine.camera.x;
    let cy = engine.camera.y;
    let zoom = engine.camera.zoom;

    let _ = ctx.translate((vw * 0.5) as f64, (vh * 0.5) as f64);
    let _ = ctx.scale(zoom as f64, zoom as f64);
    let _ = ctx.translate((-cx) as f64, (-cy) as f64);

    // 3. Vẽ Lưới Tọa độ Blueprint (Blueprint Tactical Grid)
    if engine.settings.show_grid {
        render_blueprint_grid(engine);
    }

    // 4. Vẽ Ranh giới bản đồ
    ctx.set_stroke_style_str(&engine.config.render.map_border_color);
    ctx.set_line_width(3.0);
    ctx.stroke_rect(0.0, 0.0, MAP_WIDTH as f64, MAP_HEIGHT as f64);

    // 5. Vẽ Địa hình vật cản
    render_obstacles(engine);

    // 6. Vẽ Mỏ khoáng sản
    render_minerals(engine);

    // 6.5. Vẽ Vòng ma thuật hư không dưới đất (nằm dưới chân công trình và lính)
    render_magic_circles(engine);

    // 7. Vẽ Công trình
    render_buildings(engine);

    // 8. Vẽ Lính người que (2.5D Oblique Procedural Stickmen + Tia hàn)
    render_units(engine);

    // 9. Vẽ Đạn / Tia Laser / Vết chém
    render_projectiles(engine);

    // 9.5. Vẽ Hiệu ứng Hạt (Particles - Mảnh vụn nổ, khói lửa, tia lửa)
    render_particles(engine);

    // 10. Vẽ Bóng công trình khi đang đặt móng (Ghost Building)
    if let Some(b_type) = engine.build_mode {
        render_build_ghost(engine, b_type);
    }

    // 11. Vẽ Sương mù chiến tranh (Fog of War)
    render_fog_of_war(engine);

    ctx.restore();

    // ==========================================
    // CÁC THÀNH PHẦN VẼ TRÊN MÀN HÌNH (SCREEN-SPACE UI / BEVY HUD)
    // ==========================================

    // Hiệu ứng chữ nổi bay lên (Floating numbers e.g. +10 💎)
    render_floating_texts(engine);

    // Khung kéo chuột chọn quân (Box Select)
    if let Some((sx1, sy1, sx2, sy2)) = engine.selection_box {
        ctx.set_stroke_style_str(&engine.config.render.map_border_color);
        ctx.set_line_width(1.5);
        let rx = sx1.min(sx2);
        let ry = sy1.min(sy2);
        let rw = (sx2 - sx1).abs();
        let rh = (sy2 - sy1).abs();
        ctx.stroke_rect(rx as f64, ry as f64, rw as f64, rh as f64);
        ctx.set_fill_style_str(&engine.config.render.player_fill);
        ctx.fill_rect(rx as f64, ry as f64, rw as f64, rh as f64);
    }

    // Con trỏ chế độ Attack-Move
    if engine.is_attack_move_mode {
        render_attack_move_cursor(engine);
    }

    // Toàn bộ In-game HUD
    render_in_game_hud(engine);

    // Bảng thông số hiệu năng FPS & Thời gian xử lý (Debug Performance Overlay)
    render_debug_overlay(engine);
}
