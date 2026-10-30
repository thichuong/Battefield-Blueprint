use crate::components::*;
use crate::fog::Visibility;
use crate::map::{MAP_HEIGHT, MAP_WIDTH};
use crate::GameEngine;
use web_sys::CanvasRenderingContext2d;

pub fn render(engine: &mut GameEngine) {
    let ctx = &engine.context;
    let vw = engine.camera.viewport_w;
    let vh = engine.camera.viewport_h;

    // 1. Xóa màn hình với màu nền Blueprint Dark
    ctx.set_fill_style_str("#030d1a");
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
    ctx.set_stroke_style_str("#00e5ff");
    ctx.set_line_width(3.0);
    ctx.stroke_rect(0.0, 0.0, MAP_WIDTH as f64, MAP_HEIGHT as f64);

    // 5. Vẽ Địa hình vật cản
    render_obstacles(engine);

    // 6. Vẽ Mỏ khoáng sản
    render_minerals(engine);

    // 7. Vẽ Công trình
    render_buildings(engine);

    // 8. Vẽ Lính người que (2.5D Oblique Procedural Stickmen + Tia hàn)
    render_units(engine);

    // 9. Vẽ Đạn / Tia Laser / Vết chém
    render_projectiles(engine);

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
        ctx.set_stroke_style_str("#00e5ff");
        ctx.set_line_width(1.5);
        let rx = sx1.min(sx2);
        let ry = sy1.min(sy2);
        let rw = (sx2 - sx1).abs();
        let rh = (sy2 - sy1).abs();
        ctx.stroke_rect(rx as f64, ry as f64, rw as f64, rh as f64);
        ctx.set_fill_style_str("rgba(0, 229, 255, 0.1)");
        ctx.fill_rect(rx as f64, ry as f64, rw as f64, rh as f64);
    }

    // Con trỏ chế độ Attack-Move
    if engine.is_attack_move_mode {
        render_attack_move_cursor(engine);
    }

    // Toàn bộ In-game HUD
    render_in_game_hud(engine);
}

fn render_blueprint_grid(engine: &GameEngine) {
    let ctx = &engine.context;
    ctx.set_stroke_style_str("#0a223e");
    ctx.set_line_width(1.0);
    ctx.begin_path();

    let grid_size = 80.0;
    let (min_x, min_y, max_x, max_y) = engine.camera.viewport_bounds(grid_size);
    let start_x = ((min_x / grid_size).floor() * grid_size).max(0.0);
    let end_x = ((max_x / grid_size).ceil() * grid_size).min(MAP_WIDTH);
    let start_y = ((min_y / grid_size).floor() * grid_size).max(0.0);
    let end_y = ((max_y / grid_size).ceil() * grid_size).min(MAP_HEIGHT);

    let mut x = start_x;
    while x <= end_x {
        ctx.move_to(x as f64, start_y as f64);
        ctx.line_to(x as f64, end_y as f64);
        x += grid_size;
    }
    let mut y = start_y;
    while y <= end_y {
        ctx.move_to(start_x as f64, y as f64);
        ctx.line_to(end_x as f64, y as f64);
        y += grid_size;
    }
    ctx.stroke();
}

fn render_obstacles(engine: &GameEngine) {
    let ctx = &engine.context;
    for obs in &engine.obstacles {
        if !engine.camera.is_rect_visible(obs.x, obs.y, obs.width, obs.height, 50.0) {
            continue;
        }

        ctx.set_fill_style_str("#071a2e");
        ctx.fill_rect(
            obs.x as f64,
            obs.y as f64,
            obs.width as f64,
            obs.height as f64,
        );

        ctx.set_stroke_style_str("#1e40af");
        ctx.set_line_width(1.5);
        ctx.stroke_rect(
            obs.x as f64,
            obs.y as f64,
            obs.width as f64,
            obs.height as f64,
        );
    }
}

fn render_minerals(engine: &GameEngine) {
    let ctx = &engine.context;
    for m in &engine.minerals {
        if !engine.camera.is_point_visible(m.x, m.y, 60.0) {
            continue;
        }

        // Tinh thể phát sáng Neon Cyan
        ctx.set_stroke_style_str("#38bdf8");
        ctx.set_fill_style_str("rgba(56, 189, 248, 0.28)");
        ctx.set_line_width(2.0);

        ctx.begin_path();
        ctx.move_to(m.x as f64, (m.y - 18.0) as f64);
        ctx.line_to((m.x + 16.0) as f64, (m.y - 5.0) as f64);
        ctx.line_to((m.x + 12.0) as f64, (m.y + 16.0) as f64);
        ctx.line_to((m.x - 12.0) as f64, (m.y + 16.0) as f64);
        ctx.line_to((m.x - 16.0) as f64, (m.y - 5.0) as f64);
        ctx.close_path();
        ctx.fill();
        ctx.stroke();

        // Nhãn trữ lượng
        ctx.set_font("bold 10px monospace");
        ctx.set_fill_style_str("#7dd3fc");
        let _ = ctx.fill_text(
            &format!("{:.0}💎", m.amount),
            (m.x - 16.0) as f64,
            (m.y + 30.0) as f64,
        );
    }
}

