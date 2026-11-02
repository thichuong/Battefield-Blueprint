use crate::components::{BuildingType, DamageType, Team};
use crate::renderer::buildings::draw_building_graphic;
use crate::GameEngine;

pub fn render_magic_circles(engine: &GameEngine) {
    let ctx = match engine.context.as_ref() {
        Some(c) => c,
        None => return,
    };
    for circle in &engine.magic_circles {
        if !engine
            .camera
            .is_point_visible(circle.x, circle.y, circle.radius + 20.0)
        {
            continue;
        }

        let is_visible = engine.are_allies(Team::Player, circle.team)
            || engine.fog.is_visible(circle.x, circle.y);
        if !is_visible {
            continue;
        }

        let life_ratio = (circle.lifetime / circle.max_lifetime).clamp(0.0, 1.0) as f64;
        let alpha = (life_ratio * 0.9).min(0.75);
        let rot = (engine.game_time * 1.5) as f64;
        let r = circle.radius as f64;
        let cx = circle.x as f64;
        let cy = circle.y as f64;

        ctx.save();

        // 1. Nền năng lượng tím hư không
        ctx.set_fill_style_str(&format!("rgba(168, 85, 247, {})", alpha * 0.22));
        ctx.begin_path();
        let _ = ctx.arc(cx, cy, r, 0.0, std::f64::consts::PI * 2.0);
        ctx.fill();

        // 2. Viền ngoài kép
        ctx.set_stroke_style_str(&format!("rgba(192, 132, 252, {})", alpha));
        ctx.set_line_width(2.0);
        ctx.begin_path();
        let _ = ctx.arc(cx, cy, r, 0.0, std::f64::consts::PI * 2.0);
        ctx.stroke();

        ctx.set_line_width(1.0);
        ctx.begin_path();
        let _ = ctx.arc(cx, cy, r - 5.0, 0.0, std::f64::consts::PI * 2.0);
        ctx.stroke();

        // 3. Hình lục giác cổ ngữ xoay
        ctx.set_stroke_style_str(&format!("rgba(232, 121, 249, {})", alpha * 0.85));
        ctx.set_line_width(1.5);
        ctx.begin_path();
        for i in 0..6 {
            let a = rot + (i as f64) * std::f64::consts::PI / 3.0;
            let px = cx + (r - 9.0) * a.cos();
            let py = cy + (r - 9.0) * a.sin();
            if i == 0 {
                ctx.move_to(px, py);
            } else {
                ctx.line_to(px, py);
            }
        }
        ctx.close_path();
        ctx.stroke();

        // 4. Ngôi sao 6 cánh lồng xoay ngược chiều
        let rev_rot = -rot * 0.8;
        ctx.set_stroke_style_str(&format!("rgba(168, 85, 247, {})", alpha * 0.7));
        ctx.begin_path();
        for i in 0..3 {
            let a = rev_rot + (i as f64) * std::f64::consts::PI * 2.0 / 3.0;
            let px = cx + (r * 0.62) * a.cos();
            let py = cy + (r * 0.62) * a.sin();
            if i == 0 {
                ctx.move_to(px, py);
            } else {
                ctx.line_to(px, py);
            }
        }
        ctx.close_path();
        ctx.stroke();

        ctx.begin_path();
        for i in 0..3 {
            let a = rev_rot + std::f64::consts::PI + (i as f64) * std::f64::consts::PI * 2.0 / 3.0;
            let px = cx + (r * 0.62) * a.cos();
            let py = cy + (r * 0.62) * a.sin();
            if i == 0 {
                ctx.move_to(px, py);
            } else {
                ctx.line_to(px, py);
            }
        }
        ctx.close_path();
        ctx.stroke();

        // 5. Tâm pháp trận tỏa sáng
        ctx.set_fill_style_str(&format!("rgba(244, 114, 182, {})", alpha));
        ctx.begin_path();
        let _ = ctx.arc(cx, cy, 4.5, 0.0, std::f64::consts::PI * 2.0);
        ctx.fill();

        ctx.restore();
    }
}

