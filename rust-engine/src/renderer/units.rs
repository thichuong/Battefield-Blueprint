use crate::components::{HarvestPhase, Team, Unit, UnitOrder, UnitType};
use crate::renderer::get_team_color;
use crate::GameEngine;
use web_sys::CanvasRenderingContext2d;

pub fn render_units(engine: &GameEngine) {
    let ctx = match engine.context.as_ref() {
        Some(c) => c,
        None => return,
    };
    for u in &engine.units {
        if !engine.camera.is_point_visible(u.x, u.y, 40.0) {
            continue;
        }

        let is_visible = engine.are_allies(Team::Player, u.team) || engine.fog.is_visible(u.x, u.y);
        if !is_visible {
            continue;
        }

        let is_selected = engine.selected_unit_ids.contains(&u.id);
        let color = get_team_color(engine, u.team).0;

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

        // Kiểm tra xem thợ có đang làm việc (khai thác mỏ hoặc xây dựng)
        let is_welding = match u.order {
            UnitOrder::Harvest {
                phase: HarvestPhase::Mining(_),
                ..
            } => true,
            UnitOrder::Build { target_building_id } => {
                if let Some(b) = engine.buildings.iter().find(|b| b.id == target_building_id) {
                    let dx = b.x - u.x;
                    let dy = b.y - u.y;
                    let dist = (dx * dx + dy * dy).sqrt();
                    dist <= (b.width.max(b.height) * 0.5 + 45.0) && b.build_progress < 1.0
                } else {
                    false
                }
            }
            _ => false,
        };

        // 2. Vẽ khung xương người que tối giản + vũ khí chi tiết & tư thế
        draw_procedural_stickman(ctx, u, color, engine.game_time as f64, is_welding);

        // 3. Thanh máu và thanh khiên trên đầu
        let hp_pct = (u.hp / u.max_hp).clamp(0.0, 1.0) as f64;
        let bar_w = 24.0;
        let bar_h = 3.5;
        let bar_x = (u.x - 12.0) as f64;
        let bar_y = (u.y - 32.0) as f64;

        ctx.set_fill_style_str("#1e293b");
        ctx.fill_rect(bar_x, bar_y, bar_w, bar_h);
        let hp_color = if hp_pct > 0.4 {
            &engine.config.render.hp_high_color
        } else {
            &engine.config.render.hp_low_color
        };
        ctx.set_fill_style_str(hp_color);
        ctx.fill_rect(bar_x, bar_y, bar_w * hp_pct, bar_h);

        // Thanh khiên (Shield Bar) màu xanh băng phía trên thanh máu
        if u.max_shield > 0.0 && u.shield > 0.0 {
            let shield_pct = (u.shield / u.max_shield).clamp(0.0, 1.0) as f64;
            ctx.set_fill_style_str("#0369a1");
            ctx.fill_rect(bar_x, bar_y - 4.0, bar_w, 2.5);
            ctx.set_fill_style_str("#38bdf8");
            ctx.fill_rect(bar_x, bar_y - 4.0, bar_w * shield_pct, 2.5);
        }
    }
}

