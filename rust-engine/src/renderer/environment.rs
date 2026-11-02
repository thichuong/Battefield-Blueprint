use crate::fog::Visibility;
use crate::map::{MAP_HEIGHT, MAP_WIDTH};
use crate::GameEngine;

pub fn render_blueprint_grid(engine: &GameEngine) {
    let ctx = match engine.context.as_ref() {
        Some(c) => c,
        None => return,
    };
    ctx.set_stroke_style_str(&engine.config.render.grid_line_color);
    ctx.set_line_width(1.0);
    ctx.begin_path();

    let grid_size = engine.config.render.grid_size.max(10.0);
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

pub fn render_obstacles(engine: &GameEngine) {
    let ctx = match engine.context.as_ref() {
        Some(c) => c,
        None => return,
    };
    for obs in &engine.obstacles {
        if !engine
            .camera
            .is_rect_visible(obs.x, obs.y, obs.width, obs.height, 50.0)
        {
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

pub fn render_minerals(engine: &GameEngine) {
    let ctx = match engine.context.as_ref() {
        Some(c) => c,
        None => return,
    };
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

pub fn render_fog_of_war(engine: &GameEngine) {
    let ctx = match engine.context.as_ref() {
        Some(c) => c,
        None => return,
    };
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
