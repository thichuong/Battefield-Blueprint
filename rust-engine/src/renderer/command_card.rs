use crate::components::{Building, BuildingType, FormationType, Team, UnitType};
use crate::GameEngine;
use web_sys::CanvasRenderingContext2d;

pub fn render_command_card(engine: &GameEngine, ctx: &CanvasRenderingContext2d, x: f64, y: f64) {
    ctx.set_fill_style_str("rgba(3, 13, 26, 0.94)");
    ctx.fill_rect(x, y, 215.0, 140.0);
    ctx.set_stroke_style_str("#0284c7");
    ctx.set_line_width(1.5);
    ctx.stroke_rect(x, y, 215.0, 140.0);

    let has_worker = engine.selected_unit_ids.iter().any(|uid| {
        engine
            .get_unit_by_id(*uid)
            .map(|u| u.unit_type == UnitType::Engineer)
            .unwrap_or(false)
    });

    let mut tooltip: Option<(String, String)> = None;

    if has_worker {
        let supply_cost = engine
            .config
            .get_building_cost_label(BuildingType::SupplyDepot);
        let reactor_cost = engine
            .config
            .get_building_cost_label(BuildingType::PowerReactor);
        let barracks_cost = engine
            .config
            .get_building_cost_label(BuildingType::Barracks);
        let turret_cost = engine
            .config
            .get_building_cost_label(BuildingType::TurretTower);
        let cc_cost = engine
            .config
            .get_building_cost_label(BuildingType::CommandCenter);
        let conv_cost = engine
            .config
            .get_building_cost_label(BuildingType::EnergyConverter);

        let supply_lbl = engine
            .config
            .get_building_short_name(BuildingType::SupplyDepot);
        let reactor_lbl = engine
            .config
            .get_building_short_name(BuildingType::PowerReactor);
        let barracks_lbl = engine
            .config
            .get_building_short_name(BuildingType::Barracks);
        let turret_lbl = engine
            .config
            .get_building_short_name(BuildingType::TurretTower);
        let cc_lbl = engine
            .config
            .get_building_short_name(BuildingType::CommandCenter);
        let conv_lbl = engine
            .config
            .get_building_short_name(BuildingType::EnergyConverter);

        draw_hud_button(ctx, x + 5.0, y + 6.0, 62.0, 40.0, supply_lbl, &supply_cost);
        draw_hud_button(
            ctx,
            x + 72.0,
            y + 6.0,
            62.0,
            40.0,
            reactor_lbl,
            &reactor_cost,
        );
        draw_hud_button(
            ctx,
            x + 139.0,
            y + 6.0,
            62.0,
            40.0,
            barracks_lbl,
            &barracks_cost,
        );
        draw_hud_button(ctx, x + 5.0, y + 50.0, 62.0, 40.0, turret_lbl, &turret_cost);
        draw_hud_button(ctx, x + 72.0, y + 50.0, 62.0, 40.0, cc_lbl, &cc_cost);
        draw_hud_button(ctx, x + 139.0, y + 50.0, 62.0, 40.0, conv_lbl, &conv_cost);

        // Hàng 3: Nút đổi đội hình kế tiếp (phím F)
        let cur_form = engine.team_formation(Team::Player);
        draw_hud_button(
            ctx,
            x + 5.0,
            y + 94.0,
            200.0,
            38.0,
            &format!("ĐỘI HÌNH [F]: {}", cur_form.button_label()),
            "Bấm để đổi kiểu đội hình",
        );

        if let Some(hover_idx) = engine.hovered_command_button {
            tooltip = match hover_idx {
                0 => {
                    let bt = BuildingType::SupplyDepot;
                    let title = engine.config.get_building_name(bt).to_string();
                    let bonus = engine
                        .config
                        .get_building(bt)
                        .map(|b| b.supply_bonus)
                        .unwrap_or(8);
                    Some((
                        title,
                        format!(
                            "Giá: {}. Tăng +{} Giới hạn dân số (Supply).",
                            supply_cost, bonus
                        ),
                    ))
                }
                1 => {
                    let bt = BuildingType::PowerReactor;
                    let title = engine.config.get_building_name(bt).to_string();
                    let en = engine
                        .config
                        .get_building(bt)
                        .map(|b| b.energy_per_sec)
                        .unwrap_or(8.0);
                    Some((
                        title,
                        format!(
                            "Giá: {}. Tăng tốc độ sản sinh Năng lượng +{:.0}/s.",
                            reactor_cost, en
                        ),
                    ))
                }
                2 => {
                    let bt = BuildingType::Barracks;
                    let title = engine.config.get_building_name(bt).to_string();
                    let desc = engine.config.get_building_description(bt);
                    Some((title, format!("Giá: {}. {}", barracks_cost, desc)))
                }
                3 => {
                    let bt = BuildingType::TurretTower;
                    let title = engine.config.get_building_name(bt).to_string();
                    let desc = engine.config.get_building_description(bt);
                    Some((title, format!("Giá: {}. {}", turret_cost, desc)))
                }
                4 => {
                    let bt = BuildingType::CommandCenter;
                    let title = engine.config.get_building_name(bt).to_string();
                    let desc = engine.config.get_building_description(bt);
                    Some((title, format!("Giá: {}. {}", cc_cost, desc)))
                }
                5 => {
                    let bt = BuildingType::EnergyConverter;
                    let title = engine.config.get_building_name(bt).to_string();
                    Some((
                        title,
                        format!(
                            "Giá: {}. Tự động chuyển Năng lượng thành Vàng khi Năng lượng >= {:.0}.",
                            conv_cost, engine.config.economy.converter_threshold
                        ),
                    ))
                }
                6..=8 => Some((
                    format!("ĐỘI HÌNH: {}", cur_form.display_name()),
                    format!(
                        "{} (Nhấn F hoặc Click để chuyển đổi)",
                        cur_form.description()
                    ),
                )),
                _ => None,
            };
        }
    } else if !engine.selected_unit_ids.is_empty() {
        // Đang chọn lính chiến đấu: Hiển thị 6 nút chọn trực tiếp Đội Hình Chiến Thuật (Formation Cards)
        let active_form = engine.team_formation(Team::Player);
        let forms = FormationType::ALL;

        for (idx, &form) in forms.iter().enumerate() {
            let row = idx / 3;
            let col = idx % 3;
            let bx = x + 5.0 + (col as f64) * 67.0;
            let by = y + 6.0 + (row as f64) * 44.0;
            let is_active = form == active_form;
            let is_hovered = engine.hovered_command_button == Some(idx);

            // Nền nút
            ctx.set_fill_style_str(if is_active {
                "#0c334f"
            } else if is_hovered {
                "#153859"
            } else {
                "#0a1c2e"
            });
            ctx.fill_rect(bx, by, 62.0, 40.0);

            // Viền nút
            ctx.set_stroke_style_str(if is_active {
                "#00e5ff"
            } else if is_hovered {
                "#38bdf8"
            } else {
                "#1e3a5f"
            });
            ctx.set_line_width(if is_active { 2.0 } else { 1.0 });
            ctx.stroke_rect(bx, by, 62.0, 40.0);

            // Icon / Sơ đồ chấm đội hình nhỏ
            draw_formation_mini_icon(ctx, bx + 31.0, by + 12.0, form, is_active);

            // Nhãn nút
            ctx.set_font(if is_active {
                "bold 10px monospace"
            } else {
                "9px monospace"
            });
            ctx.set_fill_style_str(if is_active { "#00e5ff" } else { "#94a3b8" });
            let _ = ctx.fill_text(form.button_label(), bx + 6.0, by + 34.0);
        }

        if let Some(hover_idx) = engine.hovered_command_button {
            if hover_idx < 6 {
                let hovered_form = FormationType::from_index(hover_idx);
                tooltip = Some((
                    format!("ĐỘI HÌNH: {}", hovered_form.display_name()),
                    hovered_form.description().to_string(),
                ));
            }
        }
    } else if let Some(bid) = engine.selected_building_id {
        if let Some(b) = engine
            .get_building_by_id(bid)
            .filter(|b| b.team == Team::Player)
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

                    let u_lbl = engine.config.get_unit_short_name(u_type);
                    draw_unit_purchase_button(
                        ctx,
                        x + 5.0,
                        y + 6.0,
                        62.0,
                        40.0,
                        u_lbl,
                        &cost_lbl,
                        count,
                        is_training,
                        prog,
                        engine.hovered_command_button == Some(0),
                        can_afford,
                    );

                    if engine.hovered_command_button == Some(0) {
                        let u_name = engine.config.get_unit_name(u_type);
                        let u_desc = engine.config.get_unit_description(u_type);
                        let mut desc = format!("Giá: {}. {}", cost_lbl, u_desc);
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
                        tooltip = Some((u_name.to_string(), desc));
                    }
                }
                BuildingType::Barracks => {
                    // 0: Striker
                    let s0 = draw_barracks_slot(
                        ctx,
                        engine,
                        b,
                        UnitType::ShockTrooper,
                        x + 5.0,
                        y + 6.0,
                        0,
                    );
                    // 1: Rifle
                    let s1 = draw_barracks_slot(
                        ctx,
                        engine,
                        b,
                        UnitType::Rifleman,
                        x + 72.0,
                        y + 6.0,
                        1,
                    );
                    // 2: Vanguard
                    let s2 = if engine.player_tech.energy_shielding {
                        Some(draw_barracks_slot(
                            ctx,
                            engine,
                            b,
                            UnitType::Vanguard,
                            x + 139.0,
                            y + 6.0,
                            2,
                        ))
                    } else {
                        None
                    };
                    // 3: Laser
                    let s3 = if engine.player_tech.focused_laser {
                        Some(draw_barracks_slot(
                            ctx,
                            engine,
                            b,
                            UnitType::LaserMarksman,
                            x + 5.0,
                            y + 50.0,
                            3,
                        ))
                    } else {
                        None
                    };
                    // 4: Dark Rocketeer
                    let s4 = if engine.player_tech.dark_matter_rockets {
                        Some(draw_barracks_slot(
                            ctx,
                            engine,
                            b,
                            UnitType::DarkRocketeer,
                            x + 72.0,
                            y + 50.0,
                            4,
                        ))
                    } else {
                        None
                    };
                    // 5: Taoist Swordmaster
                    let s5 = if engine.player_tech.celestial_flying_swords {
                        Some(draw_barracks_slot(
                            ctx,
                            engine,
                            b,
                            UnitType::TaoistSwordmaster,
                            x + 139.0,
                            y + 50.0,
                            5,
                        ))
                    } else {
                        None
                    };
                    // 6: Void Warlock
                    let s6 = if engine.player_tech.void_cataclysm {
                        Some(draw_barracks_slot(
                            ctx,
                            engine,
                            b,
                            UnitType::VoidWarlock,
                            x + 5.0,
                            y + 94.0,
                            6,
                        ))
                    } else {
                        None
                    };
                    // 7: Behemoth Titan
                    let s7 = if engine.player_tech.behemoth_mutation {
                        Some(draw_barracks_slot(
                            ctx,
                            engine,
                            b,
                            UnitType::BehemothTitan,
                            x + 72.0,
                            y + 94.0,
                            7,
                        ))
                    } else {
                        None
                    };
                    // 8: Acid Baneling
                    let s8 = if engine.player_tech.acid_banelings {
                        Some(draw_barracks_slot(
                            ctx,
                            engine,
                            b,
                            UnitType::AcidBaneling,
                            x + 139.0,
                            y + 94.0,
                            8,
                        ))
                    } else {
                        None
                    };

                    if let Some(hover_idx) = engine.hovered_command_button {
                        let target_unit_info = match hover_idx {
                            0 => Some((UnitType::ShockTrooper, s0)),
                            1 => Some((UnitType::Rifleman, s1)),
                            2 => s2.map(|s| (UnitType::Vanguard, s)),
                            3 => s3.map(|s| (UnitType::LaserMarksman, s)),
                            4 => s4.map(|s| (UnitType::DarkRocketeer, s)),
                            5 => s5.map(|s| (UnitType::TaoistSwordmaster, s)),
                            6 => s6.map(|s| (UnitType::VoidWarlock, s)),
                            7 => s7.map(|s| (UnitType::BehemothTitan, s)),
                            8 => s8.map(|s| (UnitType::AcidBaneling, s)),
                            _ => None,
                        };

                        if let Some((u_type, (count, is_tr, prog))) = target_unit_info {
                            let name = engine.config.get_unit_name(u_type).to_string();
                            let (mut min_c, mut en_c) = engine.config.get_unit_cost(u_type);
                            if engine.player_tech.hive_proliferation {
                                min_c *= 0.65;
                                en_c *= 0.65;
                            }
                            let cost_str = if en_c > 0.0 {
                                format!("{:.0}M {:.0}E", min_c, en_c)
                            } else {
                                format!("{:.0}M", min_c)
                            };
                            let u_desc = engine.config.get_unit_description(u_type);
                            let mut desc = format!("Giá: {}. {}", cost_str, u_desc);
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
                            tooltip = Some((name, desc));
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
    let cost_lbl = if en_cost > 0.0 {
        format!("{:.0}M {:.0}E", min_cost, en_cost)
    } else {
        format!("{:.0}M", min_cost)
    };
    let label = engine.config.get_unit_short_name(u_type);

    draw_unit_purchase_button(
        ctx,
        slot_x,
        slot_y,
        62.0,
        40.0,
        label,
        &cost_lbl,
        count,
        is_training,
        prog,
        engine.hovered_command_button == Some(btn_idx),
        can_afford,
    );

    (count, is_training, prog)
}

fn draw_formation_mini_icon(
    ctx: &CanvasRenderingContext2d,
    cx: f64,
    cy: f64,
    form: FormationType,
    is_active: bool,
) {
    let dot_color = if is_active { "#00e5ff" } else { "#64748b" };
    ctx.set_fill_style_str(dot_color);

    let draw_dot = |x: f64, y: f64| {
        ctx.begin_path();
        let _ = ctx.arc(x, y, 1.8, 0.0, std::f64::consts::PI * 2.0);
        ctx.fill();
    };

    match form {
        FormationType::Line => {
            // 2 hàng ngang
            draw_dot(cx - 10.0, cy - 3.0);
            draw_dot(cx, cy - 3.0);
            draw_dot(cx + 10.0, cy - 3.0);
            draw_dot(cx - 6.0, cy + 4.0);
            draw_dot(cx + 6.0, cy + 4.0);
        }
        FormationType::Wedge => {
            // Mũi tên chữ V
            draw_dot(cx, cy - 5.0);
            draw_dot(cx - 6.0, cy);
            draw_dot(cx + 6.0, cy);
            draw_dot(cx - 12.0, cy + 5.0);
            draw_dot(cx + 12.0, cy + 5.0);
        }
        FormationType::Box => {
            // Ô vuông viền
            draw_dot(cx - 8.0, cy - 5.0);
            draw_dot(cx + 8.0, cy - 5.0);
            draw_dot(cx - 8.0, cy + 5.0);
            draw_dot(cx + 8.0, cy + 5.0);
            draw_dot(cx, cy); // Lõi
        }
        FormationType::Scatter => {
            // So le tản rộng
            draw_dot(cx - 10.0, cy - 5.0);
            draw_dot(cx + 4.0, cy - 5.0);
            draw_dot(cx - 3.0, cy);
            draw_dot(cx + 10.0, cy + 4.0);
            draw_dot(cx - 9.0, cy + 5.0);
        }
        FormationType::Circle => {
            // Cung bán nguyệt
            draw_dot(cx - 11.0, cy + 4.0);
            draw_dot(cx - 6.0, cy - 2.0);
            draw_dot(cx, cy - 5.0);
            draw_dot(cx + 6.0, cy - 2.0);
            draw_dot(cx + 11.0, cy + 4.0);
        }
        FormationType::Column => {
            // 2 hàng dọc
            draw_dot(cx - 4.0, cy - 5.0);
            draw_dot(cx + 4.0, cy - 5.0);
            draw_dot(cx - 4.0, cy);
            draw_dot(cx + 4.0, cy);
            draw_dot(cx - 4.0, cy + 5.0);
            draw_dot(cx + 4.0, cy + 5.0);
        }
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