fn render_buildings(engine: &GameEngine) {
    let ctx = &engine.context;
    for b in &engine.buildings {
        let bw = b.width;
        let bh = b.height;
        let bx_f32 = b.x - bw * 0.5;
        let by_f32 = b.y - bh * 0.5;
        if !engine.camera.is_rect_visible(bx_f32, by_f32, bw, bh, 50.0) {
            continue;
        }

        let is_visible = b.team == Team::Player || engine.fog.is_visible(b.x, b.y);
        if !is_visible {
            continue;
        }

        let is_selected = engine.selected_building_id == Some(b.id);
        let color_neon = if b.team == Team::Player {
            "#00e5ff"
        } else {
            "#f43f5e"
        };
        let color_fill = if b.team == Team::Player {
            "rgba(0, 229, 255, 0.12)"
        } else {
            "rgba(244, 63, 94, 0.12)"
        };

        let bx = (b.x - b.width * 0.5) as f64;
        let by = (b.y - b.height * 0.5) as f64;
        let bw = b.width as f64;
        let bh = b.height as f64;

        // Khung nhà Blueprint
        ctx.set_fill_style_str(color_fill);
        ctx.fill_rect(bx, by, bw, bh);

        ctx.set_stroke_style_str(if is_selected { "#ffffff" } else { color_neon });
        ctx.set_line_width(if is_selected { 2.5 } else { 1.5 });
        ctx.stroke_rect(bx, by, bw, bh);

        // Hiển thị vòng tròn tầm bắn của công trình khi được chọn
        if is_selected {
            let b_cfg = engine.config.get_building(b.building_type);
            let range = b_cfg.map(|c| c.turret_range).unwrap_or(0.0);
            if range > 0.0 {
                ctx.set_stroke_style_str("rgba(0, 229, 255, 0.45)");
                ctx.set_line_width(1.5);
                ctx.begin_path();
                let _ = ctx.arc(b.x as f64, b.y as f64, range as f64, 0.0, std::f64::consts::PI * 2.0);
                ctx.stroke();
            }
        }

        // Ký hiệu góc kỹ thuật
        draw_blueprint_brackets(ctx, bx, by, bw, bh, color_neon);

        // Hiệu ứng khói / tia lửa nếu công trình hư hỏng nặng (< 45% HP)
        if b.hp < b.max_hp * 0.45 && b.build_progress >= 1.0 {
            ctx.set_stroke_style_str("#f97316");
            ctx.set_line_width(1.5);
            ctx.begin_path();
            ctx.move_to(bx + bw * 0.3, by + bh * 0.4);
            ctx.line_to(bx + bw * 0.4, by + bh * 0.2);
            ctx.line_to(bx + bw * 0.35, by + bh * 0.1);
            ctx.stroke();
        }

        // Tên công trình & Tiến độ
        ctx.set_font("bold 11px monospace");
        ctx.set_fill_style_str(color_neon);
        let title = match b.building_type {
            BuildingType::CommandCenter => "HQ COMMAND",
            BuildingType::SupplyDepot => "SUPPLY DEPOT",
            BuildingType::PowerReactor => "REACTOR",
            BuildingType::Barracks => "BARRACKS",
            BuildingType::TurretTower => "TURRET",
            BuildingType::EnergyConverter => "CONVERTER",
        };
        let _ = ctx.fill_text(title, bx + 6.0, by + 18.0);

        // Hiệu ứng lõi cho Trạm Chuyển Hóa Năng Lượng
        if b.building_type == BuildingType::EnergyConverter && b.build_progress >= 1.0 {
            let cx = b.x as f64;
            let cy = (b.y + 3.0) as f64;
            let is_active = if b.team == Team::Player {
                engine.player_eco.energy >= 1000.0
            } else {
                engine.enemy_eco.energy >= 1000.0
            };

            if is_active {
                // Vòng xoay năng lượng đang chuyển hóa (Màu hổ phách / vàng kim)
                let pulse = ((engine.game_time * 5.0).sin() * 2.5) as f64;
                ctx.set_stroke_style_str("#fbbf24");
                ctx.set_line_width(1.8);
                ctx.begin_path();
                let _ = ctx.arc(cx, cy, 14.0 + pulse * 0.5, 0.0, std::f64::consts::PI * 2.0);
                ctx.stroke();

                ctx.set_fill_style_str("rgba(251, 191, 36, 0.35)");
                ctx.begin_path();
                let _ = ctx.arc(cx, cy, (8.0 + pulse).max(4.0), 0.0, std::f64::consts::PI * 2.0);
                ctx.fill();

                // Tia chuyển đổi xoay
                let angle = (engine.game_time * 3.0) as f64;
                ctx.set_stroke_style_str("#fef08a");
                ctx.set_line_width(1.5);
                ctx.begin_path();
                ctx.move_to(cx + angle.cos() * 14.0, cy + angle.sin() * 14.0);
                ctx.line_to(cx - angle.cos() * 14.0, cy - angle.sin() * 14.0);
                ctx.move_to(cx + (angle + 1.57).cos() * 14.0, cy + (angle + 1.57).sin() * 14.0);
                ctx.line_to(cx - (angle + 1.57).cos() * 14.0, cy - (angle + 1.57).sin() * 14.0);
                ctx.stroke();

                ctx.set_font("bold 9px monospace");
                ctx.set_fill_style_str("#fbbf24");
                let _ = ctx.fill_text("⚡➔💎", cx - 13.0, cy + 22.0);
            } else {
                // Trạng thái STANDBY khi Năng lượng < 1000
                ctx.set_stroke_style_str("#64748b");
                ctx.set_line_width(1.2);
                ctx.begin_path();
                let _ = ctx.arc(cx, cy, 12.0, 0.0, std::f64::consts::PI * 2.0);
                ctx.stroke();

                ctx.set_font("bold 8px monospace");
                ctx.set_fill_style_str("#94a3b8");
                let _ = ctx.fill_text("STANDBY", cx - 18.0, cy + 22.0);
            }
        }

        // Thanh tiến độ xây dựng hoặc Thanh máu
        if b.build_progress < 1.0 {
            ctx.set_fill_style_str("#334155");
            ctx.fill_rect(bx, by - 12.0, bw, 6.0);
            ctx.set_fill_style_str("#fbbf24");
            ctx.fill_rect(bx, by - 12.0, bw * (b.build_progress as f64), 6.0);
        } else {
            // Thanh máu HP bar
            let hp_pct = (b.hp / b.max_hp).clamp(0.0, 1.0) as f64;
            ctx.set_fill_style_str("#1e293b");
            ctx.fill_rect(bx, by - 10.0, bw, 5.0);
            ctx.set_fill_style_str(if hp_pct > 0.4 { "#22c55e" } else { "#ef4444" });
            ctx.fill_rect(bx, by - 10.0, bw * hp_pct, 5.0);
        }

        // Vòng tròn điểm tập kết (Rally Point)
        if is_selected && b.team == Team::Player {
            ctx.set_stroke_style_str("#22c55e");
            ctx.set_line_width(1.0);
            ctx.begin_path();
            ctx.move_to(b.x as f64, b.y as f64);
            ctx.line_to(b.rally_x as f64, b.rally_y as f64);
            ctx.stroke();

            ctx.begin_path();
            let _ = ctx.arc(
                b.rally_x as f64,
                b.rally_y as f64,
                6.0,
                0.0,
                std::f64::consts::PI * 2.0,
            );
            ctx.stroke();
        }
    }
}

fn draw_blueprint_brackets(
    ctx: &CanvasRenderingContext2d,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    color: &str,
) {
    ctx.set_stroke_style_str(color);
    ctx.set_line_width(2.0);
    ctx.begin_path();
    let d = 8.0;
    ctx.move_to(x, y + d);
    ctx.line_to(x, y);
    ctx.line_to(x + d, y);
    ctx.move_to(x + w - d, y);
    ctx.line_to(x + w, y);
    ctx.line_to(x + w, y + d);
    ctx.move_to(x, y + h - d);
    ctx.line_to(x, y + h);
    ctx.line_to(x + d, y + h);
    ctx.move_to(x + w - d, y + h);
    ctx.line_to(x + w, y + h);
    ctx.line_to(x + w, y + h - d);
    ctx.stroke();
}

fn render_units(engine: &GameEngine) {
    let ctx = &engine.context;
    for u in &engine.units {
        if !engine.camera.is_point_visible(u.x, u.y, 40.0) {
            continue;
        }

        let is_visible = u.team == Team::Player || engine.fog.is_visible(u.x, u.y);
        if !is_visible {
            continue;
        }

        let is_selected = engine.selected_unit_ids.contains(&u.id);
        let color = if u.team == Team::Player {
            "#00e5ff"
        } else {
            "#f43f5e"
        };

        // 1. Vòng tròn chọn dưới chân
        if is_selected {
            ctx.set_stroke_style_str("#ffffff");
            ctx.set_line_width(1.5);
            ctx.begin_path();
            let _ = ctx.ellipse(
                u.x as f64,
                (u.y + 4.0) as f64,
                14.0,
                7.0,
                0.0,
                0.0,
                std::f64::consts::PI * 2.0,
            );
            ctx.stroke();
        }

        // 2. Tia hàn phát sáng khi thợ mỏ đang đào khoáng sản (Mining Sparks)
        if let UnitOrder::Harvest {
            target_mineral_id,
            phase: HarvestPhase::Mining(t),
        } = u.order
        {
            if let Some(m) = engine.minerals.iter().find(|m| m.id == target_mineral_id) {
                let flicker = (t * 30.0).sin() > 0.0;
                if flicker {
                    ctx.set_stroke_style_str("#fef08a");
                    ctx.set_line_width(1.5);
                    ctx.begin_path();
                    ctx.move_to(u.x as f64, (u.y - 8.0) as f64);
                    ctx.line_to(m.x as f64, m.y as f64);
                    ctx.stroke();
                }
            }
        }

        // 3. Vẽ người que dạng thủ tục (Procedural Stickman)
        draw_procedural_stickman(ctx, u, color);

        // 4. Thanh máu nhỏ trên đầu
        let hp_pct = (u.hp / u.max_hp).clamp(0.0, 1.0) as f64;
        let bar_w = 24.0;
        let bar_h = 3.5;
        let bar_x = (u.x - 12.0) as f64;
        let bar_y = (u.y - 32.0) as f64;

        ctx.set_fill_style_str("#1e293b");
        ctx.fill_rect(bar_x, bar_y, bar_w, bar_h);
        ctx.set_fill_style_str(if hp_pct > 0.4 { "#22c55e" } else { "#ef4444" });
        ctx.fill_rect(bar_x, bar_y, bar_w * hp_pct, bar_h);
    }
}

