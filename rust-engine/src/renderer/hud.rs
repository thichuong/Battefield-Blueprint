use crate::components::{BuildingType, HarvestPhase, Team, UnitOrder, UnitType};
use crate::map::{MAP_HEIGHT, MAP_WIDTH};
use crate::renderer::command_card::render_command_card;
use crate::renderer::get_team_color;
use crate::GameEngine;

thread_local! {
    static DASH_ARRAY: js_sys::Array = {
        let arr = js_sys::Array::new();
        arr.push(&wasm_bindgen::JsValue::from_f64(3.0));
        arr.push(&wasm_bindgen::JsValue::from_f64(2.0));
        arr
    };
    static EMPTY_DASH_ARRAY: js_sys::Array = js_sys::Array::new();
}

pub fn render_in_game_hud(engine: &GameEngine) {
    let ctx = match engine.context.as_ref() {
        Some(c) => c,
        None => return,
    };
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

    // Tiến trình nghiên cứu hiện tại (hỗ trợ hiển thị nhiều nhánh song song)
    if !engine.player_tech.active_researches.is_empty() {
        ctx.set_fill_style_str("#facc15");
        let list_str = engine
            .player_tech
            .active_researches
            .iter()
            .map(|r| format!("{} ({:.0}%)", r.tech_id, r.progress * 100.0))
            .collect::<Vec<_>>()
            .join(", ");
        let _ = ctx.fill_text(&format!("🔬 RESEARCHING: {}", list_str), 610.0, 29.0);
    } else if let Some(ref tech_id) = engine.player_tech.current_research {
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

    // 0. Huy hiệu đo FPS thời gian thực trên Top Bar (Click hoặc F3 để xem chi tiết)
    let fps_btn_x = vw - 380.0;
    let fps_val = engine.fps.round();
    let (fps_color, fps_bg) = if fps_val >= 50.0 {
        ("#10b981", "rgba(6, 78, 59, 0.45)")
    } else if fps_val >= 30.0 {
        ("#facc15", "rgba(113, 63, 18, 0.45)")
    } else {
        ("#ef4444", "rgba(127, 29, 29, 0.45)")
    };
    ctx.set_fill_style_str(fps_bg);
    ctx.fill_rect(fps_btn_x, 10.0, 78.0, 28.0);
    ctx.set_stroke_style_str(fps_color);
    ctx.set_line_width(1.0);
    ctx.stroke_rect(fps_btn_x, 10.0, 78.0, 28.0);
    ctx.set_fill_style_str(fps_color);
    ctx.set_font("bold 12px monospace");
    ctx.set_text_align("center");
    let _ = ctx.fill_text(&format!("{:.0} FPS", fps_val), fps_btn_x + 39.0, 28.0);
    ctx.set_text_align("left");

    // Nút mở Chế độ Tự Động [B]
    let auto_btn_x = vw - 295.0;
    let auto_active = engine.autoplay.config.enabled;
    if auto_active {
        ctx.set_fill_style_str("#065f46");
        ctx.fill_rect(auto_btn_x, 10.0, 95.0, 28.0);
        ctx.set_stroke_style_str("#34d399");
        ctx.set_line_width(1.5);
        ctx.stroke_rect(auto_btn_x, 10.0, 95.0, 28.0);
        ctx.set_fill_style_str("#6ee7b7");
        ctx.set_font("bold 12px monospace");
        let _ = ctx.fill_text("⚡AUTO:ON", auto_btn_x + 8.0, 28.0);
    } else {
        ctx.set_fill_style_str("#1e293b");
        ctx.fill_rect(auto_btn_x, 10.0, 95.0, 28.0);
        ctx.set_stroke_style_str("#475569");
        ctx.set_line_width(1.0);
        ctx.stroke_rect(auto_btn_x, 10.0, 95.0, 28.0);
        ctx.set_fill_style_str("#94a3b8");
        ctx.set_font("bold 12px monospace");
        let _ = ctx.fill_text("AUTO [B]", auto_btn_x + 14.0, 28.0);
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

    // Tuyến phòng thủ TurretDefenseLine trên Minimap
    let def_line = &engine.autoplay.config.turret_defense;
    if def_line.enabled {
        let p1x = mm_x + (def_line.p1.0 / MAP_WIDTH) as f64 * mm_s;
        let p1y = mm_y + (def_line.p1.1 / MAP_HEIGHT) as f64 * mm_s;
        let p2x = mm_x + (def_line.p2.0 / MAP_WIDTH) as f64 * mm_s;
        let p2y = mm_y + (def_line.p2.1 / MAP_HEIGHT) as f64 * mm_s;

        ctx.set_stroke_style_str("rgba(245, 158, 11, 0.85)");
        ctx.set_line_width(1.5);
        DASH_ARRAY.with(|dash| {
            let _ = ctx.set_line_dash(dash);
        });
        ctx.begin_path();
        ctx.move_to(p1x, p1y);
        ctx.line_to(p2x, p2y);
        ctx.stroke();
        EMPTY_DASH_ARRAY.with(|empty| {
            let _ = ctx.set_line_dash(empty);
        });

        // Điểm A (Cyan) & Điểm B (Purple)
        ctx.set_fill_style_str("#06b6d4");
        ctx.begin_path();
        let _ = ctx.arc(p1x, p1y, 2.5, 0.0, std::f64::consts::PI * 2.0);
        ctx.fill();

        ctx.set_fill_style_str("#a855f7");
        ctx.begin_path();
        let _ = ctx.arc(p2x, p2y, 2.5, 0.0, std::f64::consts::PI * 2.0);
        ctx.fill();
    }

    // Vẽ mỏ quặng trên Minimap (chỉ vẽ nếu đã từng được thám hiểm)
    ctx.set_fill_style_str("#38bdf8");
    for m in &engine.minerals {
        if engine.fog.is_explored(m.x, m.y) {
            let mx = mm_x + (m.x / MAP_WIDTH) as f64 * mm_s;
            let my = mm_y + (m.y / MAP_HEIGHT) as f64 * mm_s;
            ctx.fill_rect(mx - 1.5, my - 1.5, 3.0, 3.0);
        }
    }

    // Vẽ công trình trên Minimap gom nhóm theo Team (giảm số lần đổi context fill_style)
    for i in 0..8 {
        let team = Team::from_index(i);
        let color = get_team_color(engine, team).0;
        let mut color_set = false;

        for b in &engine.buildings {
            if b.team == team
                && (engine.are_allies(Team::Player, b.team) || engine.fog.is_explored(b.x, b.y))
            {
                if !color_set {
                    ctx.set_fill_style_str(color);
                    color_set = true;
                }
                let mx = mm_x + (b.x / MAP_WIDTH) as f64 * mm_s;
                let my = mm_y + (b.y / MAP_HEIGHT) as f64 * mm_s;
                ctx.fill_rect(mx - 2.5, my - 2.5, 5.0, 5.0);
            }
        }
    }

    // Phủ sương mù chiến tranh lên Minimap (Minimap Fog of War Overlay)
    if let Some(ref cache) = engine.gpu_cache {
        let _ = ctx.draw_image_with_html_canvas_element_and_dw_and_dh(
            &cache.fog_canvas,
            mm_x,
            mm_y,
            mm_s,
            mm_s,
        );
    } else {
        use crate::fog::{Visibility, FOG_COLS, FOG_ROWS};
        let cell_mm = mm_s / (FOG_COLS as f64);
        for r in 0..FOG_ROWS {
            let ry = mm_y + (r as f64) * cell_mm;
            let mut current_vis = engine.fog.grid[r * FOG_COLS];
            let mut start_c = 0;
            let mut count = 0;

            for c in 0..FOG_COLS {
                let vis = engine.fog.grid[r * FOG_COLS + c];
                if vis == current_vis {
                    count += 1;
                } else {
                    if current_vis == Visibility::Hidden {
                        ctx.set_fill_style_str("rgba(2, 6, 13, 0.72)");
                        ctx.fill_rect(
                            mm_x + (start_c as f64) * cell_mm,
                            ry,
                            (count as f64) * cell_mm + 0.3,
                            cell_mm + 0.3,
                        );
                    }
                    current_vis = vis;
                    start_c = c;
                    count = 1;
                }
            }
            if current_vis == Visibility::Hidden {
                ctx.set_fill_style_str("rgba(2, 6, 13, 0.72)");
                ctx.fill_rect(
                    mm_x + (start_c as f64) * cell_mm,
                    ry,
                    (count as f64) * cell_mm + 0.3,
                    cell_mm + 0.3,
                );
            }
        }
    }

    // Vẽ lính trên Minimap gom nhóm theo Team
    for i in 0..8 {
        let team = Team::from_index(i);
        let color = get_team_color(engine, team).0;
        let mut color_set = false;

        for u in &engine.units {
            if u.team == team
                && (engine.are_allies(Team::Player, u.team) || engine.fog.is_visible(u.x, u.y))
            {
                if !color_set {
                    ctx.set_fill_style_str(color);
                    color_set = true;
                }
                let mx = mm_x + (u.x / MAP_WIDTH) as f64 * mm_s;
                let my = mm_y + (u.y / MAP_HEIGHT) as f64 * mm_s;
                ctx.fill_rect(mx - 1.5, my - 1.5, 3.0, 3.0);
            }
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
    let ctx = match engine.context.as_ref() {
        Some(c) => c,
        None => return,
    };
    ctx.set_fill_style_str("rgba(3, 13, 26, 0.94)");
    ctx.fill_rect(x, y, w, h);
    ctx.set_stroke_style_str("#0284c7");
    ctx.set_line_width(1.5);
    ctx.stroke_rect(x, y, w, h);

    // 1. Nếu đang chọn 1 công trình
    if let Some(bid) = engine.selected_building_id {
        if let Some(b) = engine.get_building_by_id(bid) {
            let color = if b.team == Team::Player {
                "#00e5ff"
            } else {
                "#f43f5e"
            };
            ctx.set_font("bold 13px monospace");
            ctx.set_fill_style_str(color);
            let title = engine.config.get_building_name(b.building_type);
            let _ = ctx.fill_text(title, x + 15.0, y + 22.0);

            // Thanh máu chi tiết
            let hp_pct = (b.hp / b.max_hp).clamp(0.0, 1.0) as f64;
            ctx.set_fill_style_str("#1e293b");
            ctx.fill_rect(x + 15.0, y + 30.0, w - 30.0, 10.0);
            let hp_color = if hp_pct > 0.4 {
                &engine.config.render.hp_high_color
            } else {
                &engine.config.render.hp_low_color
            };
            ctx.set_fill_style_str(hp_color);
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
                let threshold = engine.config.economy.converter_threshold;
                let en_rate = crate::config::CONVERT_ENERGY_PER_SEC;
                let min_rate = crate::config::CONVERT_MINERALS_PER_SEC;

                if cur_energy >= threshold {
                    ctx.set_fill_style_str("#fbbf24");
                    let _ = ctx.fill_text(
                        &format!(
                            "STATUS: CONVERTING ({:.0}⚡ ➔ {:.0}💎/s)",
                            en_rate, min_rate
                        ),
                        x + 15.0,
                        y + 85.0,
                    );
                    ctx.set_font("10px monospace");
                    ctx.set_fill_style_str("#38bdf8");
                    let _ = ctx.fill_text(
                        &format!(
                            "Tự động chuyển Năng lượng dư thừa (>{:.0}⚡) thành Vàng.",
                            threshold
                        ),
                        x + 15.0,
                        y + 105.0,
                    );
                } else {
                    ctx.set_fill_style_str("#f97316");
                    let _ = ctx.fill_text(
                        &format!("STATUS: STANDBY (Năng lượng < {:.0}⚡)", threshold),
                        x + 15.0,
                        y + 85.0,
                    );
                    ctx.set_font("10px monospace");
                    ctx.set_fill_style_str("#94a3b8");
                    let _ = ctx.fill_text(
                        &format!(
                            "Cần thêm {:.0}⚡ để tự động kích hoạt.",
                            (threshold - cur_energy).max(0.0)
                        ),
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
                        &format!(
                            "⚔️ ATK: {:.0} [{}] | RNG: {:.0}",
                            b_damage,
                            b.damage_type.display_name(),
                            b_range
                        ),
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
                let has_dark_rockets =
                    b.team == Team::Player && engine.player_tech.dark_matter_rockets;
                let has_overload = b.team == Team::Player && engine.player_tech.overload_strike;

                let eff_damage = if has_dark_rockets {
                    b_damage * 1.4
                } else {
                    b_damage
                };
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
                let _ = ctx.fill_text(
                    "Targeting: Automatic hostile engagement",
                    x + 15.0,
                    y + 112.0,
                );
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
        if let Some(u) = engine.get_unit_by_id(uid) {
            ctx.set_font("bold 13px monospace");
            ctx.set_fill_style_str(if u.team == Team::Player {
                "#00e5ff"
            } else {
                "#f43f5e"
            });
            let name = engine.config.get_unit_name(u.unit_type);
            let _ = ctx.fill_text(name, x + 15.0, y + 22.0);

            // Thanh máu (+ Thanh khiên nếu có)
            let hp_pct = (u.hp / u.max_hp).clamp(0.0, 1.0) as f64;
            ctx.set_fill_style_str("#1e293b");
            ctx.fill_rect(x + 15.0, y + 30.0, w - 30.0, 10.0);
            let hp_color = if hp_pct > 0.4 {
                &engine.config.render.hp_high_color
            } else {
                &engine.config.render.hp_low_color
            };
            ctx.set_fill_style_str(hp_color);
            ctx.fill_rect(x + 15.0, y + 30.0, (w - 30.0) * hp_pct, 10.0);

            // Vạch khiên đè lên thanh trạng thái nếu có
            if u.max_shield > 0.0 {
                let shield_pct = (u.shield / u.max_shield).clamp(0.0, 1.0) as f64;
                ctx.set_fill_style_str("#0284c7");
                ctx.fill_rect(x + 15.0, y + 27.0, w - 30.0, 3.0);
                ctx.set_fill_style_str("#38bdf8");
                ctx.fill_rect(x + 15.0, y + 27.0, (w - 30.0) * shield_pct, 3.0);
            }

            ctx.set_font("bold 10px monospace");
            ctx.set_fill_style_str("#ffffff");
            if u.max_shield > 0.0 {
                let _ = ctx.fill_text(
                    &format!(
                        "{:.0}/{:.0} HP  [🛡️Khiên: {:.0}/{:.0}]",
                        u.hp, u.max_hp, u.shield, u.max_shield
                    ),
                    x + 20.0,
                    y + 39.0,
                );
            } else {
                let _ = ctx.fill_text(
                    &format!("{:.0} / {:.0} HP", u.hp, u.max_hp),
                    x + 20.0,
                    y + 39.0,
                );
            }

            // Chỉ số chiến đấu & Loại sát thương (hiển thị đặc thù cho từng đơn vị)
            ctx.set_font("11px monospace");
            ctx.set_fill_style_str("#e2e8f0");
            let atk_str = if u.unit_type == UnitType::LaserMarksman {
                let cur_dmg = 1.0 + (u.laser_stacks as f32) * 2.0;
                format!(
                    "⚔️ ATK: {:.0} (Stack {}) [ENERGY]   🎯 RNG: {:.0}",
                    cur_dmg, u.laser_stacks, u.attack_range
                )
            } else if u.unit_type == UnitType::TaoistSwordmaster {
                format!(
                    "⚔️ ATK: {:.0} [ARCANE]   🎯 RNG: {:.0}   🗡️ KIẾM: {}/7",
                    u.attack_damage, u.attack_range, u.flying_swords
                )
            } else if u.unit_type == UnitType::VoidWarlock {
                let dps_pct = if u.percent_max_hp_dps > 0.0 {
                    u.percent_max_hp_dps * 100.0
                } else {
                    engine
                        .config
                        .get_unit(UnitType::VoidWarlock)
                        .map(|cfg| cfg.percent_max_hp_dps)
                        .filter(|&p| p > 0.0)
                        .unwrap_or(crate::config::VOID_WARLOCK_DEFAULT_MAX_HP_DPS)
                        * 100.0
                };
                format!(
                    "⚔️ PHÁP TRẬN: {:.0}% MaxHP/s   🎯 RNG: {:.0}   ⚡ CD: {:.1}s",
                    dps_pct, u.attack_range, u.attack_cooldown
                )
            } else if u.unit_type == UnitType::AcidBaneling {
                format!(
                    "⚔️ CẢM TỬ: {:.0} (x2 Nhà)   💥 AOE: {:.0}",
                    u.attack_damage, u.aoe_radius
                )
            } else {
                format!(
                    "⚔️ ATK: {:.0} [{}]   🎯 RNG: {:.0}   🏃 SPD: {:.0}",
                    u.attack_damage,
                    u.damage_type.display_name(),
                    u.attack_range,
                    u.speed
                )
            };
            let _ = ctx.fill_text(&atk_str, x + 15.0, y + 60.0);

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
            if let Some(u) = engine.get_unit_by_id(*uid) {
                match u.unit_type {
                    UnitType::Engineer => workers += 1,
                    UnitType::ShockTrooper
                    | UnitType::Rifleman
                    | UnitType::Vanguard
                    | UnitType::LaserMarksman => {
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
        ctx.set_font("bold 11px monospace");
        ctx.set_fill_style_str("#38bdf8");
        let active_form = engine.team_formation(Team::Player);
        let _ = ctx.fill_text(
            &format!("📐 ĐỘI HÌNH: {}", active_form.display_name()),
            x + 15.0,
            y + 98.0,
        );
        ctx.set_font("10px monospace");
        ctx.set_fill_style_str("#64748b");
        let _ = ctx.fill_text(
            "Phím F / Command Card: Đổi đội hình  |  Phím A: Attack-Move",
            x + 15.0,
            y + 118.0,
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

/// Bảng thông số hiệu năng FPS và thời gian xử lý (Debug Performance Overlay)
pub fn render_debug_overlay(engine: &GameEngine) {
    if !engine.show_debug_overlay {
        return;
    }
    let ctx = match engine.context.as_ref() {
        Some(c) => c,
        None => return,
    };
    let vw = engine.camera.viewport_w as f64;
    let panel_w = 270.0;
    let panel_h = 75.0;
    let px = vw - panel_w - 12.0;
    let py = 56.0;

    ctx.set_fill_style_str("rgba(3, 13, 26, 0.88)");
    ctx.fill_rect(px, py, panel_w, panel_h);
    ctx.set_stroke_style_str("#10b981");
    ctx.set_line_width(1.0);
    ctx.stroke_rect(px, py, panel_w, panel_h);

    ctx.set_font("bold 11px monospace");
    ctx.set_fill_style_str("#10b981");
    let limit_tag = match engine.settings.fps_limit.as_str() {
        "vsync" => "V-Sync",
        "unlimited" => "Max",
        other => other,
    };
    let _ = ctx.fill_text(
        &format!(
            "⚡ FPS: {:.0} [{}] | Sim: {:.1}ms | Rnd: {:.1}ms",
            engine.fps, limit_tag, engine.sim_time_ms, engine.render_time_ms
        ),
        px + 10.0,
        py + 20.0,
    );
    ctx.set_fill_style_str("#38bdf8");
    let _ = ctx.fill_text(
        &format!(
            "👤 Units: {} | 🏛 Buildings: {}",
            engine.units.len(),
            engine.buildings.len()
        ),
        px + 10.0,
        py + 40.0,
    );
    ctx.set_fill_style_str("#facc15");
    let _ = ctx.fill_text(
        &format!(
            "🚀 Projectiles: {} | ✨ Particles: {}",
            engine.projectiles.len(),
            engine.particles.len()
        ),
        px + 10.0,
        py + 60.0,
    );
}
