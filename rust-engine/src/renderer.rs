use crate::components::*;
use crate::fog::Visibility;
use crate::map::{MAP_HEIGHT, MAP_WIDTH};
use crate::GameEngine;
use wasm_bindgen::JsValue;
use web_sys::CanvasRenderingContext2d;

pub fn render(engine: &mut GameEngine) {
    let ctx = &engine.context;
    let vw = engine.camera.viewport_w;
    let vh = engine.camera.viewport_h;

    // 1. Xóa màn hình với màu nền Blueprint Dark
    ctx.set_fill_style(&JsValue::from_str("#030d1a"));
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
    ctx.set_stroke_style(&JsValue::from_str("#00e5ff"));
    ctx.set_line_width(3.0);
    ctx.stroke_rect(0.0, 0.0, MAP_WIDTH as f64, MAP_HEIGHT as f64);

    // 5. Vẽ Địa hình vật cản
    render_obstacles(engine);

    // 6. Vẽ Mỏ khoáng sản
    render_minerals(engine);

    // 7. Vẽ Công trình
    render_buildings(engine);

    // 8. Vẽ Lính người que (2.5D Oblique Procedural Stickmen)
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

    // Khung kéo chuột chọn quân (Box Select)
    if let Some((sx1, sy1, sx2, sy2)) = engine.selection_box {
        ctx.set_stroke_style(&JsValue::from_str("#00e5ff"));
        ctx.set_line_width(1.5);
        let rx = sx1.min(sx2);
        let ry = sy1.min(sy2);
        let rw = (sx2 - sx1).abs();
        let rh = (sy2 - sy1).abs();
        ctx.stroke_rect(rx as f64, ry as f64, rw as f64, rh as f64);
        ctx.set_fill_style(&JsValue::from_str("rgba(0, 229, 255, 0.1)"));
        ctx.fill_rect(rx as f64, ry as f64, rw as f64, rh as f64);
    }

    // Toàn bộ In-game HUD
    render_in_game_hud(engine);
}

fn render_blueprint_grid(engine: &GameEngine) {
    let ctx = &engine.context;
    ctx.set_stroke_style(&JsValue::from_str("#0a223e"));
    ctx.set_line_width(1.0);
    ctx.begin_path();

    let grid_size = 80.0;
    let mut x = 0.0;
    while x <= MAP_WIDTH {
        ctx.move_to(x as f64, 0.0);
        ctx.line_to(x as f64, MAP_HEIGHT as f64);
        x += grid_size;
    }
    let mut y = 0.0;
    while y <= MAP_HEIGHT {
        ctx.move_to(0.0, y as f64);
        ctx.line_to(MAP_WIDTH as f64, y as f64);
        y += grid_size;
    }
    ctx.stroke();
}

fn render_obstacles(engine: &GameEngine) {
    let ctx = &engine.context;
    for obs in &engine.obstacles {
        ctx.set_fill_style(&JsValue::from_str("#071a2e"));
        ctx.fill_rect(obs.x as f64, obs.y as f64, obs.width as f64, obs.height as f64);

        ctx.set_stroke_style(&JsValue::from_str("#1e40af"));
        ctx.set_line_width(1.5);
        ctx.stroke_rect(obs.x as f64, obs.y as f64, obs.width as f64, obs.height as f64);
    }
}

fn render_minerals(engine: &GameEngine) {
    let ctx = &engine.context;
    for m in &engine.minerals {
        if engine.fog.grid[0] != Visibility::Hidden || true { // Đơn giản hóa hiển thị mỏ đã biết
            // Vẽ tinh thể phát sáng Neon Cyan
            ctx.set_stroke_style(&JsValue::from_str("#38bdf8"));
            ctx.set_fill_style(&JsValue::from_str("rgba(56, 189, 248, 0.25)"));
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
            ctx.set_font("10px monospace");
            ctx.set_fill_style(&JsValue::from_str("#7dd3fc"));
            let _ = ctx.fill_text(&format!("{:.0}", m.amount), (m.x - 12.0) as f64, (m.y + 30.0) as f64);
        }
    }
}