fn draw_procedural_stickman(ctx: &CanvasRenderingContext2d, u: &Unit, color: &str) {
    ctx.save();
    ctx.set_stroke_style_str(color);
    ctx.set_fill_style_str(color);
    ctx.set_line_width(2.0);

    let flip = if u.facing_right { 1.0 } else { -1.0 };
    let leg_swing = (u.walk_cycle).sin() * 8.0;

    let x = u.x as f64;
    let y = u.y as f64;

    // Đầu
    ctx.begin_path();
    let _ = ctx.arc(x, y - 20.0, 5.0, 0.0, std::f64::consts::PI * 2.0);
    ctx.stroke();

    // Thân
    ctx.begin_path();
    ctx.move_to(x, y - 15.0);
    ctx.line_to(x, y - 4.0);
    ctx.stroke();

    // Hai chân
    ctx.begin_path();
    ctx.move_to(x, y - 4.0);
    ctx.line_to(x - (leg_swing * flip) as f64, y + 4.0);
    ctx.move_to(x, y - 4.0);
    ctx.line_to(x + (leg_swing * flip) as f64, y + 4.0);
    ctx.stroke();

    // Tay & Vũ khí
    match u.unit_type {
        UnitType::Engineer => {
            ctx.begin_path();
            ctx.move_to(x, y - 11.0);
            ctx.line_to(x + 10.0 * flip as f64, y - 6.0);
            ctx.stroke();
            ctx.set_fill_style_str("#fbbf24");
            ctx.fill_rect(x + 9.0 * flip as f64, y - 9.0, 4.0, 6.0);
        }
        UnitType::ShockTrooper => {
            ctx.set_stroke_style_str("#38bdf8");
            ctx.set_line_width(2.5);
            ctx.begin_path();
            ctx.move_to(x, y - 10.0);
            ctx.line_to(x + 16.0 * flip as f64, y - 14.0);
            ctx.stroke();
        }
        UnitType::Rifleman => {
            ctx.set_stroke_style_str("#94a3b8");
            ctx.set_line_width(2.0);
            ctx.begin_path();
            ctx.move_to(x - 4.0 * flip as f64, y - 8.0);
            ctx.line_to(x + 14.0 * flip as f64, y - 10.0);
            ctx.stroke();
        }
        UnitType::Vanguard => {
            ctx.set_stroke_style_str("#60a5fa");
            ctx.set_line_width(3.0);
            ctx.begin_path();
            ctx.move_to(x + 10.0 * flip as f64, y - 22.0);
            ctx.line_to(x + 10.0 * flip as f64, y + 2.0);
            ctx.stroke();
        }
        UnitType::LaserMarksman => {
            ctx.set_stroke_style_str("#c084fc");
            ctx.set_line_width(3.0);
            ctx.begin_path();
            ctx.move_to(x - 5.0 * flip as f64, y - 9.0);
            ctx.line_to(x + 20.0 * flip as f64, y - 11.0);
            ctx.stroke();
        }
        UnitType::DarkRocketeer => {
            // Ống phóng tên lửa vác vai 2 nòng
            ctx.set_stroke_style_str("#fb923c");
            ctx.set_line_width(3.5);
            ctx.begin_path();
            ctx.move_to(x - 8.0 * flip as f64, y - 16.0);
            ctx.line_to(x + 16.0 * flip as f64, y - 12.0);
            ctx.stroke();
            // Đầu tên lửa
            ctx.set_fill_style_str("#ef4444");
            ctx.fill_rect(x + 15.0 * flip as f64, y - 14.0, 4.0, 4.0);
        }
        UnitType::TaoistSwordmaster => {
            // Phi kiếm lơ lửng phát sáng sau lưng
            ctx.set_stroke_style_str("#38bdf8");
            ctx.set_line_width(2.0);
            ctx.begin_path();
            ctx.move_to(x - 8.0 * flip as f64, y - 28.0);
            ctx.line_to(x - 2.0 * flip as f64, y - 12.0);
            ctx.stroke();
            // Tay kết ấn
            ctx.begin_path();
            ctx.move_to(x, y - 10.0);
            ctx.line_to(x + 12.0 * flip as f64, y - 10.0);
            ctx.stroke();
        }
        UnitType::VoidWarlock => {
            // Quyền trượng hư không có cầu ma thuật tím
            ctx.set_stroke_style_str("#a855f7");
            ctx.set_line_width(2.5);
            ctx.begin_path();
            ctx.move_to(x + 10.0 * flip as f64, y + 2.0);
            ctx.line_to(x + 10.0 * flip as f64, y - 24.0);
            ctx.stroke();
            // Cầu bóng tối xoay
            ctx.set_fill_style_str("#c084fc");
            ctx.begin_path();
            let _ = ctx.arc(x + 10.0 * flip as f64, y - 26.0, 4.0, 0.0, std::f64::consts::PI * 2.0);
            ctx.fill();
        }
        UnitType::BehemothTitan => {
            // Cự thú Titan: chùy gai khổng lồ
            ctx.set_stroke_style_str("#f59e0b");
            ctx.set_line_width(4.0);
            ctx.begin_path();
            ctx.move_to(x, y - 10.0);
            ctx.line_to(x + 16.0 * flip as f64, y - 18.0);
            ctx.stroke();
            // Đầu chùy gai to
            ctx.set_fill_style_str("#d97706");
            ctx.fill_rect(x + 14.0 * flip as f64, y - 22.0, 8.0, 8.0);
        }
        UnitType::AcidBaneling => {
            // Bọ nổ axit: túi dịch axit phát sáng trên lưng
            ctx.set_fill_style_str("#84cc16");
            ctx.begin_path();
            let _ = ctx.arc(x - 4.0 * flip as f64, y - 12.0, 7.0, 0.0, std::f64::consts::PI * 2.0);
            ctx.fill();
            ctx.set_stroke_style_str("#a3e635");
            ctx.set_line_width(1.5);
            ctx.stroke();
        }
    }

    ctx.restore();
}

fn render_projectiles(engine: &GameEngine) {
    let ctx = &engine.context;
    for p in &engine.projectiles {
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
            let _ = ctx.arc(p.x as f64, p.y as f64, radius, 0.0, std::f64::consts::PI * 2.0);
            ctx.fill();

            if p.aoe_radius > 0.0 {
                ctx.set_stroke_style_str(color);
                ctx.set_line_width(1.0);
                ctx.stroke();
            }
        }
    }
}

fn render_build_ghost(engine: &GameEngine, b_type: BuildingType) {
    let ctx = &engine.context;
    let (wx, wy) = engine.screen_to_world(engine.mouse_screen_x, engine.mouse_screen_y);
    let (w, h) = match b_type {
        BuildingType::CommandCenter => (90.0, 90.0),
        BuildingType::SupplyDepot => (60.0, 60.0),
        BuildingType::PowerReactor => (60.0, 60.0),
        BuildingType::Barracks => (80.0, 70.0),
        BuildingType::TurretTower => (50.0, 50.0),
        BuildingType::EnergyConverter => (65.0, 65.0),
    };

    let bx = (wx - w * 0.5) as f64;
    let by = (wy - h * 0.5) as f64;

    ctx.set_fill_style_str("rgba(0, 229, 255, 0.25)");
    ctx.fill_rect(bx, by, w as f64, h as f64);

    ctx.set_stroke_style_str("#00e5ff");
    ctx.set_line_width(2.0);
    ctx.stroke_rect(bx, by, w as f64, h as f64);

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
        let _ = ctx.arc(wx as f64, wy as f64, range as f64, 0.0, std::f64::consts::PI * 2.0);
        ctx.stroke();
    }
}