fn draw_procedural_stickman(
    ctx: &CanvasRenderingContext2d,
    u: &Unit,
    color: &str,
    game_time: f64,
    is_welding: bool,
) {
    ctx.save();
    ctx.set_stroke_style_str(color);
    ctx.set_fill_style_str(color);
    ctx.set_line_width(2.0);

    let flip = if u.facing_right { 1.0 } else { -1.0 };
    let is_moving = !is_welding
        && matches!(
            u.order,
            UnitOrder::MoveTo { .. }
                | UnitOrder::AttackMove { .. }
                | UnitOrder::Harvest {
                    phase: HarvestPhase::MovingToNode | HarvestPhase::ReturningToBase,
                    ..
                }
                | UnitOrder::Build { .. }
        );
    let anim = u.attack_anim_timer as f64; // 1.0 -> 0.0

    let x = u.x as f64;
    let y = u.y as f64;

    // Riêng AcidBaneling là dạng bọ côn trùng đặc thù
    if u.unit_type == UnitType::AcidBaneling {
        let leg_phase = u.walk_cycle as f64;

        // Thân bọ bọc giáp hình bầu dục tĩnh, gọn gàng, không rung lắc
        ctx.set_fill_style_str("rgba(132, 204, 22, 0.35)");
        ctx.set_stroke_style_str("#a3e635");
        ctx.set_line_width(1.5);
        ctx.begin_path();
        let _ = ctx.ellipse(
            x - 3.0 * flip,
            y - 12.0,
            9.0,
            6.5,
            0.0,
            0.0,
            std::f64::consts::PI * 2.0,
        );
        ctx.fill();
        ctx.stroke();

        // Đầu bọ
        ctx.set_fill_style_str("#4d7c0f");
        ctx.begin_path();
        let _ = ctx.arc(
            x + 7.0 * flip,
            y - 8.0,
            3.5,
            0.0,
            std::f64::consts::PI * 2.0,
        );
        ctx.fill();

        // Cặp càng kẹp nhọn phía trước
        ctx.set_stroke_style_str("#d9f99d");
        ctx.set_line_width(1.5);
        ctx.begin_path();
        ctx.move_to(x + 8.0 * flip, y - 10.0);
        ctx.line_to(x + 13.0 * flip, y - 12.0);
        ctx.line_to(x + 14.0 * flip, y - 9.0);
        ctx.move_to(x + 8.0 * flip, y - 6.0);
        ctx.line_to(x + 13.0 * flip, y - 4.0);
        ctx.line_to(x + 14.0 * flip, y - 7.0);
        ctx.stroke();

        // 4 chân bọ đơn giản
        ctx.set_stroke_style_str("#a3e635");
        ctx.set_line_width(1.5);
        for i in 0..4 {
            let offset_x = (i as f64 - 1.5) * 4.5 * flip;
            let phase = leg_phase + (i as f64) * 1.5;
            let step_x = if is_moving { phase.cos() * 3.5 } else { 0.0 };

            let root_x = x + offset_x;
            let root_y = y - 7.0;
            let knee_x = root_x + (if i < 2 { -4.0 } else { 4.0 }) * flip + step_x * 0.5;
            let knee_y = root_y - 4.0;
            let foot_x = root_x + (if i < 2 { -7.0 } else { 7.0 }) * flip + step_x;
            let foot_y = y + 1.0;

            ctx.begin_path();
            ctx.move_to(root_x, root_y);
            ctx.line_to(knee_x, knee_y);
            ctx.line_to(foot_x, foot_y);
            ctx.stroke();
        }

        ctx.restore();
        return;
    }

    // Tỉ lệ scale cho BehemothTitan
    let scale = if u.unit_type == UnitType::BehemothTitan {
        1.35
    } else {
        1.0
    };

    // Tọa độ các khớp chính (Không còn rung/bobbing khi di chuyển)
    let pelvis_x = x;
    let pelvis_y = y - 7.0 * scale;
    let spine_tilt = if is_moving { 1.5 * flip } else { 0.0 };
    let chest_x = x + spine_tilt;
    let chest_y = y - 17.0 * scale;
    let head_x = chest_x + 0.5 * flip;
    let head_y = y - 24.0 * scale;

    // 1. CHÂN NGƯỜI QUE: Dáng đi thẳng gọn tối giản, không rung giật, không chấm khớp gối
    let leg_cycle = u.walk_cycle as f64;
    for leg_idx in 0..2 {
        let is_front_leg = leg_idx == 1;
        let phase = if is_front_leg {
            leg_cycle
        } else {
            leg_cycle + std::f64::consts::PI
        };

        let hip_x = pelvis_x + (if is_front_leg { 1.2 } else { -1.2 }) * flip;
        let hip_y = pelvis_y;

        let swing = if is_moving { phase.sin() * 0.35 } else { 0.0 };
        let leg_len = 13.5 * scale;
        let foot_x = hip_x + (swing * flip) * leg_len;
        let foot_y = y + 1.0;

        ctx.set_stroke_style_str(color);
        ctx.set_line_width(if is_front_leg {
            2.0 * scale
        } else {
            1.6 * scale
        });
        ctx.begin_path();
        ctx.move_to(hip_x, hip_y);
        ctx.line_to(foot_x, foot_y);
        ctx.stroke();
    }

    // 2. THÂN & CỘT SỐNG (Spine)
    ctx.set_stroke_style_str(color);
    ctx.set_line_width(2.2 * scale);
    ctx.begin_path();
    ctx.move_to(pelvis_x, pelvis_y);
    ctx.line_to(chest_x, chest_y);
    ctx.stroke();

    // 3. ĐẦU NGƯỜI QUE: Vòng tròn đơn giản, hoàn toàn bỏ bóng ở đầu
    ctx.set_line_width(1.8 * scale);
    ctx.begin_path();
    let _ = ctx.arc(head_x, head_y, 4.5 * scale, 0.0, std::f64::consts::PI * 2.0);
    ctx.stroke();

    // 4. VŨ KHÍ CẦM TAY CHI TIẾT & TƯ THẾ (Pose)
    let shoulder_x = chest_x;
    let shoulder_y = chest_y + 1.5 * scale;

    match u.unit_type {
        UnitType::Engineer => {
            // Engineer: Máy hàn cầm tay chi tiết
            if is_welding {
                let hand_x = shoulder_x + 9.0 * flip;
                let hand_y = shoulder_y + 4.0;

                // Hai tay giữ máy hàn
                ctx.set_stroke_style_str(color);
                ctx.set_line_width(1.8);
                // Tay trước
                ctx.begin_path();
                ctx.move_to(shoulder_x, shoulder_y);
                ctx.line_to(shoulder_x + 5.0 * flip, shoulder_y + 5.0);
                ctx.line_to(hand_x, hand_y);
                ctx.stroke();
                // Tay sau đỡ dưới
                ctx.begin_path();
                ctx.move_to(shoulder_x, shoulder_y);
                ctx.line_to(shoulder_x + 2.0 * flip, shoulder_y + 6.0);
                ctx.line_to(hand_x - 1.0 * flip, hand_y + 3.0);
                ctx.stroke();

                // Thân máy hàn hình hộp
                ctx.set_fill_style_str("#475569");
                ctx.fill_rect(hand_x - 3.0 * flip, hand_y - 2.5, 7.0 * flip, 5.0);

                // Đầu vòi khò hàn kim loại
                let nozzle_x = hand_x + 9.0 * flip;
                let nozzle_y = hand_y;
                ctx.set_stroke_style_str("#cbd5e1");
                ctx.set_line_width(2.0);
                ctx.begin_path();
                ctx.move_to(hand_x + 4.0 * flip, hand_y);
                ctx.line_to(nozzle_x, nozzle_y);
                ctx.stroke();

                // Dây cáp khí mềm nối từ đuôi máy hàn về thắt lưng
                ctx.set_stroke_style_str("#f59e0b");
                ctx.set_line_width(1.2);
                ctx.begin_path();
                ctx.move_to(hand_x - 3.0 * flip, hand_y + 1.5);
                ctx.quadratic_curve_to(
                    shoulder_x - 3.0 * flip,
                    shoulder_y + 9.0,
                    pelvis_x,
                    pelvis_y,
                );
                ctx.stroke();

                // Hiệu ứng tia lửa hàn lóe sáng tại đầu vòi hàn
                let seed = (game_time * 25.0).floor();
                ctx.set_stroke_style_str("#ffffff");
                ctx.set_line_width(1.2);
                for i in 0..4 {
                    let ang = ((seed * 1.7 + (i as f64) * 1.6).sin() * 1.2) - 0.2;
                    let len = 4.0 + ((seed * 2.3 + (i as f64)).cos().abs() * 5.0);
                    let sp_x = nozzle_x + ang.cos() * len * flip;
                    let sp_y = nozzle_y + ang.sin() * len;
                    ctx.begin_path();
                    ctx.move_to(nozzle_x, nozzle_y);
                    ctx.line_to(sp_x, sp_y);
                    ctx.stroke();
                }
                ctx.set_fill_style_str("#fef08a");
                ctx.begin_path();
                let _ = ctx.arc(nozzle_x, nozzle_y, 1.8, 0.0, std::f64::consts::PI * 2.0);
                ctx.fill();
            } else {
                // Đứng hoặc di chuyển: Cầm máy hàn chúc xuống gọn gàng
                let hand_x = shoulder_x + 6.0 * flip;
                let hand_y = shoulder_y + 6.0;

                ctx.set_stroke_style_str(color);
                ctx.set_line_width(1.8);
                // Tay trước cầm máy hàn
                ctx.begin_path();
                ctx.move_to(shoulder_x, shoulder_y);
                ctx.line_to(shoulder_x + 3.0 * flip, shoulder_y + 5.0);
                ctx.line_to(hand_x, hand_y);
                ctx.stroke();
                // Tay sau vung nhẹ
                ctx.begin_path();
                ctx.move_to(shoulder_x, shoulder_y);
                ctx.line_to(shoulder_x - 3.0 * flip, shoulder_y + 6.0);
                ctx.line_to(shoulder_x - 4.5 * flip, shoulder_y + 11.0);
                ctx.stroke();

                // Thân máy hàn chúc xuống
                ctx.set_fill_style_str("#475569");
                ctx.fill_rect(hand_x - 2.0 * flip, hand_y - 2.0, 5.0 * flip, 4.0);

                // Đầu vòi hàn
                ctx.set_stroke_style_str("#cbd5e1");
                ctx.set_line_width(1.8);
                ctx.begin_path();
                ctx.move_to(hand_x + 3.0 * flip, hand_y);
                ctx.line_to(hand_x + 7.0 * flip, hand_y + 2.5);
                ctx.stroke();

                // Dây cáp khí về thắt lưng
                ctx.set_stroke_style_str("#f59e0b");
                ctx.set_line_width(1.2);
                ctx.begin_path();
                ctx.move_to(hand_x - 2.0 * flip, hand_y + 1.0);
                ctx.quadratic_curve_to(
                    shoulder_x - 2.0 * flip,
                    shoulder_y + 8.0,
                    pelvis_x,
                    pelvis_y,
                );
                ctx.stroke();
            }
        }
        UnitType::ShockTrooper => {
            // ShockTrooper: Kiếm chi tiết. Dáng đi đơn giản. Tấn công là chém từ trên xuống!
            let (ang, arm_x, arm_y) = if anim > 0.0 {
                // Đang tấn công: Chém bổ từ trên đỉnh đầu xuống dưới
                if anim > 0.55 {
                    // Pha 1: Giơ cao kiếm qua đầu
                    let p = (1.0 - anim) / 0.45;
                    let a = -1.9 + p * 0.4;
                    (a, shoulder_x + 2.0 * flip, shoulder_y - 7.0)
                } else {
                    // Pha 2: Bổ mạnh từ trên đỉnh đầu xuống dưới chân
                    let p = (0.55 - anim) / 0.55;
                    let a = -1.5 + p * 2.4; // -1.5 rad -> +0.9 rad
                    (
                        a,
                        shoulder_x + (4.0 + p * 4.0) * flip,
                        shoulder_y - 3.0 + p * 7.0,
                    )
                }
            } else {
                // Tư thế chuẩn bị: Cầm kiếm nghiêng góc trước người
                (0.45, shoulder_x + 6.0 * flip, shoulder_y + 4.0)
            };

            // Tay cầm kiếm
            ctx.set_stroke_style_str(color);
            ctx.set_line_width(2.0);
            ctx.begin_path();
            ctx.move_to(shoulder_x, shoulder_y);
            ctx.line_to(arm_x, arm_y);
            ctx.stroke();

            // Tay sau giương nhẹ giữ thăng bằng
            ctx.begin_path();
            ctx.move_to(shoulder_x, shoulder_y);
            ctx.line_to(shoulder_x - 4.0 * flip, shoulder_y + 4.0);
            ctx.line_to(shoulder_x - 6.0 * flip, shoulder_y + 8.0);
            ctx.stroke();

            // VŨ KHÍ CHI TIẾT: Kiếm Plasma có chuôi, thanh chắn ngang và lưỡi vát
            // 1. Chuôi kiếm & Quả táo chuôi (Pommel)
            let pommel_x = arm_x - ang.cos() * 5.0 * flip;
            let pommel_y = arm_y - ang.sin() * 5.0;
            ctx.set_stroke_style_str("#64748b");
            ctx.set_line_width(2.0);
            ctx.begin_path();
            ctx.move_to(pommel_x, pommel_y);
            ctx.line_to(arm_x, arm_y);
            ctx.stroke();

            ctx.set_fill_style_str("#94a3b8");
            ctx.begin_path();
            let _ = ctx.arc(pommel_x, pommel_y, 1.5, 0.0, std::f64::consts::PI * 2.0);
            ctx.fill();

            // 2. Thanh chắn kiếm ngang (Crossguard)
            let g1_x = arm_x + ang.sin() * 4.0;
            let g1_y = arm_y - ang.cos() * 4.0 * flip;
            let g2_x = arm_x - ang.sin() * 4.0;
            let g2_y = arm_y + ang.cos() * 4.0 * flip;
            ctx.set_stroke_style_str("#38bdf8");
            ctx.set_line_width(2.2);
            ctx.begin_path();
            ctx.move_to(g1_x, g1_y);
            ctx.line_to(g2_x, g2_y);
            ctx.stroke();

            // 3. Lưỡi kiếm Plasma thẳng và sắc bén (không vệt cung tròn arc)
            let tip_x = arm_x + ang.cos() * 18.0 * flip;
            let tip_y = arm_y + ang.sin() * 18.0;
            ctx.set_stroke_style_str("#38bdf8");
            ctx.set_line_width(2.8);
            ctx.begin_path();
            ctx.move_to(arm_x, arm_y);
            ctx.line_to(tip_x, tip_y);
            ctx.stroke();

            // Sống kiếm sáng bóng
            ctx.set_stroke_style_str("#ffffff");
            ctx.set_line_width(1.0);
            ctx.begin_path();
            ctx.move_to(arm_x, arm_y);
            ctx.line_to(tip_x, tip_y);
            ctx.stroke();
        }
        UnitType::Rifleman => {
            // Rifleman: Súng trường chi tiết hơn, tư thế 2 tay cầm súng
            let recoil = if anim > 0.0 { anim * 2.5 } else { 0.0 };

            let hand_rear_x = shoulder_x + (3.0 - recoil) * flip;
            let hand_rear_y = shoulder_y + 3.0;
            let hand_front_x = shoulder_x + (9.0 - recoil) * flip;
            let hand_front_y = shoulder_y + 3.5;

            // Tay sau cầm báng/cò
            ctx.set_stroke_style_str(color);
            ctx.set_line_width(1.8);
            ctx.begin_path();
            ctx.move_to(shoulder_x, shoulder_y);
            ctx.line_to(hand_rear_x, hand_rear_y);
            ctx.stroke();

            // Tay trước đỡ ốp lót tay
            ctx.begin_path();
            ctx.move_to(shoulder_x, shoulder_y);
            ctx.line_to(shoulder_x + 5.0 * flip, shoulder_y + 6.0);
            ctx.line_to(hand_front_x, hand_front_y);
            ctx.stroke();

            // CHI TIẾT SÚNG TRƯỜNG:
            // 1. Báng súng (Stock) tỳ vai
            ctx.set_stroke_style_str("#475569");
            ctx.set_line_width(2.5);
            ctx.begin_path();
            ctx.move_to(hand_rear_x - 5.0 * flip, hand_rear_y + 1.0);
            ctx.line_to(hand_rear_x, hand_rear_y);
            ctx.stroke();

            // 2. Thân súng (Receiver)
            ctx.set_fill_style_str("#334155");
            ctx.fill_rect(hand_rear_x - 1.0 * flip, hand_rear_y - 2.0, 7.0 * flip, 3.5);

            // 3. Ống ngắm / Thước ngắm (Scope) trên thân súng
            ctx.set_stroke_style_str("#94a3b8");
            ctx.set_line_width(1.5);
            ctx.begin_path();
            ctx.move_to(hand_rear_x + 1.0 * flip, hand_rear_y - 3.2);
            ctx.line_to(hand_rear_x + 5.5 * flip, hand_rear_y - 3.2);
            ctx.stroke();

            // 4. Băng đạn cong (Curved Magazine)
            ctx.set_stroke_style_str("#64748b");
            ctx.set_line_width(2.0);
            ctx.begin_path();
            ctx.move_to(hand_rear_x + 2.5 * flip, hand_rear_y + 1.5);
            ctx.line_to(hand_rear_x + 4.0 * flip, hand_rear_y + 6.0);
            ctx.stroke();

            // 5. Nòng súng & Đầu giảm giật (Barrel)
            let barrel_tip_x = hand_front_x + 9.0 * flip;
            let barrel_tip_y = hand_front_y - 1.0;
            ctx.set_stroke_style_str("#94a3b8");
            ctx.set_line_width(2.0);
            ctx.begin_path();
            ctx.move_to(hand_front_x - 1.0 * flip, hand_front_y - 1.0);
            ctx.line_to(barrel_tip_x, barrel_tip_y);
            ctx.stroke();
        }
        UnitType::Vanguard => {
            // Vanguard: Khiên chiến thuật chi tiết & Thương xung kích
            let shield_push = if anim > 0.0 { anim * 3.5 } else { 0.0 };
            let s_hand_x = shoulder_x + (8.0 + shield_push) * flip;
            let s_hand_y = shoulder_y + 1.0;

            // Tay trước nâng khiên
            ctx.set_stroke_style_str(color);
            ctx.set_line_width(2.0);
            ctx.begin_path();
            ctx.move_to(shoulder_x, shoulder_y);
            ctx.line_to(shoulder_x + 4.0 * flip, shoulder_y + 3.0);
            ctx.line_to(s_hand_x, s_hand_y);
            ctx.stroke();

            // TẤM KHIÊN CHIẾN THUẬT CHI TIẾT:
            // Thân khiên kim loại màu xanh
            ctx.set_fill_style_str("rgba(56, 189, 248, 0.35)");
            ctx.set_stroke_style_str("#38bdf8");
            ctx.set_line_width(2.2);
            ctx.stroke_rect(s_hand_x - 2.5 * flip, s_hand_y - 13.0, 5.0 * flip, 26.0);
            ctx.fill_rect(s_hand_x - 2.5 * flip, s_hand_y - 13.0, 5.0 * flip, 26.0);

            // Khe ngắm trên khiên (Vision slit)
            ctx.set_stroke_style_str("#ffffff");
            ctx.set_line_width(1.2);
            ctx.begin_path();
            ctx.move_to(s_hand_x - 2.0 * flip, s_hand_y - 4.0);
            ctx.line_to(s_hand_x + 2.0 * flip, s_hand_y - 4.0);
            ctx.stroke();

            // Tay sau cầm thương/dùi cui xung kích
            let spear_thrust = if anim > 0.0 { anim * 6.0 } else { 0.0 };
            let spear_tip_x = s_hand_x + (6.0 + spear_thrust) * flip;
            let spear_tip_y = s_hand_y + 7.0;

            ctx.set_stroke_style_str(color);
            ctx.set_line_width(1.8);
            ctx.begin_path();
            ctx.move_to(shoulder_x, shoulder_y);
            ctx.line_to(shoulder_x - 3.0 * flip, shoulder_y + 5.0);
            ctx.line_to(shoulder_x + 2.0 * flip, shoulder_y + 8.0);
            ctx.stroke();

            // Thân thương & Mũi thương
            ctx.set_stroke_style_str("#94a3b8");
            ctx.set_line_width(2.0);
            ctx.begin_path();
            ctx.move_to(shoulder_x - 5.0 * flip, shoulder_y + 8.0);
            ctx.line_to(spear_tip_x, spear_tip_y);
            ctx.stroke();

            // Mũi nhọn
            ctx.set_fill_style_str("#38bdf8");
            ctx.begin_path();
            ctx.move_to(spear_tip_x + 3.0 * flip, spear_tip_y);
            ctx.line_to(spear_tip_x, spear_tip_y - 2.0);
            ctx.line_to(spear_tip_x, spear_tip_y + 2.0);
            ctx.close_path();
            ctx.fill();
        }
        UnitType::LaserMarksman => {
            // LaserMarksman: Pháo laser chi tiết
            let hand_x = shoulder_x + 10.0 * flip;
            let hand_y = shoulder_y + 2.5;

            // 2 tay nâng súng
            ctx.set_stroke_style_str(color);
            ctx.set_line_width(1.8);
            ctx.begin_path();
            ctx.move_to(shoulder_x, shoulder_y);
            ctx.line_to(shoulder_x + 4.0 * flip, shoulder_y + 4.0);
            ctx.line_to(hand_x, hand_y);
            ctx.stroke();

            // Thân súng hình hộp công nghệ cao
            ctx.set_fill_style_str("#3b0764");
            ctx.stroke_rect(hand_x - 5.0 * flip, hand_y - 3.0, 8.0 * flip, 6.0);
            ctx.fill_rect(hand_x - 5.0 * flip, hand_y - 3.0, 8.0 * flip, 6.0);

            // Nòng súng dài có 3 vòng cảm ứng năng lượng
            let gun_tip_x = hand_x + 13.0 * flip;
            let gun_tip_y = hand_y - 0.5;
            ctx.set_stroke_style_str("#c084fc");
            ctx.set_line_width(2.5);
            ctx.begin_path();
            ctx.move_to(hand_x + 3.0 * flip, gun_tip_y);
            ctx.line_to(gun_tip_x, gun_tip_y);
            ctx.stroke();

            // 3 vòng cảm ứng năng lượng
            ctx.set_stroke_style_str("#e879f9");
            ctx.set_line_width(1.5);
            for i in 1..4 {
                let rx = hand_x + (3.0 + (i as f64) * 2.5) * flip;
                ctx.begin_path();
                ctx.move_to(rx, gun_tip_y - 2.5);
                ctx.line_to(rx, gun_tip_y + 2.5);
                ctx.stroke();
            }

            // Tia sáng laser bắn thẳng sắc nét tới mục tiêu
            if let Some((target_x, target_y)) = u.last_attack_pos {
                let tx = target_x as f64;
                let ty = target_y as f64;
                let stacks = u.laser_stacks.min(25) as f64;
                let beam_width = 1.5 + stacks * 0.25;

                ctx.set_stroke_style_str("#c084fc");
                ctx.set_line_width(beam_width + 1.5);
                ctx.begin_path();
                ctx.move_to(gun_tip_x, gun_tip_y);
                ctx.line_to(tx, ty);
                ctx.stroke();

                ctx.set_stroke_style_str("#ffffff");
                ctx.set_line_width(beam_width * 0.6);
                ctx.begin_path();
                ctx.move_to(gun_tip_x, gun_tip_y);
                ctx.line_to(tx, ty);
                ctx.stroke();
            }
        }
        UnitType::DarkRocketeer => {
            // DarkRocketeer: Súng phóng tên lửa vác vai RPG chi tiết
            let hand_x = shoulder_x + 7.0 * flip;
            let hand_y = shoulder_y + 2.0;

            // Hai tay nâng tay cầm dưới ống
            ctx.set_stroke_style_str(color);
            ctx.set_line_width(1.8);
            ctx.begin_path();
            ctx.move_to(shoulder_x, shoulder_y);
            ctx.line_to(hand_x, hand_y);
            ctx.stroke();

            // Ống phóng vác vai
            let launch_y = shoulder_y - 4.5;
            let back_x = shoulder_x - 9.0 * flip;
            let front_x = shoulder_x + 12.0 * flip;

            // Thân ống phóng
            ctx.set_stroke_style_str("#ea580c");
            ctx.set_line_width(3.8);
            ctx.begin_path();
            ctx.move_to(back_x, launch_y);
            ctx.line_to(front_x, launch_y);
            ctx.stroke();

            // Loa xả khí loe rộng ở đuôi (Exhaust cone)
            ctx.set_stroke_style_str("#c2410c");
            ctx.set_line_width(2.0);
            ctx.begin_path();
            ctx.move_to(back_x + 1.5 * flip, launch_y - 3.0);
            ctx.line_to(back_x - 2.5 * flip, launch_y - 4.0);
            ctx.line_to(back_x - 2.5 * flip, launch_y + 4.0);
            ctx.line_to(back_x + 1.5 * flip, launch_y + 3.0);
            ctx.stroke();

            // Đầu đạn rocket nhọn ở phía trước
            ctx.set_fill_style_str("#ef4444");
            ctx.begin_path();
            ctx.move_to(front_x + 4.5 * flip, launch_y);
            ctx.line_to(front_x, launch_y - 3.0);
            ctx.line_to(front_x, launch_y + 3.0);
            ctx.close_path();
            ctx.fill();
        }
        UnitType::TaoistSwordmaster => {
            // TaoistSwordmaster: Cổ kiếm chi tiết & Kiếm quyết
            let hand_x = shoulder_x + 8.0 * flip;
            let hand_y = shoulder_y + 2.0;

            // Tay trước cầm cổ kiếm
            ctx.set_stroke_style_str(color);
            ctx.set_line_width(1.8);
            ctx.begin_path();
            ctx.move_to(shoulder_x, shoulder_y);
            ctx.line_to(hand_x, hand_y);
            ctx.stroke();

            // Lưỡi kiếm cầm tay
            let sw_tip_x = hand_x + 14.0 * flip;
            let sw_tip_y = hand_y - 6.0;
            ctx.set_stroke_style_str("#38bdf8");
            ctx.set_line_width(2.2);
            ctx.begin_path();
            ctx.move_to(hand_x, hand_y);
            ctx.line_to(sw_tip_x, sw_tip_y);
            ctx.stroke();

            // Chắn kiếm chữ thập nhỏ
            ctx.set_stroke_style_str("#facc15");
            ctx.set_line_width(2.0);
            ctx.begin_path();
            ctx.move_to(hand_x - 2.0 * flip, hand_y - 2.0);
            ctx.line_to(hand_x + 2.0 * flip, hand_y + 2.0);
            ctx.stroke();

            // Tay sau kết kiếm quyết chỉ thẳng ra sau / nâng cao
            ctx.set_stroke_style_str(color);
            ctx.set_line_width(1.8);
            ctx.begin_path();
            ctx.move_to(shoulder_x, shoulder_y);
            ctx.line_to(shoulder_x - 5.0 * flip, shoulder_y + 3.0);
            ctx.line_to(shoulder_x - 8.0 * flip, shoulder_y - 2.0);
            ctx.stroke();

            // Các phi kiếm lơ lửng phía sau lưng (tĩnh, trang nhã, không rung lắc)
            let swords_count = u.flying_swords.min(7) as usize;
            for i in 0..swords_count {
                let arc_offset = ((i as f64) - (swords_count as f64 - 1.0) * 0.5) * 5.0;
                let sw_x = shoulder_x - (9.0 + (i as f64) * 1.5) * flip;
                let sw_y = shoulder_y - 12.0 + arc_offset;

                // Thân phi kiếm
                ctx.set_stroke_style_str("#38bdf8");
                ctx.set_line_width(2.0);
                ctx.begin_path();
                ctx.move_to(sw_x, sw_y);
                ctx.line_to(sw_x + 11.0 * flip, sw_y - 4.5);
                ctx.stroke();

                // Chuôi kiếm vàng
                ctx.set_stroke_style_str("#facc15");
                ctx.set_line_width(1.5);
                ctx.begin_path();
                ctx.move_to(sw_x - 1.0 * flip, sw_y + 1.0);
                ctx.line_to(sw_x + 1.0 * flip, sw_y - 1.0);
                ctx.stroke();
            }
        }
        UnitType::VoidWarlock => {
            // VoidWarlock: Quyền trượng Hư Không chi tiết
            let staff_lift = if anim > 0.0 { anim * 4.0 } else { 0.0 };
            let hand_x = shoulder_x + 7.0 * flip;
            let hand_y = shoulder_y - 2.0 - staff_lift;

            // Tay nâng trượng
            ctx.set_stroke_style_str(color);
            ctx.set_line_width(1.8);
            ctx.begin_path();
            ctx.move_to(shoulder_x, shoulder_y);
            ctx.line_to(hand_x, hand_y);
            ctx.stroke();

            // Tay sau giương nhẹ
            ctx.begin_path();
            ctx.move_to(shoulder_x, shoulder_y);
            ctx.line_to(shoulder_x - 4.0 * flip, shoulder_y + 5.0);
            ctx.stroke();

            // Cán trượng
            let staff_top_x = hand_x + 2.0 * flip;
            let staff_top_y = hand_y - 16.0;
            let staff_bot_x = hand_x - 2.0 * flip;
            let staff_bot_y = hand_y + 13.0;
            ctx.set_stroke_style_str("#7e22ce");
            ctx.set_line_width(2.2);
            ctx.begin_path();
            ctx.move_to(staff_bot_x, staff_bot_y);
            ctx.line_to(staff_top_x, staff_top_y);
            ctx.stroke();

            // Đầu trượng hình trăng khuyết / đôi sừng
            ctx.set_stroke_style_str("#c084fc");
            ctx.set_line_width(1.8);
            ctx.begin_path();
            let _ = ctx.arc(
                staff_top_x,
                staff_top_y,
                4.5,
                -std::f64::consts::FRAC_PI_2,
                std::f64::consts::FRAC_PI_2,
            );
            ctx.stroke();

            // Viên tinh thể hư không hình thoi ở giữa
            ctx.set_fill_style_str("#e879f9");
            ctx.begin_path();
            ctx.move_to(staff_top_x, staff_top_y - 3.5);
            ctx.line_to(staff_top_x + 2.5 * flip, staff_top_y);
            ctx.line_to(staff_top_x, staff_top_y + 3.5);
            ctx.line_to(staff_top_x - 2.5 * flip, staff_top_y);
            ctx.close_path();
            ctx.fill();
        }
        UnitType::BehemothTitan => {
            // BehemothTitan: Chiến búa / Chùy gai khổng lồ, bổ từ trên xuống
            let (arm_ang, hand_x, hand_y) = if anim > 0.0 {
                if anim > 0.5 {
                    // Giơ búa lên đỉnh đầu
                    let p = (1.0 - anim) / 0.5;
                    (
                        -1.8 + p * 0.3,
                        shoulder_x + 3.0 * scale * flip,
                        shoulder_y - 8.0 * scale,
                    )
                } else {
                    // Bổ mạnh xuống đất
                    let p = (0.5 - anim) / 0.5;
                    (
                        -1.5 + p * 2.3,
                        shoulder_x + (5.0 + p * 5.0) * scale * flip,
                        shoulder_y - 2.0 * scale + p * 8.0 * scale,
                    )
                }
            } else {
                (
                    0.3,
                    shoulder_x + 8.0 * scale * flip,
                    shoulder_y + 4.0 * scale,
                )
            };

            // Tay cầm búa
            ctx.set_stroke_style_str(color);
            ctx.set_line_width(3.0 * scale);
            ctx.begin_path();
            ctx.move_to(shoulder_x, shoulder_y);
            ctx.line_to(hand_x, hand_y);
            ctx.stroke();

            // Cán búa dài
            let club_head_x = hand_x + arm_ang.cos() * 16.0 * scale * flip;
            let club_head_y = hand_y + arm_ang.sin() * 16.0 * scale;
            ctx.set_stroke_style_str("#92400e");
            ctx.set_line_width(3.5 * scale);
            ctx.begin_path();
            ctx.move_to(
                hand_x - arm_ang.cos() * 5.0 * scale * flip,
                hand_y - arm_ang.sin() * 5.0 * scale,
            );
            ctx.line_to(club_head_x, club_head_y);
            ctx.stroke();

            // Đầu búa chiến bọc thép góc cạnh
            ctx.set_fill_style_str("#f59e0b");
            ctx.fill_rect(
                club_head_x - 5.0 * scale,
                club_head_y - 5.0 * scale,
                10.0 * scale,
                10.0 * scale,
            );

            // Gai nhọn 2 bên đầu búa
            ctx.set_stroke_style_str("#d97706");
            ctx.set_line_width(2.0 * scale);
            ctx.begin_path();
            ctx.move_to(club_head_x, club_head_y - 7.0 * scale);
            ctx.line_to(club_head_x, club_head_y + 7.0 * scale);
            ctx.stroke();
        }
        UnitType::AcidBaneling => {}
    }

    ctx.restore();
}
