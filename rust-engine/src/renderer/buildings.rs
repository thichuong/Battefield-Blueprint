use super::get_team_color;
use crate::components::*;
use crate::renderer::gpu_cache::get_color_index;
use crate::GameEngine;
use web_sys::CanvasRenderingContext2d;

pub fn render_buildings(engine: &GameEngine) {
    let ctx = match engine.context.as_ref() {
        Some(c) => c,
        None => return,
    };

    let player_faction = engine.get_faction(Team::Player);
    let team_factions = engine.team_factions();

    for b in &engine.buildings {
        let bw = b.width;
        let bh = b.height;
        let bx_f32 = b.x - bw * 0.5;
        let by_f32 = b.y - bh * 0.5;
        if !engine.camera.is_rect_visible(bx_f32, by_f32, bw, bh, 50.0) {
            continue;
        }

        let is_visible = engine.are_allies(Team::Player, b.team) || engine.fog.is_visible(b.x, b.y);
        if !is_visible {
            continue;
        }

        let is_selected = engine.selected_building_id == Some(b.id);
        let is_hit_flash = b.hit_flash_timer > 0.0;
        let color_idx = get_color_index(b.team, is_hit_flash, player_faction, &team_factions);

        let bx = (b.x - b.width * 0.5) as f64;
        let by = (b.y - b.height * 0.5) as f64;
        let bw = b.width as f64;
        let bh = b.height as f64;

        // Vẽ từ GPU Texture Atlas
        let mut rendered_from_cache = false;
        if let Some(ref cache) = engine.gpu_cache {
            let bt_idx = b.building_type as usize;
            if bt_idx < cache.building_sprites.len()
                && color_idx < cache.building_sprites[bt_idx].len()
            {
                let sprite_canvas = &cache.building_sprites[bt_idx][color_idx];
                let dx = bx - 10.0;
                let dy = by - 5.0;
                let _ = ctx.draw_image_with_html_canvas_element(sprite_canvas, dx, dy);
                rendered_from_cache = true;
            }
        }

        if !rendered_from_cache {
            let (base_neon, color_fill) = get_team_color(engine, b.team);
            let color_neon = if is_hit_flash { "#ffffff" } else { base_neon };

            // Khung nhà Blueprint
            ctx.set_fill_style_str(color_fill);
            ctx.fill_rect(bx, by, bw, bh);

            ctx.set_stroke_style_str(if is_selected {
                &engine.config.render.selection_color
            } else {
                color_neon
            });
            ctx.set_line_width(if is_selected { 2.5 } else { 1.5 });
            ctx.stroke_rect(bx, by, bw, bh);

            // Ký hiệu góc kỹ thuật
            draw_blueprint_brackets(ctx, bx, by, bw, bh, color_neon);

            // Đồ họa 2D Tĩnh Cho Công Trình
            draw_building_graphic(
                ctx,
                b.building_type,
                b.x as f64,
                b.y as f64,
                bw,
                bh,
                color_neon,
            );

            // Tên công trình hiển thị bên dưới
            ctx.set_font("bold 10px monospace");
            ctx.set_text_align("center");
            ctx.set_fill_style_str(color_neon);
            let title = engine.config.get_building_short_name(b.building_type);
            let _ = ctx.fill_text(title, b.x as f64, by + bh + 13.0);
            ctx.set_text_align("left");
        }

        // Viền chọn highlight nếu được chọn
        if is_selected {
            ctx.set_stroke_style_str(&engine.config.render.selection_color);
            ctx.set_line_width(2.5);
            ctx.stroke_rect(bx, by, bw, bh);

            // Hiển thị vòng tròn tầm bắn của công trình khi được chọn
            let b_cfg = engine.config.get_building(b.building_type);
            let range = b_cfg.map(|c| c.turret_range).unwrap_or(0.0);
            if range > 0.0 {
                ctx.set_stroke_style_str("rgba(0, 229, 255, 0.45)");
                ctx.set_line_width(1.5);
                ctx.begin_path();
                let _ = ctx.arc(
                    b.x as f64,
                    b.y as f64,
                    range as f64,
                    0.0,
                    std::f64::consts::PI * 2.0,
                );
                ctx.stroke();
            }
        }

        // Thanh tiến độ xây dựng hoặc Thanh máu
        if b.build_progress < 1.0 {
            ctx.set_fill_style_str("#334155");
            ctx.fill_rect(bx, by - 12.0, bw, 6.0);
            let bar_color = if b.under_attack_timer > 0.0 {
                &engine.config.render.build_under_attack_color
            } else {
                &engine.config.render.build_progress_color
            };
            ctx.set_fill_style_str(bar_color);
            ctx.fill_rect(bx, by - 12.0, bw * (b.build_progress as f64), 6.0);
        } else {
            // Thanh máu HP bar
            let hp_pct = (b.hp / b.max_hp).clamp(0.0, 1.0) as f64;
            ctx.set_fill_style_str("#1e293b");
            ctx.fill_rect(bx, by - 10.0, bw, 5.0);
            let hp_color = if hp_pct > 0.4 {
                &engine.config.render.hp_high_color
            } else {
                &engine.config.render.hp_low_color
            };
            ctx.set_fill_style_str(hp_color);
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

pub fn draw_blueprint_brackets(
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

pub fn draw_building_graphic(
    ctx: &CanvasRenderingContext2d,
    b_type: BuildingType,
    cx: f64,
    cy: f64,
    bw: f64,
    bh: f64,
    color: &str,
) {
    ctx.save();
    ctx.set_stroke_style_str(color);
    ctx.set_fill_style_str(color);
    ctx.set_line_width(1.8);

    match b_type {
        BuildingType::CommandCenter => {
            // 1. HQ CommandCenter: Khối tháp pháo đài chỉ huy bát giác kiên cố + ăng-ten radar ở giữa
            let r_outer = (bw.min(bh) * 0.32).max(18.0);
            let r_inner = r_outer * 0.6;

            // Viền bát giác ngoài
            ctx.begin_path();
            for i in 0..8 {
                let ang = (i as f64) * std::f64::consts::PI / 4.0;
                let px = cx + ang.cos() * r_outer;
                let py = cy + ang.sin() * r_outer;
                if i == 0 {
                    ctx.move_to(px, py);
                } else {
                    ctx.line_to(px, py);
                }
            }
            ctx.close_path();
            ctx.stroke();

            // Khối phòng ngự lõi trung tâm
            ctx.stroke_rect(cx - r_inner, cy - r_inner, r_inner * 2.0, r_inner * 2.0);

            // Ăng-ten radar trung tâm
            ctx.begin_path();
            let _ = ctx.arc(cx, cy, 3.5, 0.0, std::f64::consts::PI * 2.0);
            ctx.fill();

            // 4 chấu tiếp đất bảo vệ ở 4 hướng chính
            ctx.begin_path();
            ctx.move_to(cx - r_outer - 5.0, cy);
            ctx.line_to(cx - r_inner, cy);
            ctx.move_to(cx + r_inner, cy);
            ctx.line_to(cx + r_outer + 5.0, cy);
            ctx.move_to(cx, cy - r_outer - 5.0);
            ctx.line_to(cx, cy - r_inner);
            ctx.move_to(cx, cy + r_inner);
            ctx.line_to(cx, cy + r_outer + 5.0);
            ctx.stroke();
        }
        BuildingType::SupplyDepot => {
            // 2. Supply Depot: 4 kiện hàng công-te-nơ tiếp vận (2x2) có dây chằng chéo X
            let box_size = (bw.min(bh) * 0.30).max(12.0);
            let gap = 3.5;
            let offsets = [
                (-box_size - gap * 0.5, -box_size - gap * 0.5),
                (gap * 0.5, -box_size - gap * 0.5),
                (-box_size - gap * 0.5, gap * 0.5),
                (gap * 0.5, gap * 0.5),
            ];

            for (ox, oy) in offsets {
                let rx = cx + ox;
                let ry = cy + oy;
                ctx.stroke_rect(rx, ry, box_size, box_size);

                // Dây chằng chéo chữ X trên từng kiện hàng
                ctx.begin_path();
                ctx.move_to(rx, ry);
                ctx.line_to(rx + box_size, ry + box_size);
                ctx.move_to(rx + box_size, ry);
                ctx.line_to(rx, ry + box_size);
                ctx.stroke();
            }
        }
        BuildingType::PowerReactor => {
            // 3. Power Reactor: Lõi lò phản ứng hạt nhân tĩnh - Vòng tròn đồng tâm + 3 cánh quạt năng lượng
            let r = (bw.min(bh) * 0.35).max(15.0);

            // Vòng tròn ngoài
            ctx.begin_path();
            let _ = ctx.arc(cx, cy, r, 0.0, std::f64::consts::PI * 2.0);
            ctx.stroke();

            // Lõi phát điện trung tâm
            ctx.begin_path();
            let _ = ctx.arc(cx, cy, 4.0, 0.0, std::f64::consts::PI * 2.0);
            ctx.fill();

            // 3 cánh quạt / thanh năng lượng hạt nhân phân kỳ 120 độ
            for i in 0..3 {
                let ang =
                    (i as f64) * std::f64::consts::PI * 2.0 / 3.0 - std::f64::consts::FRAC_PI_2;
                ctx.begin_path();
                ctx.move_to(cx + ang.cos() * 5.0, cy + ang.sin() * 5.0);
                ctx.line_to(cx + ang.cos() * (r - 2.5), cy + ang.sin() * (r - 2.5));
                ctx.stroke();
            }
        }
        BuildingType::Barracks => {
            // 4. Barracks: Doanh trại huấn luyện - Mái che/khung trại + biểu tượng 2 kiếm chéo tĩnh
            let hw = bw * 0.36;
            let hh = bh * 0.34;

            // Khung lều trại quân đội
            ctx.begin_path();
            ctx.move_to(cx - hw, cy + hh);
            ctx.line_to(cx - hw, cy - hh * 0.3);
            ctx.line_to(cx, cy - hh);
            ctx.line_to(cx + hw, cy - hh * 0.3);
            ctx.line_to(cx + hw, cy + hh);
            ctx.close_path();
            ctx.stroke();

            // Biểu tượng kiếm chéo quân đội ở giữa
            let sw_len = 9.0;
            ctx.begin_path();
            ctx.move_to(cx - sw_len, cy - sw_len * 0.7);
            ctx.line_to(cx + sw_len, cy + sw_len * 0.7);
            ctx.move_to(cx + sw_len, cy - sw_len * 0.7);
            ctx.line_to(cx - sw_len, cy + sw_len * 0.7);
            ctx.stroke();
        }
        BuildingType::TurretTower => {
            // 5. TurretTower: Bệ pháo phòng thủ tự động - Đế tròn/bát giác + Ụ pháo 2 nòng súng
            let r = (bw.min(bh) * 0.34).max(12.0);

            // Đế tháp pháo
            ctx.begin_path();
            let _ = ctx.arc(cx, cy + 2.0, r, 0.0, std::f64::consts::PI * 2.0);
            ctx.stroke();

            // Ụ xoay pháo
            ctx.begin_path();
            let _ = ctx.arc(cx, cy + 2.0, r * 0.55, 0.0, std::f64::consts::PI * 2.0);
            ctx.fill();

            // 2 nòng pháo vươn lên trên
            let gun_w = 3.2;
            let gun_h = r * 1.1;
            ctx.fill_rect(cx - gun_w * 1.5, cy + 2.0 - gun_h, gun_w, gun_h);
            ctx.fill_rect(cx + gun_w * 0.5, cy + 2.0 - gun_h, gun_w, gun_h);
        }
        BuildingType::EnergyConverter => {
            // 6. EnergyConverter: Trạm chuyển hóa - Hình thoi tinh thể kim cương nằm trong vòng tròn tĩnh
            let r = (bw.min(bh) * 0.36).max(15.0);

            // Vòng tròn chuyển hóa
            ctx.begin_path();
            let _ = ctx.arc(cx, cy, r, 0.0, std::f64::consts::PI * 2.0);
            ctx.stroke();

            // Tinh thể kim cương hình thoi tĩnh ở trung tâm
            let dw = r * 0.7;
            let dh = r * 0.85;
            ctx.begin_path();
            ctx.move_to(cx, cy - dh);
            ctx.line_to(cx + dw, cy);
            ctx.line_to(cx, cy + dh);
            ctx.line_to(cx - dw, cy);
            ctx.close_path();
            ctx.stroke();

            // Tâm năng lượng
            ctx.begin_path();
            let _ = ctx.arc(cx, cy, 3.0, 0.0, std::f64::consts::PI * 2.0);
            ctx.fill();
        }
    }

    ctx.restore();
}