fn render_fog_of_war(engine: &GameEngine) {
    let ctx = &engine.context;
    use crate::fog::{CELL_H, CELL_W, FOG_COLS, FOG_ROWS};

    let (min_x, min_y, max_x, max_y) = engine.camera.viewport_bounds(CELL_W.max(CELL_H));
    let min_c = ((min_x / CELL_W).max(0.0) as usize).min(FOG_COLS - 1);
    let max_c = (((max_x / CELL_W).max(0.0) as usize) + 1).min(FOG_COLS);
    let min_r = ((min_y / CELL_H).max(0.0) as usize).min(FOG_ROWS - 1);
    let max_r = (((max_y / CELL_H).max(0.0) as usize) + 1).min(FOG_ROWS);

    // Gom các ô liên tiếp cùng trạng thái để vẽ 1 lần (Horizontal Run-Length Merging)
    for r in min_r..max_r {
        let ry = (r as f32 * CELL_H) as f64;
        let rh = (CELL_H + 0.5) as f64;

        let mut current_vis = Visibility::Visible;
        let mut start_c = min_c;
        let mut count = 0;

        for c in min_c..max_c {
            let vis = engine.fog.grid[r * FOG_COLS + c];
            if vis == current_vis {
                count += 1;
            } else {
                if current_vis != Visibility::Visible && count > 0 {
                    let rx = (start_c as f32 * CELL_W) as f64;
                    let rw = (count as f32 * CELL_W + 0.5) as f64;
                    if current_vis == Visibility::Hidden {
                        ctx.set_fill_style_str("rgba(2, 6, 13, 0.96)");
                    } else {
                        ctx.set_fill_style_str("rgba(3, 13, 26, 0.6)");
                    }
                    ctx.fill_rect(rx, ry, rw, rh);
                }
                current_vis = vis;
                start_c = c;
                count = 1;
            }
        }

        if current_vis != Visibility::Visible && count > 0 {
            let rx = (start_c as f32 * CELL_W) as f64;
            let rw = (count as f32 * CELL_W + 0.5) as f64;
            if current_vis == Visibility::Hidden {
                ctx.set_fill_style_str("rgba(2, 6, 13, 0.96)");
            } else {
                ctx.set_fill_style_str("rgba(3, 13, 26, 0.6)");
            }
            ctx.fill_rect(rx, ry, rw, rh);
        }
    }
}

fn render_floating_texts(engine: &GameEngine) {
    let ctx = &engine.context;
    ctx.set_font("bold 13px monospace");
    for ft in &engine.floating_texts {
        let (sx, sy) = engine.world_to_screen(ft.x, ft.y);
        let alpha = (ft.lifetime / ft.max_lifetime).clamp(0.0, 1.0);
        ctx.set_fill_style_str(&format!("rgba(0, 229, 255, {})", alpha));
        let _ = ctx.fill_text(&ft.text, sx as f64 - 16.0, sy as f64);
    }
}

