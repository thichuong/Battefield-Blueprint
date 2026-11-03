use crate::components::{BuildingType, ProjectileVisual, Team};
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

        if let Some(ref cache) = engine.gpu_cache {
            ctx.save();
            ctx.set_global_alpha(alpha);
            let _ = ctx.translate(cx, cy);
            let _ = ctx.rotate(rot);
            let scale = r / 85.0;
            let _ = ctx.scale(scale, scale);
            let _ =
                ctx.draw_image_with_html_canvas_element(&cache.magic_circle_canvas, -96.0, -96.0);
            ctx.restore();
            continue;
        }

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
    let alpha = engine.render_alpha;

    for p in &engine.projectiles {
        // Đòn cận chiến không hiển thị tia đạn
        if p.is_melee {
            continue;
        }

        // Tọa độ nội suy của đạn
        let draw_x = p.prev_x + (p.x - p.prev_x) * alpha;
        let draw_y = p.prev_y + (p.y - p.prev_y) * alpha;

        if !engine.camera.is_point_visible(draw_x, draw_y, 40.0)
            && !engine.camera.is_point_visible(p.target_x, p.target_y, 40.0)
        {
            continue;
        }

        // Không vẽ đạn nếu cả điểm xuất phát/hiện tại và đích đến đều nằm ngoài tầm nhìn sương mù
        let is_visible = engine.are_allies(Team::Player, p.team)
            || engine.fog.is_visible(draw_x, draw_y)
            || engine.fog.is_visible(p.target_x, p.target_y);
        if !is_visible {
            continue;
        }

        if p.is_laser || p.visual == ProjectileVisual::Laser {
            let (laser_color, _) = crate::renderer::get_team_color(engine, p.team);
            ctx.set_stroke_style_str(laser_color);
            ctx.set_line_width(2.5);
            ctx.begin_path();
            ctx.move_to(draw_x as f64, draw_y as f64);
            ctx.line_to(p.target_x as f64, p.target_y as f64);
            ctx.stroke();
            continue;
        }

        ctx.save();
        let _ = ctx.translate(draw_x as f64, draw_y as f64);
        let _ = ctx.rotate(p.angle as f64);

        if let Some(ref cache) = engine.gpu_cache {
            match p.visual {
                ProjectileVisual::Bullet => {
                    let _ =
                        ctx.draw_image_with_html_canvas_element(&cache.bullet_canvas, -20.0, -8.0);
                }
                ProjectileVisual::Rocket => {
                    let _ =
                        ctx.draw_image_with_html_canvas_element(&cache.rocket_canvas, -18.0, -9.0);
                }
                ProjectileVisual::FlyingSword => {
                    let _ = ctx.draw_image_with_html_canvas_element(
                        &cache.flying_sword_canvas,
                        -18.0,
                        -8.0,
                    );
                }
                ProjectileVisual::Laser => {}
            }
            ctx.restore();
            continue;
        }

        match p.visual {
            ProjectileVisual::Bullet => {
                // 1. Rifleman & Đạn súng thường: Tia đạn (Bullet Tracer)
                // Vệt mờ đuôi đạn
                ctx.set_stroke_style_str("rgba(250, 204, 21, 0.45)");
                ctx.set_line_width(3.0);
                ctx.begin_path();
                ctx.move_to(-14.0, 0.0);
                ctx.line_to(0.0, 0.0);
                ctx.stroke();

                // Thân tia đạn vàng sáng
                ctx.set_stroke_style_str("#facc15");
                ctx.set_line_width(2.0);
                ctx.begin_path();
                ctx.move_to(-9.0, 0.0);
                ctx.line_to(3.0, 0.0);
                ctx.stroke();

                // Lõi sáng trắng rực tại đầu đạn
                ctx.set_stroke_style_str("#ffffff");
                ctx.set_line_width(1.2);
                ctx.begin_path();
                ctx.move_to(-3.0, 0.0);
                ctx.line_to(3.0, 0.0);
                ctx.stroke();

                // Đầu mũi đạn điểm sáng nhỏ
                ctx.set_fill_style_str("#ffffff");
                ctx.begin_path();
                let _ = ctx.arc(3.0, 0.0, 1.2, 0.0, std::f64::consts::PI * 2.0);
                ctx.fill();
            }
            ProjectileVisual::Rocket => {
                // 2. DarkRocketeer: Quả tên lửa (Rocket/Missile)
                // Luồng lửa phản lực ở đuôi
                ctx.set_fill_style_str("#ea580c");
                ctx.begin_path();
                ctx.move_to(-6.0, -2.2);
                ctx.line_to(-12.0, 0.0);
                ctx.line_to(-6.0, 2.2);
                ctx.close_path();
                ctx.fill();

                ctx.set_fill_style_str("#fef08a");
                ctx.begin_path();
                ctx.move_to(-6.0, -1.2);
                ctx.line_to(-9.5, 0.0);
                ctx.line_to(-6.0, 1.2);
                ctx.close_path();
                ctx.fill();

                // Cánh đuôi định hướng (Fins)
                ctx.set_stroke_style_str("#c2410c");
                ctx.set_line_width(1.6);
                ctx.begin_path();
                ctx.move_to(-6.0, -4.0);
                ctx.line_to(-3.0, -2.0);
                ctx.line_to(-3.0, 2.0);
                ctx.line_to(-6.0, 4.0);
                ctx.stroke();

                // Thân tên lửa hình khối trụ
                ctx.set_fill_style_str("#475569");
                ctx.fill_rect(-6.0, -2.0, 9.0, 4.0);
                ctx.set_stroke_style_str("#ea580c");
                ctx.set_line_width(1.2);
                ctx.stroke_rect(-6.0, -2.0, 9.0, 4.0);

                // Đầu đạn nhọn hình nón màu đỏ rực
                ctx.set_fill_style_str("#ef4444");
                ctx.begin_path();
                ctx.move_to(3.0, -2.2);
                ctx.line_to(7.5, 0.0);
                ctx.line_to(3.0, 2.2);
                ctx.close_path();
                ctx.fill();

                // Đốm sáng phản quang ở chóp đạn
                ctx.set_fill_style_str("#fca5a5");
                ctx.begin_path();
                let _ = ctx.arc(6.0, -0.6, 0.8, 0.0, std::f64::consts::PI * 2.0);
                ctx.fill();
            }
            ProjectileVisual::FlyingSword => {
                // 3. TaoistSwordmaster: Cổ phi kiếm ngự chân khí
                // Vệt kiếm khí hào quang lướt theo sau
                ctx.set_stroke_style_str("rgba(56, 189, 248, 0.35)");
                ctx.set_line_width(3.2);
                ctx.begin_path();
                ctx.move_to(-12.0, 0.0);
                ctx.line_to(0.0, 0.0);
                ctx.stroke();

                // Quả táo chuôi kiếm (Pommel)
                ctx.set_fill_style_str("#facc15");
                ctx.begin_path();
                let _ = ctx.arc(-6.5, 0.0, 1.2, 0.0, std::f64::consts::PI * 2.0);
                ctx.fill();

                // Chuôi kiếm
                ctx.set_stroke_style_str("#64748b");
                ctx.set_line_width(1.8);
                ctx.begin_path();
                ctx.move_to(-6.5, 0.0);
                ctx.line_to(-4.0, 0.0);
                ctx.stroke();

                // Thanh chắn kiếm chữ thập (Crossguard) vàng hoàng kim
                ctx.set_stroke_style_str("#facc15");
                ctx.set_line_width(1.8);
                ctx.begin_path();
                ctx.move_to(-4.0, -3.2);
                ctx.line_to(-4.0, 3.2);
                ctx.stroke();

                // Lưỡi phi kiếm xanh lam sắc nét
                ctx.set_stroke_style_str("#38bdf8");
                ctx.set_line_width(2.2);
                ctx.begin_path();
                ctx.move_to(-4.0, 0.0);
                ctx.line_to(7.5, 0.0);
                ctx.stroke();

                // Sống kiếm và mũi kiếm sáng trắng chân khí
                ctx.set_stroke_style_str("#ffffff");
                ctx.set_line_width(1.0);
                ctx.begin_path();
                ctx.move_to(-3.5, 0.0);
                ctx.line_to(7.5, 0.0);
                ctx.stroke();

                // Chân khí tỏa sáng nơi mũi kiếm
                ctx.set_fill_style_str("#e0f2fe");
                ctx.begin_path();
                let _ = ctx.arc(7.5, 0.0, 1.0, 0.0, std::f64::consts::PI * 2.0);
                ctx.fill();
            }
            ProjectileVisual::Laser => {}
        }

        ctx.restore();
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

    // Tên công trình bên dưới ghost (căn giữa chuẩn xác)
    ctx.set_font("bold 10px monospace");
    ctx.set_text_align("center");
    ctx.set_fill_style_str(&engine.config.render.player_color);
    let ghost_title = engine.config.get_building_short_name(b_type);
    let _ = ctx.fill_text(ghost_title, wx as f64, by + (h as f64) + 13.0);
    ctx.set_text_align("left");

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
    let ctx = match engine.ui_context.as_ref().or(engine.context.as_ref()) {
        Some(c) => c,
        None => return,
    };
    ctx.set_font("bold 13px monospace");
    for ft in &engine.floating_texts {
        if !engine.camera.is_point_visible(ft.x, ft.y, 40.0) {
            continue;
        }
        if !engine.fog.is_visible(ft.x, ft.y) {
            continue;
        }
        let (sx, sy) = engine.world_to_screen(ft.x, ft.y);
        let alpha = (ft.lifetime / ft.max_lifetime).clamp(0.0, 1.0);
        ctx.set_fill_style_str(&format!("rgba(0, 229, 255, {})", alpha));
        let _ = ctx.fill_text(&ft.text, sx as f64 - 16.0, sy as f64);
    }
}