fn render_buildings(engine: &GameEngine) {
    let ctx = &engine.context;
    for b in &engine.buildings {
        let is_visible = b.team == Team::Player || engine.fog.is_visible(b.x, b.y);
        if !is_visible {
            continue;
        }

        let is_selected = engine.selected_building_id == Some(b.id);
        let color_neon = if b.team == Team::Player { "#00e5ff" } else { "#f43f5e" };
        let color_fill = if b.team == Team::Player { "rgba(0, 229, 255, 0.12)" } else { "rgba(244, 63, 94, 0.12)" };

        let bx = (b.x - b.width * 0.5) as f64;
        let by = (b.y - b.height * 0.5) as f64;
        let bw = b.width as f64;
        let bh = b.height as f64;

        // Khung nhà Blueprint
        ctx.set_fill_style(&JsValue::from_str(color_fill));
        ctx.fill_rect(bx, by, bw, bh);

        ctx.set_stroke_style(&JsValue::from_str(if is_selected { "#ffffff" } else { color_neon }));
        ctx.set_line_width(if is_selected { 2.5 } else { 1.5 });
        ctx.stroke_rect(bx, by, bw, bh);

        // Ký hiệu góc kỹ thuật
        draw_blueprint_brackets(ctx, bx, by, bw, bh, color_neon);

        // Tên công trình & Tiến độ
        ctx.set_font("bold 11px monospace");
        ctx.set_fill_style(&JsValue::from_str(color_neon));
        let title = match b.building_type {
            BuildingType::CommandCenter => "HQ COMMAND",
            BuildingType::SupplyDepot => "SUPPLY DEPOT",
            BuildingType::PowerReactor => "REACTOR",
            BuildingType::Barracks => "BARRACKS",
            BuildingType::TurretTower => "TURRET",
        };
        let _ = ctx.fill_text(title, bx + 6.0, by + 18.0);

        // Thanh tiến độ xây dựng hoặc Thanh máu
        if b.build_progress < 1.0 {
            ctx.set_fill_style(&JsValue::from_str("#334155"));
            ctx.fill_rect(bx, by - 12.0, bw, 6.0);
            ctx.set_fill_style(&JsValue::from_str("#fbbf24"));
            ctx.fill_rect(bx, by - 12.0, bw * (b.build_progress as f64), 6.0);
        } else {
            // Thanh máu HP bar
            let hp_pct = (b.hp / b.max_hp).clamp(0.0, 1.0) as f64;
            ctx.set_fill_style(&JsValue::from_str("#1e293b"));
            ctx.fill_rect(bx, by - 10.0, bw, 5.0);
            ctx.set_fill_style(&JsValue::from_str(if hp_pct > 0.4 { "#22c55e" } else { "#ef4444" }));
            ctx.fill_rect(bx, by - 10.0, bw * hp_pct, 5.0);
        }

        // Vòng tròn điểm tập kết (Rally Point)
        if is_selected && b.team == Team::Player {
            ctx.set_stroke_style(&JsValue::from_str("#22c55e"));
            ctx.set_line_width(1.0);
            ctx.begin_path();
            ctx.move_to(b.x as f64, b.y as f64);
            ctx.line_to(b.rally_x as f64, b.rally_y as f64);
            ctx.stroke();

            ctx.begin_path();
            let _ = ctx.arc(b.rally_x as f64, b.rally_y as f64, 6.0, 0.0, std::f64::consts::PI * 2.0);
            ctx.stroke();
        }
    }
}

fn draw_blueprint_brackets(ctx: &CanvasRenderingContext2d, x: f64, y: f64, w: f64, h: f64, color: &str) {
    ctx.set_stroke_style(&JsValue::from_str(color));
    ctx.set_line_width(2.0);
    ctx.begin_path();
    let d = 8.0;
    // Góc trên trái
    ctx.move_to(x, y + d);
    ctx.line_to(x, y);
    ctx.line_to(x + d, y);
    // Góc trên phải
    ctx.move_to(x + w - d, y);
    ctx.line_to(x + w, y);
    ctx.line_to(x + w, y + d);
    // Góc dưới trái
    ctx.move_to(x, y + h - d);
    ctx.line_to(x, y + h);
    ctx.line_to(x + d, y + h);
    // Góc dưới phải
    ctx.move_to(x + w - d, y + h);
    ctx.line_to(x + w, y + h);
    ctx.line_to(x + w, y + h - d);
    ctx.stroke();
}