fn render_attack_move_cursor(engine: &GameEngine) {
    let ctx = &engine.context;
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

fn render_in_game_hud(engine: &GameEngine) {
    let ctx = &engine.context;
    let vw = engine.camera.viewport_w as f64;
    let vh = engine.camera.viewport_h as f64;

    // 1. THANH TRẠNG THÁI TRÊN CÙNG (TOP BAR HUD)
    ctx.set_fill_style_str("rgba(3, 13, 26, 0.94)");
    ctx.fill_rect(0.0, 0.0, vw, 48.0);
    ctx.set_stroke_style_str("#0284c7");
    ctx.set_line_width(1.5);
    ctx.stroke_rect(0.0, 0.0, vw, 48.0);

    ctx.set_font("bold 14px monospace");

    // Khoáng sản có kèm tốc độ (+X/s)
    ctx.set_fill_style_str("#00e5ff");
    let _ = ctx.fill_text(
        &format!(
            "💎 MINERALS: {:.0} (+{:.0}/s)",
            engine.player_eco.minerals, engine.player_eco.minerals_per_sec
        ),
        20.0,
        29.0,
    );

    // Năng lượng có kèm tốc độ (+X/s)
    ctx.set_fill_style_str("#c084fc");
    let _ = ctx.fill_text(
        &format!(
            "⚡ ENERGY: {:.0} (+{:.0}/s)",
            engine.player_eco.energy, engine.player_eco.energy_per_sec
        ),
        245.0,
        29.0,
    );

    // Dân số
    ctx.set_fill_style_str("#38bdf8");
    let _ = ctx.fill_text(
        &format!(
            "👥 SUPPLY: {}/{}",
            engine.player_eco.supply, engine.player_eco.max_supply
        ),
        450.0,
        29.0,
    );

    // Tiến trình nghiên cứu hiện tại
    if let Some(ref tech_id) = engine.player_tech.current_research {
        ctx.set_fill_style_str("#facc15");
        let _ = ctx.fill_text(
            &format!(
                "🔬 RESEARCHING: {} ({:.0}%)",
                tech_id,
                engine.player_tech.research_progress * 100.0
            ),
            610.0,
            29.0,
        );
    }

    // Nút mở Cây Công Nghệ [T]
    let tech_btn_x = vw - 190.0;
    ctx.set_fill_style_str("#0369a1");
    ctx.fill_rect(tech_btn_x, 10.0, 85.0, 28.0);
    ctx.set_stroke_style_str("#38bdf8");
    ctx.set_line_width(1.5);
    ctx.stroke_rect(tech_btn_x, 10.0, 85.0, 28.0);
    ctx.set_fill_style_str("#ffffff");
    ctx.set_font("bold 12px monospace");
    let _ = ctx.fill_text("TECH [T]", tech_btn_x + 12.0, 28.0);

    // Nút Cài đặt [ESC]
    let opt_btn_x = vw - 90.0;
    ctx.set_fill_style_str("#1e293b");
    ctx.fill_rect(opt_btn_x, 10.0, 80.0, 28.0);
    ctx.set_stroke_style_str("#64748b");
    ctx.stroke_rect(opt_btn_x, 10.0, 80.0, 28.0);
    ctx.set_fill_style_str("#cbd5e1");
    let _ = ctx.fill_text("MENU [ESC]", opt_btn_x + 8.0, 28.0);

    // 2. MINIMAP GÓC DƯỚI TRÁI (Kèm hiển thị mỏ khoáng sản)
    let mm_x = 10.0;
    let mm_y = vh - 170.0;
    let mm_s = 160.0;

    ctx.set_fill_style_str("rgba(3, 13, 26, 0.94)");
    ctx.fill_rect(mm_x, mm_y, mm_s, mm_s);
    ctx.set_stroke_style_str("#00e5ff");
    ctx.set_line_width(2.0);
    ctx.stroke_rect(mm_x, mm_y, mm_s, mm_s);

    // Vẽ mỏ quặng trên Minimap
    for m in &engine.minerals {
        let mx = mm_x + (m.x / MAP_WIDTH) as f64 * mm_s;
        let my = mm_y + (m.y / MAP_HEIGHT) as f64 * mm_s;
        ctx.set_fill_style_str("#38bdf8");
        ctx.fill_rect(mx - 1.5, my - 1.5, 3.0, 3.0);
    }

    // Vẽ công trình trên Minimap
    for b in &engine.buildings {
        let mx = mm_x + (b.x / MAP_WIDTH) as f64 * mm_s;
        let my = mm_y + (b.y / MAP_HEIGHT) as f64 * mm_s;
        ctx.set_fill_style_str(if b.team == Team::Player {
            "#00e5ff"
        } else {
            "#f43f5e"
        });
        ctx.fill_rect(mx - 2.5, my - 2.5, 5.0, 5.0);
    }

    // Vẽ lính trên Minimap
    for u in &engine.units {
        if u.team == Team::Player || engine.fog.is_visible(u.x, u.y) {
            let mx = mm_x + (u.x / MAP_WIDTH) as f64 * mm_s;
            let my = mm_y + (u.y / MAP_HEIGHT) as f64 * mm_s;
            ctx.set_fill_style_str(if u.team == Team::Player {
                "#38bdf8"
            } else {
                "#ef4444"
            });
            ctx.fill_rect(mx - 1.5, my - 1.5, 3.0, 3.0);
        }
    }

    // Khung nhìn Camera trên Minimap
    let cam_rect_x = mm_x
        + ((engine.camera.x - engine.camera.viewport_w * 0.5 / engine.camera.zoom) / MAP_WIDTH)
            .clamp(0.0, 1.0) as f64
            * mm_s;
    let cam_rect_y = mm_y
        + ((engine.camera.y - engine.camera.viewport_h * 0.5 / engine.camera.zoom) / MAP_HEIGHT)
            .clamp(0.0, 1.0) as f64
            * mm_s;
    let cam_rect_w = ((engine.camera.viewport_w / engine.camera.zoom) / MAP_WIDTH) as f64 * mm_s;
    let cam_rect_h = ((engine.camera.viewport_h / engine.camera.zoom) / MAP_HEIGHT) as f64 * mm_s;
    ctx.set_stroke_style_str("#ffffff");
    ctx.set_line_width(1.2);
    ctx.stroke_rect(cam_rect_x, cam_rect_y, cam_rect_w, cam_rect_h);

    // 3. SELECTION INFO PANEL (TRUNG TÂM ĐÁY MÀN HÌNH)
    render_selection_info_panel(engine, (vw - 360.0) * 0.5, vh - 150.0, 360.0, 140.0);

    // 4. BẢNG NÚT BẤM LỆNH GÓC DƯỚI PHẢI (COMMAND CARD)
    render_command_card(engine, vw - 225.0, vh - 150.0);
}

fn render_selection_info_panel(engine: &GameEngine, x: f64, y: f64, w: f64, h: f64) {
    let ctx = &engine.context;
    ctx.set_fill_style_str("rgba(3, 13, 26, 0.94)");
    ctx.fill_rect(x, y, w, h);
    ctx.set_stroke_style_str("#0284c7");
    ctx.set_line_width(1.5);
    ctx.stroke_rect(x, y, w, h);

    // 1. Nếu đang chọn 1 công trình
    if let Some(bid) = engine.selected_building_id {
        if let Some(b) = engine.buildings.iter().find(|b| b.id == bid) {
            let color = if b.team == Team::Player {
                "#00e5ff"
            } else {
                "#f43f5e"
            };
            ctx.set_font("bold 13px monospace");
            ctx.set_fill_style_str(color);
            let title = match b.building_type {
                BuildingType::CommandCenter => "HQ COMMAND CENTER",
                BuildingType::SupplyDepot => "SUPPLY DEPOT",
                BuildingType::PowerReactor => "POWER REACTOR",
                BuildingType::Barracks => "INFANTRY BARRACKS",
                BuildingType::TurretTower => "DEFENSE TURRET",
                BuildingType::EnergyConverter => "ENERGY CONVERTER",
            };
            let _ = ctx.fill_text(title, x + 15.0, y + 22.0);

            // Thanh máu chi tiết
            let hp_pct = (b.hp / b.max_hp).clamp(0.0, 1.0) as f64;
            ctx.set_fill_style_str("#1e293b");
            ctx.fill_rect(x + 15.0, y + 30.0, w - 30.0, 10.0);
            ctx.set_fill_style_str(if hp_pct > 0.4 { "#22c55e" } else { "#ef4444" });
            ctx.fill_rect(x + 15.0, y + 30.0, (w - 30.0) * hp_pct, 10.0);

            ctx.set_font("bold 10px monospace");
            ctx.set_fill_style_str("#ffffff");
            let _ = ctx.fill_text(
                &format!("{:.0} / {:.0} HP", b.hp, b.max_hp),
                x + 20.0,
                y + 39.0,
            );

            // Chỉ số kháng công trình
            ctx.set_font("10px monospace");
            ctx.set_fill_style_str("#38bdf8");
            let _ = ctx.fill_text(
                &format!(
                    "🛡️ RES: PHY {:.0}% | ENG {:.0}% | EXP {:.0}% | ARC {:.0}%",
                    b.resistances.physical * 100.0,
                    b.resistances.energy * 100.0,
                    b.resistances.explosive * 100.0,
                    b.resistances.arcane * 100.0
                ),
                x + 15.0,
                y + 58.0,
            );

            let b_cfg = engine.config.get_building(b.building_type);
            let b_range = b_cfg.map(|c| c.turret_range).unwrap_or(0.0);
            let b_damage = b_cfg.map(|c| c.turret_damage).unwrap_or(0.0);
            let is_armed = b_range > 0.0 && b_damage > 0.0;

            // Trạng thái công trình đặc thù cho Trạm Chuyển Hóa
            if b.building_type == BuildingType::EnergyConverter {
                ctx.set_font("bold 11px monospace");
                let cur_energy = if b.team == Team::Player {
                    engine.player_eco.energy
                } else {
                    engine.enemy_eco.energy
                };

                if cur_energy >= 1000.0 {
                    ctx.set_fill_style_str("#fbbf24");
                    let _ = ctx.fill_text("STATUS: CONVERTING (30⚡ ➔ 25💎/s)", x + 15.0, y + 85.0);
                    ctx.set_font("10px monospace");
                    ctx.set_fill_style_str("#38bdf8");
                    let _ = ctx.fill_text("Tự động chuyển Năng lượng dư thừa (>1000⚡) thành Vàng.", x + 15.0, y + 105.0);
                } else {
                    ctx.set_fill_style_str("#f97316");
                    let _ = ctx.fill_text("STATUS: STANDBY (Năng lượng < 1000⚡)", x + 15.0, y + 85.0);
                    ctx.set_font("10px monospace");
                    ctx.set_fill_style_str("#94a3b8");
                    let _ = ctx.fill_text(
                        &format!("Cần thêm {:.0}⚡ để tự động kích hoạt.", (1000.0 - cur_energy).max(0.0)),
                        x + 15.0,
                        y + 105.0,
                    );
                }
            } else if !b.training_queue.is_empty() {
                ctx.set_font("11px monospace");
                ctx.set_fill_style_str("#facc15");
                let cur_unit = b.training_queue[0].button_label();
                let req_time = engine.config.get_unit_training_time(b.training_queue[0]);
                let prog = (b.training_timer / req_time).clamp(0.0, 1.0);
                let _ = ctx.fill_text(
                    &format!("TRAINING: {} ({:.0}%)", cur_unit, prog * 100.0),
                    x + 15.0,
                    y + 78.0,
                );

                ctx.set_fill_style_str("#334155");
                ctx.fill_rect(x + 15.0, y + 84.0, w - 30.0, 6.0);
                ctx.set_fill_style_str("#facc15");
                ctx.fill_rect(x + 15.0, y + 84.0, (w - 30.0) * (prog as f64), 6.0);

                if is_armed {
                    ctx.set_font("10px monospace");
                    ctx.set_fill_style_str("#38bdf8");
                    let _ = ctx.fill_text(
                        &format!("⚔️ ATK: {:.0} [{}] | RNG: {:.0}", b_damage, b.damage_type.display_name(), b_range),
                        x + 15.0,
                        y + 106.0,
                    );
                } else if b.training_queue.len() > 1 {
                    ctx.set_font("10px monospace");
                    ctx.set_fill_style_str("#94a3b8");
                    let mut q_str = String::from("Queue: ");
                    for (i, u) in b.training_queue.iter().skip(1).enumerate() {
                        if i > 0 {
                            q_str.push_str(" > ");
                        }
                        q_str.push_str(u.button_label());
                    }
                    let _ = ctx.fill_text(&q_str, x + 15.0, y + 106.0);
                }
            } else if is_armed {
                let b_cd = b_cfg.map(|c| c.turret_cooldown).unwrap_or(1.0).max(0.1);
                let has_dark_rockets = b.team == Team::Player && engine.player_tech.dark_matter_rockets;
                let has_overload = b.team == Team::Player && engine.player_tech.overload_strike;

                let eff_damage = if has_dark_rockets { b_damage * 1.4 } else { b_damage };
                let eff_cd = if has_overload { b_cd * 0.6 } else { b_cd };

                ctx.set_font("11px monospace");
                ctx.set_fill_style_str("#e2e8f0");
                let _ = ctx.fill_text(
                    &format!(
                        "⚔️ ATK: {:.0} [{}]   🎯 RNG: {:.0}   ⚡ CD: {:.1}s",
                        eff_damage,
                        b.damage_type.display_name(),
                        b_range,
                        eff_cd
                    ),
                    x + 15.0,
                    y + 78.0,
                );

                ctx.set_font("10px monospace");
                if has_dark_rockets || has_overload {
                    ctx.set_fill_style_str("#f59e0b");
                    let mut upgrades_str = String::from("UPGRADES: ");
                    if has_dark_rockets {
                        upgrades_str.push_str("[DARK ROCKETS: AOE] ");
                    }
                    if has_overload {
                        upgrades_str.push_str("[OVERLOAD: +66% SPD]");
                    }
                    let _ = ctx.fill_text(&upgrades_str, x + 15.0, y + 96.0);
                } else {
                    ctx.set_fill_style_str("#94a3b8");
                    let _ = ctx.fill_text("STATUS: DEFENSIVE // ARMED", x + 15.0, y + 96.0);
                }
                ctx.set_font("10px monospace");
                ctx.set_fill_style_str("#64748b");
                let _ = ctx.fill_text("Targeting: Automatic hostile engagement", x + 15.0, y + 112.0);
            } else {
                ctx.set_font("11px monospace");
                ctx.set_fill_style_str("#94a3b8");
                let _ = ctx.fill_text("STATUS: OPERATIONAL // IDLE", x + 15.0, y + 85.0);
                let _ = ctx.fill_text("Rally Point: Set via Right-Click", x + 15.0, y + 105.0);
            }
            return;
        }
    }

    // 2. Nếu đang chọn 1 lính
    if engine.selected_unit_ids.len() == 1 {
        let uid = engine.selected_unit_ids[0];
        if let Some(u) = engine.units.iter().find(|u| u.id == uid) {
            ctx.set_font("bold 13px monospace");
            ctx.set_fill_style_str(if u.team == Team::Player { "#00e5ff" } else { "#f43f5e" });
            let name = u.unit_type.display_name();
            let _ = ctx.fill_text(name, x + 15.0, y + 22.0);

            // Thanh máu
            let hp_pct = (u.hp / u.max_hp).clamp(0.0, 1.0) as f64;
            ctx.set_fill_style_str("#1e293b");
            ctx.fill_rect(x + 15.0, y + 30.0, w - 30.0, 10.0);
            ctx.set_fill_style_str(if hp_pct > 0.4 { "#22c55e" } else { "#ef4444" });
            ctx.fill_rect(x + 15.0, y + 30.0, (w - 30.0) * hp_pct, 10.0);

            ctx.set_font("bold 10px monospace");
            ctx.set_fill_style_str("#ffffff");
            let _ = ctx.fill_text(
                &format!("{:.0} / {:.0} HP", u.hp, u.max_hp),
                x + 20.0,
                y + 39.0,
            );

            // Chỉ số chiến đấu & Loại sát thương
            ctx.set_font("11px monospace");
            ctx.set_fill_style_str("#e2e8f0");
            let _ = ctx.fill_text(
                &format!(
                    "⚔️ ATK: {:.0} [{}]   🎯 RNG: {:.0}   🏃 SPD: {:.0}",
                    u.attack_damage,
                    u.damage_type.display_name(),
                    u.attack_range,
                    u.speed
                ),
                x + 15.0,
                y + 60.0,
            );

            // Chỉ số kháng
            ctx.set_font("10px monospace");
            ctx.set_fill_style_str("#38bdf8");
            let _ = ctx.fill_text(
                &format!(
                    "🛡️ RES: PHY {:.0}% | ENG {:.0}% | EXP {:.0}% | ARC {:.0}%",
                    u.resistances.physical * 100.0,
                    u.resistances.energy * 100.0,
                    u.resistances.explosive * 100.0,
                    u.resistances.arcane * 100.0
                ),
                x + 15.0,
                y + 80.0,
            );

            // Trạng thái hành động
            ctx.set_fill_style_str("#94a3b8");
            let status_text = match u.order {
                UnitOrder::Idle => "Status: Standing By [IDLE]",
                UnitOrder::MoveTo { .. } => "Status: Moving to coordinates",
                UnitOrder::AttackMove { .. } => "Status: Executing Attack-Move [A]",
                UnitOrder::Harvest { phase, .. } => match phase {
                    HarvestPhase::MovingToNode => "Status: Moving to Mineral Node",
                    HarvestPhase::Mining(_) => "Status: Mining Minerals...",
                    HarvestPhase::ReturningToBase => "Status: Returning cargo to HQ",
                },
                UnitOrder::Build { .. } => "Status: Constructing blueprint",
                UnitOrder::AttackTarget { is_building, .. } => {
                    if is_building {
                        "Status: Attacking Enemy Structure!"
                    } else {
                        "Status: Engaging Enemy Unit!"
                    }
                }
            };
            let _ = ctx.fill_text(status_text, x + 15.0, y + 102.0);
            return;
        }
    }

    // 3. Nếu đang chọn nhiều lính
    if engine.selected_unit_ids.len() > 1 {
        ctx.set_font("bold 13px monospace");
        ctx.set_fill_style_str("#00e5ff");
        let _ = ctx.fill_text(
            &format!("SELECTED SQUAD ({} UNITS)", engine.selected_unit_ids.len()),
            x + 15.0,
            y + 25.0,
        );

        let mut workers = 0;
        let mut combat_count = 0;
        let mut evo_count = 0;

        for uid in &engine.selected_unit_ids {
            if let Some(u) = engine.units.iter().find(|u| u.id == *uid) {
                match u.unit_type {
                    UnitType::Engineer => workers += 1,
                    UnitType::ShockTrooper | UnitType::Rifleman | UnitType::Vanguard | UnitType::LaserMarksman => {
                        combat_count += 1;
                    }
                    _ => {
                        evo_count += 1;
                    }
                }
            }
        }

        ctx.set_font("11px monospace");
        ctx.set_fill_style_str("#e2e8f0");
        let _ = ctx.fill_text(
            &format!("• Workers: {}  • Regular Forces: {}", workers, combat_count),
            x + 15.0,
            y + 55.0,
        );
        let _ = ctx.fill_text(
            &format!("• Evolution Elite Units: {}", evo_count),
            x + 15.0,
            y + 75.0,
        );
        ctx.set_fill_style_str("#64748b");
        let _ = ctx.fill_text(
            "Phím A: Attack-Move  | Chuột Phải: Điều động",
            x + 15.0,
            y + 105.0,
        );
        return;
    }

    // 4. Khi chưa chọn gì
    ctx.set_font("12px monospace");
    ctx.set_fill_style_str("#475569");
    let _ = ctx.fill_text("NO TARGET SELECTED", x + 15.0, y + 35.0);
    ctx.set_font("10px monospace");
    let _ = ctx.fill_text(
        "Click hoặc Kéo chuột để chọn đơn vị / công trình.",
        x + 15.0,
        y + 60.0,
    );
    let _ = ctx.fill_text(
        "Phím A: Attack-Move  |  Phím T: Cây Công Nghệ",
        x + 15.0,
        y + 80.0,
    );
}

fn draw_barracks_slot(
    ctx: &CanvasRenderingContext2d,
    engine: &GameEngine,
    b: &Building,
    u_type: UnitType,
    slot_x: f64,
    slot_y: f64,
    btn_idx: usize,
) -> (usize, bool, f32) {
    let count = b.training_queue.iter().filter(|&&t| t == u_type).count();
    let is_training = b.training_queue.first() == Some(&u_type);
    let mut req_time = engine.config.get_unit_training_time(u_type);
    if b.team == Team::Player && engine.player_tech.hive_proliferation {
        req_time *= 0.65;
    }
    let prog = if is_training {
        (b.training_timer / req_time).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let (mut min_cost, mut en_cost) = engine.config.get_unit_cost(u_type);
    if engine.player_tech.hive_proliferation {
        min_cost *= 0.65;
        en_cost *= 0.65;
    }
    let can_afford = engine.player_eco.minerals >= min_cost && engine.player_eco.energy >= en_cost;
    let cost_lbl = engine.config.get_unit_cost_label(u_type);

    draw_unit_purchase_button(
        ctx,
        slot_x,
        slot_y,
        62.0,
        40.0,
        u_type.button_label(),
        &cost_lbl,
        count,
        is_training,
        prog,
        engine.hovered_command_button == Some(btn_idx),
        can_afford,
    );

    (count, is_training, prog)
}

fn render_command_card(engine: &GameEngine, x: f64, y: f64) {
    let ctx = &engine.context;
    ctx.set_fill_style_str("rgba(3, 13, 26, 0.94)");
    ctx.fill_rect(x, y, 215.0, 140.0);
    ctx.set_stroke_style_str("#0284c7");
    ctx.set_line_width(1.5);
    ctx.stroke_rect(x, y, 215.0, 140.0);

    let has_worker = engine.selected_unit_ids.iter().any(|uid| {
        engine
            .units
            .iter()
            .any(|u| u.id == *uid && u.unit_type == UnitType::Engineer)
    });

    let mut tooltip: Option<(String, String)> = None;

    if has_worker {
        let supply_cost = engine.config.get_building_cost_label(BuildingType::SupplyDepot);
        let reactor_cost = engine.config.get_building_cost_label(BuildingType::PowerReactor);
        let barracks_cost = engine.config.get_building_cost_label(BuildingType::Barracks);
        let turret_cost = engine.config.get_building_cost_label(BuildingType::TurretTower);
        let cc_cost = engine.config.get_building_cost_label(BuildingType::CommandCenter);
        let conv_cost = engine.config.get_building_cost_label(BuildingType::EnergyConverter);

        draw_hud_button(ctx, x + 5.0, y + 6.0, 62.0, 40.0, "SUPPLY", &supply_cost);
        draw_hud_button(ctx, x + 72.0, y + 6.0, 62.0, 40.0, "REACTOR", &reactor_cost);
        draw_hud_button(ctx, x + 139.0, y + 6.0, 62.0, 40.0, "BARRACKS", &barracks_cost);
        draw_hud_button(ctx, x + 5.0, y + 50.0, 62.0, 40.0, "TURRET", &turret_cost);
        draw_hud_button(ctx, x + 72.0, y + 50.0, 62.0, 40.0, "HQ CC", &cc_cost);
        draw_hud_button(ctx, x + 139.0, y + 50.0, 62.0, 40.0, "CONVERT", &conv_cost);

        if let Some(hover_idx) = engine.hovered_command_button {
            tooltip = match hover_idx {
                0 => Some((
                    "SUPPLY DEPOT".into(),
                    format!(
                        "Giá: {}. Tăng +{} Giới hạn dân số (Supply).",
                        supply_cost,
                        engine
                            .config
                            .get_building(BuildingType::SupplyDepot)
                            .map(|b| b.supply_bonus)
                            .unwrap_or(8)
                    ),
                )),
                1 => Some((
                    "POWER REACTOR".into(),
                    format!(
                        "Giá: {}. Tăng tốc độ sản sinh Năng lượng +{:.0}/s.",
                        reactor_cost,
                        engine
                            .config
                            .get_building(BuildingType::PowerReactor)
                            .map(|b| b.energy_per_sec)
                            .unwrap_or(8.0)
                    ),
                )),
                2 => Some((
                    "BARRACKS".into(),
                    format!("Giá: {}. Trại huấn luyện lính chiến đấu.", barracks_cost),
                )),
                3 => Some((
                    "TURRET TOWER".into(),
                    format!("Giá: {}. Tháp phòng thủ tự động bắn tỉa.", turret_cost),
                )),
                4 => Some((
                    "HQ COMMAND CENTER".into(),
                    format!("Giá: {}. Trung tâm chỉ huy mở rộng.", cc_cost),
                )),
                5 => Some((
                    "ENERGY CONVERTER".into(),
                    format!(
                        "Giá: {}. Tự động chuyển Năng lượng thành Vàng khi Năng lượng >= {:.0}.",
                        conv_cost, engine.config.economy.converter_threshold
                    ),
                )),
                _ => None,
            };
        }
    } else if let Some(bid) = engine.selected_building_id {
        if let Some(b) = engine
            .buildings
            .iter()
            .find(|b| b.id == bid && b.team == Team::Player)
        {
            match b.building_type {
                BuildingType::CommandCenter => {
                    let u_type = UnitType::Engineer;
                    let count = b.training_queue.iter().filter(|&&t| t == u_type).count();
                    let is_training = b.training_queue.first() == Some(&u_type);
                    let req_time = engine.config.get_unit_training_time(u_type);
                    let prog = if is_training {
                        (b.training_timer / req_time).clamp(0.0, 1.0)
                    } else {
                        0.0
                    };
                    let (min_cost, en_cost) = engine.config.get_unit_cost(u_type);
                    let can_afford = engine.player_eco.minerals >= min_cost
                        && engine.player_eco.energy >= en_cost;
                    let cost_lbl = engine.config.get_unit_cost_label(u_type);

                    draw_unit_purchase_button(
                        ctx,
                        x + 5.0,
                        y + 6.0,
                        62.0,
                        40.0,
                        u_type.button_label(),
                        &cost_lbl,
                        count,
                        is_training,
                        prog,
                        engine.hovered_command_button == Some(0),
                        can_afford,
                    );

                    if engine.hovered_command_button == Some(0) {
                        let mut desc = format!(
                            "Giá: {}. Huấn luyện thợ mỏ / kỹ sư xây dựng.",
                            cost_lbl
                        );
                        if count > 0 {
                            if is_training {
                                desc.push_str(&format!(
                                    " [Đang tạo: {:.0}% | Chờ: {}]",
                                    prog * 100.0,
                                    count
                                ));
                            } else {
                                desc.push_str(&format!(" [Đang chờ: {}]", count));
                            }
                            desc.push_str(" (Click phải để hủy & hoàn tiền)");
                        }
                        tooltip = Some(("TRAIN WORKER".into(), desc));
                    }
                }
                BuildingType::Barracks => {
                    // 0: Striker
                    let s0 = draw_barracks_slot(ctx, engine, b, UnitType::ShockTrooper, x + 5.0, y + 6.0, 0);
                    // 1: Rifle
                    let s1 = draw_barracks_slot(ctx, engine, b, UnitType::Rifleman, x + 72.0, y + 6.0, 1);
                    // 2: Vanguard
                    let s2 = if engine.player_tech.energy_shielding {
                        Some(draw_barracks_slot(ctx, engine, b, UnitType::Vanguard, x + 139.0, y + 6.0, 2))
                    } else {
                        None
                    };
                    // 3: Laser
                    let s3 = if engine.player_tech.focused_laser {
                        Some(draw_barracks_slot(ctx, engine, b, UnitType::LaserMarksman, x + 5.0, y + 50.0, 3))
                    } else {
                        None
                    };
                    // 4: Dark Rocketeer
                    let s4 = if engine.player_tech.dark_matter_rockets {
                        Some(draw_barracks_slot(ctx, engine, b, UnitType::DarkRocketeer, x + 72.0, y + 50.0, 4))
                    } else {
                        None
                    };
                    // 5: Taoist Swordmaster
                    let s5 = if engine.player_tech.celestial_flying_swords {
                        Some(draw_barracks_slot(ctx, engine, b, UnitType::TaoistSwordmaster, x + 139.0, y + 50.0, 5))
                    } else {
                        None
                    };
                    // 6: Void Warlock
                    let s6 = if engine.player_tech.void_cataclysm {
                        Some(draw_barracks_slot(ctx, engine, b, UnitType::VoidWarlock, x + 5.0, y + 94.0, 6))
                    } else {
                        None
                    };
                    // 7: Behemoth Titan
                    let s7 = if engine.player_tech.behemoth_mutation {
                        Some(draw_barracks_slot(ctx, engine, b, UnitType::BehemothTitan, x + 72.0, y + 94.0, 7))
                    } else {
                        None
                    };
                    // 8: Acid Baneling
                    let s8 = if engine.player_tech.acid_banelings {
                        Some(draw_barracks_slot(ctx, engine, b, UnitType::AcidBaneling, x + 139.0, y + 94.0, 8))
                    } else {
                        None
                    };

                    if let Some(hover_idx) = engine.hovered_command_button {
                        let info = match hover_idx {
                            0 => Some(("SHOCK TROOPER", "Giá: 75M. Kiếm sĩ Plasma, chạy nhanh, chém mạnh.", s0)),
                            1 => Some(("RIFLEMAN", "Giá: 100M 20E. Xạ thủ súng trường tầm xa.", s1)),
                            2 => s2.map(|s| ("VANGUARD", "Giá: 150M 50E. Vệ binh khiên chắn năng lượng.", s)),
                            3 => s3.map(|s| ("LASER MARKSMAN", "Giá: 175M 100E. Pháo Laser siêu sát thương.", s)),
                            4 => s4.map(|s| ("DARK ROCKETEER", "Giá: 160M 80E. Bệ phóng Rocket nổ AoE tầm xa.", s)),
                            5 => s5.map(|s| ("SWORDMASTER", "Giá: 180M 90E. Tu tiên giả ngự phi kiếm chân khí Arcane.", s)),
                            6 => s6.map(|s| ("VOID WARLOCK", "Giá: 200M 120E. Vu sư hư không, cầu ma thuật nổ AoE.", s)),
                            7 => s7.map(|s| ("BEHEMOTH TITAN", "Giá: 250M 100E. Cự thú 650 HP, dậm chấn động diện rộng.", s)),
                            8 => s8.map(|s| ("ACID BANELING", "Giá: 60M 30E. Bọ cảm tử lao vào tự nổ axit AoE cực lớn.", s)),
                            _ => None,
                        };

                        if let Some((name, base_desc, (count, is_tr, prog))) = info {
                            let mut desc = base_desc.to_string();
                            if count > 0 {
                                if is_tr {
                                    desc.push_str(&format!(
                                        " [Đang tạo: {:.0}% | Chờ: {}]",
                                        prog * 100.0,
                                        count
                                    ));
                                } else {
                                    desc.push_str(&format!(" [Đang chờ: {}]", count));
                                }
                                desc.push_str(" (Click phải để hủy & hoàn tiền)");
                            }
                            tooltip = Some((name.to_string(), desc));
                        }
                    }
                }
                _ => {}
            }
        }
    }

    // Vẽ Tooltip thông tin nút nếu đang rê chuột
    if let Some((name, desc)) = tooltip {
        let tt_w = 320.0;
        let tt_h = 45.0;
        let tt_x = (x + 210.0 - tt_w).max(10.0);
        let tt_y = y - tt_h - 8.0;

        ctx.set_fill_style_str("rgba(3, 13, 26, 0.96)");
        ctx.fill_rect(tt_x, tt_y, tt_w, tt_h);
        ctx.set_stroke_style_str("#00e5ff");
        ctx.set_line_width(1.0);
        ctx.stroke_rect(tt_x, tt_y, tt_w, tt_h);

        ctx.set_font("bold 11px monospace");
        ctx.set_fill_style_str("#38bdf8");
        let _ = ctx.fill_text(&name, tt_x + 8.0, tt_y + 16.0);

        ctx.set_font("10px monospace");
        ctx.set_fill_style_str("#cbd5e1");
        let _ = ctx.fill_text(&desc, tt_x + 8.0, tt_y + 34.0);
    }
}

fn draw_hud_button(
    ctx: &CanvasRenderingContext2d,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    label: &str,
    cost: &str,
) {
    ctx.set_fill_style_str("#0f2942");
    ctx.fill_rect(x, y, w, h);
    ctx.set_stroke_style_str("#38bdf8");
    ctx.set_line_width(1.0);
    ctx.stroke_rect(x, y, w, h);

    ctx.set_font("bold 10px monospace");
    ctx.set_fill_style_str("#e2e8f0");
    let _ = ctx.fill_text(label, x + 4.0, y + 20.0);

    ctx.set_font("9px monospace");
    ctx.set_fill_style_str("#38bdf8");
    let _ = ctx.fill_text(cost, x + 4.0, y + 38.0);
}

#[allow(clippy::too_many_arguments)]
fn draw_unit_purchase_button(
    ctx: &CanvasRenderingContext2d,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    label: &str,
    cost: &str,
    queue_count: usize,
    is_actively_training: bool,
    training_progress: f32,
    is_hovered: bool,
    can_afford: bool,
) {
    // 1. Nền nút
    let bg_color = if is_actively_training {
        "#0b2d4d"
    } else if is_hovered {
        "#13385c"
    } else if !can_afford && queue_count == 0 {
        "#0a1c2e"
    } else {
        "#0f2942"
    };
    ctx.set_fill_style_str(bg_color);
    ctx.fill_rect(x, y, w, h);

    // 2. Viền nút
    let stroke_color = if is_actively_training {
        "#00e5ff"
    } else if is_hovered {
        "#38bdf8"
    } else {
        "#0284c7"
    };
    ctx.set_stroke_style_str(stroke_color);
    ctx.set_line_width(if is_actively_training || is_hovered {
        1.5
    } else {
        1.0
    });
    ctx.stroke_rect(x, y, w, h);

    // 3. Tên đơn vị
    ctx.set_font("bold 10px monospace");
    let label_color = if is_actively_training {
        "#00e5ff"
    } else if can_afford {
        "#e2e8f0"
    } else {
        "#94a3b8"
    };
    ctx.set_fill_style_str(label_color);
    let _ = ctx.fill_text(label, x + 4.0, y + 16.0);

    // 4. Giá tài nguyên
    ctx.set_font("9px monospace");
    let cost_color = if can_afford { "#38bdf8" } else { "#f87171" };
    ctx.set_fill_style_str(cost_color);
    let _ = ctx.fill_text(cost, x + 4.0, y + 27.0);

    // 5. Huy hiệu số lượng đang chờ (Queue Badge góc trên phải)
    if queue_count > 0 {
        let badge_w = 18.0;
        let badge_h = 13.0;
        let bx = x + w - badge_w - 2.0;
        let by = y + 2.0;

        let badge_bg = if is_actively_training {
            "#0284c7"
        } else {
            "#d97706"
        };
        ctx.set_fill_style_str(badge_bg);
        ctx.fill_rect(bx, by, badge_w, badge_h);
        ctx.set_stroke_style_str("#ffffff");
        ctx.set_line_width(0.8);
        ctx.stroke_rect(bx, by, badge_w, badge_h);

        ctx.set_font("bold 9px monospace");
        ctx.set_fill_style_str("#ffffff");
        let count_str = format!("x{}", queue_count);
        let _ = ctx.fill_text(&count_str, bx + 2.0, by + 10.0);
    }

    // 6. Thanh tiến độ tạo đơn vị ở đáy nút
    let pb_x = x + 3.0;
    let pb_y = y + 32.0;
    let pb_w = w - 6.0;
    let pb_h = 14.0;

    if is_actively_training {
        // Khung nền thanh tiến độ
        ctx.set_fill_style_str("#051322");
        ctx.fill_rect(pb_x, pb_y, pb_w, pb_h);
        ctx.set_stroke_style_str("#0284c7");
        ctx.set_line_width(0.8);
        ctx.stroke_rect(pb_x, pb_y, pb_w, pb_h);

        // Vạch tiến độ neon cyan
        let prog_norm = training_progress.clamp(0.0, 1.0) as f64;
        let fill_w = (pb_w * prog_norm).max(1.0);
        ctx.set_fill_style_str("#00e5ff");
        ctx.fill_rect(pb_x, pb_y, fill_w, pb_h);

        // Chữ phần trăm %
        ctx.set_font("bold 9px monospace");
        ctx.set_fill_style_str("#ffffff");
        let pct_str = format!("{:.0}%", prog_norm * 100.0);
        let _ = ctx.fill_text(&pct_str, pb_x + 13.0, pb_y + 10.5);
    } else if queue_count > 0 {
        // Đang chờ trong hàng đợi sau đơn vị khác
        ctx.set_fill_style_str("#051322");
        ctx.fill_rect(pb_x, pb_y, pb_w, pb_h);
        ctx.set_stroke_style_str("#334155");
        ctx.set_line_width(0.8);
        ctx.stroke_rect(pb_x, pb_y, pb_w, pb_h);

        ctx.set_font("bold 8px monospace");
        ctx.set_fill_style_str("#94a3b8");
        let _ = ctx.fill_text("QUEUED", pb_x + 9.0, pb_y + 10.5);
    }
}