pub fn render_attack_move_cursor(engine: &GameEngine) {
    let ctx = match engine.ui_context.as_ref().or(engine.context.as_ref()) {
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

pub fn render_particles(engine: &GameEngine) {
    let ctx = match engine.context.as_ref() {
        Some(c) => c,
        None => return,
    };
    for p in &engine.particles {
        if !engine.camera.is_point_visible(p.x, p.y, 20.0) {
            continue;
        }
        if !engine.fog.is_visible(p.x, p.y) {
            continue;
        }
        let alpha = (p.lifetime / p.max_lifetime).clamp(0.0, 1.0);
        let sz = (p.size * alpha).max(0.5) as f64;

        if let Some(ref cache) = engine.gpu_cache {
            use crate::components::ParticleColor;
            let p_canvas = match p.color {
                ParticleColor::Blue => &cache.particle_blue,
                ParticleColor::Cyan => &cache.particle_cyan,
                ParticleColor::Yellow => &cache.particle_yellow,
                ParticleColor::Orange => &cache.particle_orange,
                ParticleColor::Red => &cache.particle_red,
                ParticleColor::Purple => &cache.particle_purple,
                ParticleColor::Green => &cache.particle_green,
                ParticleColor::Slate => &cache.particle_slate,
                ParticleColor::White => &cache.particle_white,
            };
            let d = sz * 2.0;
            let _ = ctx.draw_image_with_html_canvas_element_and_dw_and_dh(
                p_canvas,
                (p.x as f64) - sz,
                (p.y as f64) - sz,
                d,
                d,
            );
            continue;
        }

        ctx.set_fill_style_str(p.color.to_hex());
        ctx.begin_path();
        let _ = ctx.arc(p.x as f64, p.y as f64, sz, 0.0, std::f64::consts::PI * 2.0);
        ctx.fill();
    }
}