fn render_units(engine: &GameEngine) {
    let ctx = &engine.context;
    for u in &engine.units {
        let is_visible = u.team == Team::Player || engine.fog.is_visible(u.x, u.y);
        if !is_visible {
            continue;
        }

        let is_selected = engine.selected_unit_ids.contains(&u.id);
        let color = if u.team == Team::Player { "#00e5ff" } else { "#f43f5e" };

        // 1. Vòng tròn chọn dưới chân
        if is_selected {
            ctx.set_stroke_style(&JsValue::from_str("#ffffff"));
            ctx.set_line_width(1.5);
            ctx.begin_path();
            let _ = ctx.ellipse(u.x as f64, (u.y + 4.0) as f64, 14.0, 7.0, 0.0, 0.0, std::f64::consts::PI * 2.0);
            ctx.stroke();
        }

        // 2. Vẽ người que dạng thủ tục (Procedural Stickman)
        draw_procedural_stickman(ctx, u, color);

        // 3. Thanh máu nhỏ trên đầu
        let hp_pct = (u.hp / u.max_hp).clamp(0.0, 1.0) as f64;
        let bar_w = 24.0;
        let bar_h = 3.5;
        let bar_x = (u.x - 12.0) as f64;
        let bar_y = (u.y - 32.0) as f64;

        ctx.set_fill_style(&JsValue::from_str("#1e293b"));
        ctx.fill_rect(bar_x, bar_y, bar_w, bar_h);
        ctx.set_fill_style(&JsValue::from_str(if hp_pct > 0.4 { "#22c55e" } else { "#ef4444" }));
        ctx.fill_rect(bar_x, bar_y, bar_w * hp_pct, bar_h);
    }
}

fn draw_procedural_stickman(ctx: &CanvasRenderingContext2d, u: &Unit, color: &str) {
    ctx.save();
    ctx.set_stroke_style(&JsValue::from_str(color));
    ctx.set_fill_style(&JsValue::from_str(color));
    ctx.set_line_width(2.0);

    let flip = if u.facing_right { 1.0 } else { -1.0 };
    let leg_swing = (u.walk_cycle).sin() * 8.0;

    let x = u.x as f64;
    let y = u.y as f64;

    // Đầu (Hình tròn)
    ctx.begin_path();
    let _ = ctx.arc(x, y - 20.0, 5.0, 0.0, std::f64::consts::PI * 2.0);
    ctx.stroke();

    // Thân (Đoạn thẳng)
    ctx.begin_path();
    ctx.move_to(x, y - 15.0);
    ctx.line_to(x, y - 4.0);
    ctx.stroke();

    // Hai Chân đung đưa khi chạy
    ctx.begin_path();
    // Chân trái
    ctx.move_to(x, y - 4.0);
    ctx.line_to(x - (leg_swing * flip) as f64, y + 4.0);
    // Chân phải
    ctx.move_to(x, y - 4.0);
    ctx.line_to(x + (leg_swing * flip) as f64, y + 4.0);
    ctx.stroke();

    // Tay & Vũ khí đặc trưng cho từng loại lính
    match u.unit_type {
        UnitType::Engineer => {
            // Cờ lê / Máy hàn phát sáng
            ctx.begin_path();
            ctx.move_to(x, y - 11.0);
            ctx.line_to(x + 10.0 * flip as f64, y - 6.0);
            ctx.stroke();
            ctx.set_fill_style(&JsValue::from_str("#fbbf24"));
            ctx.fill_rect(x + 9.0 * flip as f64, y - 9.0, 4.0, 6.0);
        }
        UnitType::ShockTrooper => {
            // Kiếm Plasma Blade
            ctx.set_stroke_style(&JsValue::from_str("#38bdf8"));
            ctx.set_line_width(2.5);
            ctx.begin_path();
            ctx.move_to(x, y - 10.0);
            ctx.line_to(x + 16.0 * flip as f64, y - 14.0);
            ctx.stroke();
        }
        UnitType::Rifleman => {
            // Súng trường dài
            ctx.set_stroke_style(&JsValue::from_str("#94a3b8"));
            ctx.set_line_width(2.0);
            ctx.begin_path();
            ctx.move_to(x - 4.0 * flip as f64, y - 8.0);
            ctx.line_to(x + 14.0 * flip as f64, y - 10.0);
            ctx.stroke();
        }
        UnitType::Vanguard => {
            // Khiên chắn năng lượng lớn phía trước
            ctx.set_stroke_style(&JsValue::from_str("#60a5fa"));
            ctx.set_line_width(3.0);
            ctx.begin_path();
            ctx.move_to(x + 10.0 * flip as f64, y - 22.0);
            ctx.line_to(x + 10.0 * flip as f64, y + 2.0);
            ctx.stroke();
        }
        UnitType::LaserMarksman => {
            // Nòng súng pháo Laser viễn tưởng
            ctx.set_stroke_style(&JsValue::from_str("#c084fc"));
            ctx.set_line_width(3.0);
            ctx.begin_path();
            ctx.move_to(x - 5.0 * flip as f64, y - 9.0);
            ctx.line_to(x + 20.0 * flip as f64, y - 11.0);
            ctx.stroke();
        }
    }

    ctx.restore();
}