pub fn render_projectiles(engine: &GameEngine) {
    let ctx = match engine.context.as_ref() {
        Some(c) => c,
        None => return,
    };
    for p in &engine.projectiles {
        // Đòn cận chiến không hiển thị tia đạn
        if p.is_melee {
            continue;
        }

        if !engine.camera.is_point_visible(p.x, p.y, 40.0)
            && !engine.camera.is_point_visible(p.target_x, p.target_y, 40.0)
        {
            continue;
        }

        if p.is_laser {
            ctx.set_stroke_style_str(if p.team == Team::Player {
                "#38bdf8"
            } else {
                "#f43f5e"
            });
            ctx.set_line_width(2.5);
            ctx.begin_path();
            ctx.move_to(p.x as f64, p.y as f64);
            ctx.line_to(p.target_x as f64, p.target_y as f64);
            ctx.stroke();
        } else {
            let color = match p.damage_type {
                DamageType::Explosive => "#fb923c",
                DamageType::Arcane => "#c084fc",
                DamageType::Energy => "#38bdf8",
                DamageType::Physical => "#facc15",
            };
            ctx.set_fill_style_str(color);
            ctx.begin_path();
            let radius = if p.aoe_radius > 0.0 { 4.5 } else { 3.0 };
            let _ = ctx.arc(
                p.x as f64,
                p.y as f64,
                radius,
                0.0,
                std::f64::consts::PI * 2.0,
            );
            ctx.fill();

            if p.aoe_radius > 0.0 {
                ctx.set_stroke_style_str(color);
                ctx.set_line_width(1.0);
                ctx.stroke();
            }
        }
    }
}

pub fn render_build_ghost(engine: &GameEngine, b_type: BuildingType) {
    let ctx = match engine.context.as_ref() {
        Some(c) => c,
        None => return,
    };
    let (wx, wy) = engine.screen_to_world(engine.mouse_screen_x, engine.mouse_screen_y);
    let (w, h) = engine.config.get_building_dimensions(b_type);

    let bx = (wx - w * 0.5) as f64;
    let by = (wy - h * 0.5) as f64;

    ctx.set_fill_style_str("rgba(0, 229, 255, 0.25)");
    ctx.fill_rect(bx, by, w as f64, h as f64);

    ctx.set_stroke_style_str(&engine.config.render.player_color);
    ctx.set_line_width(2.0);
    ctx.stroke_rect(bx, by, w as f64, h as f64);

    // Vẽ hình 2D tĩnh bóng mờ
    draw_building_graphic(
        ctx,
        b_type,
        wx as f64,
        wy as f64,
        w as f64,
        h as f64,
        &engine.config.render.player_color,
    );

    // Tên công trình bên dưới ghost
    ctx.set_font("bold 10px monospace");
    ctx.set_fill_style_str(&engine.config.render.player_color);
    let ghost_title = engine.config.get_building_short_name(b_type);
    let text_w = (ghost_title.len() as f64) * 6.2;
    let _ = ctx.fill_text(
        ghost_title,
        wx as f64 - text_w * 0.5,
        by + (h as f64) + 13.0,
    );

    // Hiển thị vòng tròn tầm bắn của công trình khi đang đặt móng
    let range = engine
        .config
        .get_building(b_type)
        .map(|c| c.turret_range)
        .unwrap_or(0.0);
    if range > 0.0 {
        ctx.set_stroke_style_str("rgba(0, 229, 255, 0.4)");
        ctx.set_line_width(1.5);
        ctx.begin_path();
        let _ = ctx.arc(
            wx as f64,
            wy as f64,
            range as f64,
            0.0,
            std::f64::consts::PI * 2.0,
        );
        ctx.stroke();
    }
}

pub fn render_floating_texts(engine: &GameEngine) {
    let ctx = match engine.context.as_ref() {
        Some(c) => c,
        None => return,
    };
    ctx.set_font("bold 13px monospace");
    for ft in &engine.floating_texts {
        let (sx, sy) = engine.world_to_screen(ft.x, ft.y);
        let alpha = (ft.lifetime / ft.max_lifetime).clamp(0.0, 1.0);
        ctx.set_fill_style_str(&format!("rgba(0, 229, 255, {})", alpha));
        let _ = ctx.fill_text(&ft.text, sx as f64 - 16.0, sy as f64);
    }
}

pub fn render_attack_move_cursor(engine: &GameEngine) {
    let ctx = match engine.context.as_ref() {
        Some(c) => c,
        None => return,
    };
    let mx = engine.mouse_screen_x as f64;
    let my = engine.mouse_screen_y as f64;

    ctx.set_stroke_style_str("#ef4444");
    ctx.set_line_width(2.0);
    ctx.begin_path();
    let _ = ctx.arc(mx, my, 14.0, 0.0, std::f64::consts::PI * 2.0);
    ctx.move_to(mx - 20.0, my);
    ctx.line_to(mx + 20.0, my);
    ctx.move_to(mx, my - 20.0);
    ctx.line_to(mx, my + 20.0);
    ctx.stroke();

    ctx.set_font("bold 11px monospace");
    ctx.set_fill_style_str("#ef4444");
    let _ = ctx.fill_text("ATTACK-MOVE", mx + 18.0, my + 4.0);
}