fn render_projectiles(engine: &GameEngine) {
    let ctx = &engine.context;
    for p in &engine.projectiles {
        if p.is_laser {
            // Tia Laser Neon rực sáng
            ctx.set_stroke_style(&JsValue::from_str(if p.team == Team::Player { "#38bdf8" } else { "#f43f5e" }));
            ctx.set_line_width(2.5);
            ctx.begin_path();
            ctx.move_to(p.x as f64, p.y as f64);
            ctx.line_to(p.target_x as f64, p.target_y as f64);
            ctx.stroke();
        } else {
            // Viên đạn phát sáng
            ctx.set_fill_style(&JsValue::from_str("#facc15"));
            ctx.begin_path();
            let _ = ctx.arc(p.x as f64, p.y as f64, 3.0, 0.0, std::f64::consts::PI * 2.0);
            ctx.fill();
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
    };

    let bx = (wx - w * 0.5) as f64;
    let by = (wy - h * 0.5) as f64;

    ctx.set_fill_style(&JsValue::from_str("rgba(0, 229, 255, 0.25)"));
    ctx.fill_rect(bx, by, w as f64, h as f64);

    ctx.set_stroke_style(&JsValue::from_str("#00e5ff"));
    ctx.set_line_width(2.0);
    ctx.stroke_rect(bx, by, w as f64, h as f64);
}

fn render_fog_of_war(engine: &GameEngine) {
    let ctx = &engine.context;
    use crate::fog::{CELL_H, CELL_W, FOG_COLS, FOG_ROWS};

    for r in 0..FOG_ROWS {
        for c in 0..FOG_COLS {
            let vis = engine.fog.grid[r * FOG_COLS + c];
            let rx = (c as f32 * CELL_W) as f64;
            let ry = (r as f32 * CELL_H) as f64;
            let rw = (CELL_W + 1.0) as f64;
            let rh = (CELL_H + 1.0) as f64;

            match vis {
                Visibility::Hidden => {
                    ctx.set_fill_style(&JsValue::from_str("rgba(2, 6, 13, 0.96)"));
                    ctx.fill_rect(rx, ry, rw, rh);
                }
                Visibility::Explored => {
                    ctx.set_fill_style(&JsValue::from_str("rgba(3, 13, 26, 0.6)"));
                    ctx.fill_rect(rx, ry, rw, rh);
                }
                Visibility::Visible => {
                    // Sáng hoàn toàn, không phủ gì
                }
            }
        }
    }
}

fn render_in_game_hud(engine: &GameEngine) {
    let ctx = &engine.context;
    let vw = engine.camera.viewport_w as f64;
    let vh = engine.camera.viewport_h as f64;

    // 1. THANH TRẠNG THÁI TRÊN CÙNG (TOP BAR HUD)
    ctx.set_fill_style(&JsValue::from_str("rgba(3, 13, 26, 0.9)"));
    ctx.fill_rect(0.0, 0.0, vw, 48.0);
    ctx.set_stroke_style(&JsValue::from_str("#0284c7"));
    ctx.set_line_width(1.5);
    ctx.stroke_rect(0.0, 0.0, vw, 48.0);

    // Hiển thị Tài nguyên
    ctx.set_font("bold 14px monospace");
    
    // Khoáng sản (Cyan)
    ctx.set_fill_style(&JsValue::from_str("#00e5ff"));
    let _ = ctx.fill_text(&format!("💎 MINERALS: {:.0}", engine.player_eco.minerals), 20.0, 29.0);

    // Năng lượng (Energy - Tím/Vàng)
    ctx.set_fill_style(&JsValue::from_str("#c084fc"));
    let _ = ctx.fill_text(&format!("⚡ ENERGY: {:.0}", engine.player_eco.energy), 200.0, 29.0);

    // Dân số (Supply)
    ctx.set_fill_style(&JsValue::from_str("#38bdf8"));
    let _ = ctx.fill_text(&format!("👥 SUPPLY: {}/{}", engine.player_eco.supply, engine.player_eco.max_supply), 360.0, 29.0);

    // Tiến trình nghiên cứu hiện tại (nếu có)
    if let Some(ref tech_id) = engine.player_tech.current_research {
        ctx.set_fill_style(&JsValue::from_str("#facc15"));
        let _ = ctx.fill_text(&format!("🔬 RESEARCHING: {} ({:.0}%)", tech_id, engine.player_tech.research_progress * 100.0), 520.0, 29.0);
    }

    // Nút mở Cây Công Nghệ [T]
    let tech_btn_x = vw - 190.0;
    ctx.set_fill_style(&JsValue::from_str("#0369a1"));
    ctx.fill_rect(tech_btn_x, 10.0, 85.0, 28.0);
    ctx.set_stroke_style(&JsValue::from_str("#38bdf8"));
    ctx.set_line_width(1.5);
    ctx.stroke_rect(tech_btn_x, 10.0, 85.0, 28.0);
    ctx.set_fill_style(&JsValue::from_str("#ffffff"));
    ctx.set_font("bold 12px monospace");
    let _ = ctx.fill_text("TECH [T]", tech_btn_x + 12.0, 28.0);

    // Nút Cài đặt [ESC]
    let opt_btn_x = vw - 90.0;
    ctx.set_fill_style(&JsValue::from_str("#1e293b"));
    ctx.fill_rect(opt_btn_x, 10.0, 80.0, 28.0);
    ctx.set_stroke_style(&JsValue::from_str("#64748b"));
    ctx.stroke_rect(opt_btn_x, 10.0, 80.0, 28.0);
    ctx.set_fill_style(&JsValue::from_str("#cbd5e1"));
    let _ = ctx.fill_text("MENU [ESC]", opt_btn_x + 8.0, 28.0);

    // 2. MINIMAP GÓC DƯỚI TRÁI
    let mm_x = 10.0;
    let mm_y = vh - 170.0;
    let mm_s = 160.0;

    ctx.set_fill_style(&JsValue::from_str("rgba(3, 13, 26, 0.92)"));
    ctx.fill_rect(mm_x, mm_y, mm_s, mm_s);
    ctx.set_stroke_style(&JsValue::from_str("#00e5ff"));
    ctx.set_line_width(2.0);
    ctx.stroke_rect(mm_x, mm_y, mm_s, mm_s);

    // Vẽ điểm trên Minimap
    for b in &engine.buildings {
        let mx = mm_x + (b.x / MAP_WIDTH) as f64 * mm_s;
        let my = mm_y + (b.y / MAP_HEIGHT) as f64 * mm_s;
        ctx.set_fill_style(&JsValue::from_str(if b.team == Team::Player { "#00e5ff" } else { "#f43f5e" }));
        ctx.fill_rect(mx - 2.5, my - 2.5, 5.0, 5.0);
    }

    for u in &engine.units {
        if u.team == Team::Player || engine.fog.is_visible(u.x, u.y) {
            let mx = mm_x + (u.x / MAP_WIDTH) as f64 * mm_s;
            let my = mm_y + (u.y / MAP_HEIGHT) as f64 * mm_s;
            ctx.set_fill_style(&JsValue::from_str(if u.team == Team::Player { "#38bdf8" } else { "#ef4444" }));
            ctx.fill_rect(mx - 1.5, my - 1.5, 3.0, 3.0);
        }
    }

    // Vẽ khung nhìn Camera trên Minimap
    let cam_rect_x = mm_x + ((engine.camera.x - engine.camera.viewport_w * 0.5 / engine.camera.zoom) / MAP_WIDTH).clamp(0.0, 1.0) as f64 * mm_s;
    let cam_rect_y = mm_y + ((engine.camera.y - engine.camera.viewport_h * 0.5 / engine.camera.zoom) / MAP_HEIGHT).clamp(0.0, 1.0) as f64 * mm_s;
    let cam_rect_w = ((engine.camera.viewport_w / engine.camera.zoom) / MAP_WIDTH) as f64 * mm_s;
    let cam_rect_h = ((engine.camera.viewport_h / engine.camera.zoom) / MAP_HEIGHT) as f64 * mm_s;
    ctx.set_stroke_style(&JsValue::from_str("#ffffff"));
    ctx.set_line_width(1.0);
    ctx.stroke_rect(cam_rect_x, cam_rect_y, cam_rect_w, cam_rect_h);

    // 3. BẢNG NÚT BẤM LỆNH GÓC DƯỚI PHẢI (COMMAND CARD)
    render_command_card(engine, vw - 220.0, vh - 140.0);
}

fn render_command_card(engine: &GameEngine, x: f64, y: f64) {
    let ctx = &engine.context;
    ctx.set_fill_style(&JsValue::from_str("rgba(3, 13, 26, 0.9)"));
    ctx.fill_rect(x, y, 210.0, 130.0);
    ctx.set_stroke_style(&JsValue::from_str("#0284c7"));
    ctx.set_line_width(1.5);
    ctx.stroke_rect(x, y, 210.0, 130.0);

    let has_worker = engine.selected_unit_ids.iter().any(|uid| {
        engine.units.iter().any(|u| u.id == *uid && u.unit_type == UnitType::Engineer)
    });

    if has_worker {
        // Nút xây dựng công trình
        draw_hud_button(ctx, x + 5.0, y + 10.0, 60.0, 50.0, "SUPPLY", "100M");
        draw_hud_button(ctx, x + 72.0, y + 10.0, 60.0, 50.0, "REACTOR", "120M");
        draw_hud_button(ctx, x + 140.0, y + 10.0, 60.0, 50.0, "BARRACKS", "150M");
        draw_hud_button(ctx, x + 5.0, y + 68.0, 60.0, 50.0, "TURRET", "125M 50E");
        draw_hud_button(ctx, x + 72.0, y + 68.0, 60.0, 50.0, "HQ CC", "400M");
    } else if let Some(bid) = engine.selected_building_id {
        if let Some(b) = engine.buildings.iter().find(|b| b.id == bid && b.team == Team::Player) {
            match b.building_type {
                BuildingType::CommandCenter => {
                    draw_hud_button(ctx, x + 5.0, y + 10.0, 60.0, 50.0, "WORKER", "50M");
                }
                BuildingType::Barracks => {
                    draw_hud_button(ctx, x + 5.0, y + 10.0, 60.0, 50.0, "STRIKER", "75M");
                    draw_hud_button(ctx, x + 72.0, y + 10.0, 60.0, 50.0, "RIFLE", "100M 20E");
                    if engine.player_tech.energy_shielding {
                        draw_hud_button(ctx, x + 140.0, y + 10.0, 60.0, 50.0, "SHIELD", "150M 50E");
                    }
                    if engine.player_tech.focused_laser {
                        draw_hud_button(ctx, x + 5.0, y + 68.0, 60.0, 50.0, "LASER", "175M 100E");
                    }
                }
                _ => {}
            }
        }
    }
}

fn draw_hud_button(ctx: &CanvasRenderingContext2d, x: f64, y: f64, w: f64, h: f64, label: &str, cost: &str) {
    ctx.set_fill_style(&JsValue::from_str("#0f2942"));
    ctx.fill_rect(x, y, w, h);
    ctx.set_stroke_style(&JsValue::from_str("#38bdf8"));
    ctx.set_line_width(1.0);
    ctx.stroke_rect(x, y, w, h);

    ctx.set_font("bold 10px monospace");
    ctx.set_fill_style(&JsValue::from_str("#e2e8f0"));
    let _ = ctx.fill_text(label, x + 4.0, y + 20.0);

    ctx.set_font("9px monospace");
    ctx.set_fill_style(&JsValue::from_str("#38bdf8"));
    let _ = ctx.fill_text(cost, x + 4.0, y + 38.0);
}
