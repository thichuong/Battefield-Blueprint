pub mod buildings;
pub mod combat;
pub mod commands;
pub mod economy;
pub mod hud_input;
pub mod input;
pub mod units;

#[cfg(test)]
mod tests;

pub use buildings::*;
pub use combat::*;
pub use commands::*;
pub use economy::*;
pub use hud_input::*;
pub use input::*;
pub use units::*;

use crate::components::Team;
use crate::GameEngine;

pub fn step(engine: &mut GameEngine, dt: f32) {
    // 1. Xử lý các lệnh từ React
    if !engine.is_headless {
        process_commands(engine);
    }

    if engine.is_paused || engine.game_over {
        return;
    }

    let dt = if engine.is_headless { dt } else { dt.min(0.05) };
    engine.game_time += dt;

    // 2. Di chuyển Camera bằng phím WASD / Mũi tên và Edge Scrolling
    if !engine.is_headless {
        update_camera(engine, dt);
    }

    // 3. Cập nhật Kinh tế tự nhiên & Đo tốc độ thu nhập
    update_economy(engine, dt);

    // 4. Cập nhật Tiến trình Nghiên cứu Cây Công Nghệ
    update_tech_research(engine, dt);

    // 5. Cập nhật Đơn vị Lính (Di chuyển, Khai mỏ, Tấn công mép nhà, Attack-Move)
    update_units(engine, dt);

    // 6. Cập nhật Công trình (Xây dựng, Huấn luyện lính, Tháp pháo bắn)
    update_buildings(engine, dt);

    // 6.5. Cập nhật Vòng ma thuật của VoidWarlock
    update_magic_circles(engine, dt);

    // 7. Cập nhật Đạn & Tia Laser (sát thương đơn vị & công trình)
    update_projectiles(engine, dt);

    // 8. Cập nhật Hiệu ứng Chữ nổi (Floating numbers) & Fog
    if !engine.is_headless {
        update_floating_texts(engine, dt);
        update_fog(engine);
    }

    // 10. Cập nhật AI Đối thủ (kinh tế + điều quân càn quét nhà)
    engine.ai_think_timer += dt;
    if engine.ai_think_timer >= 0.5 {
        let elapsed = engine.ai_think_timer;
        engine.ai_think_timer = 0.0;
        crate::ai::update_ai_with_dt(engine, elapsed);
    }

    // 10.5 Cập nhật Chế độ Tự Động (Auto-Play) cho tất cả team có autoplay bật
    for i in 0..crate::components::MAX_PLAYERS {
        let team = Team::from_index(i);
        if engine.is_slot_active(team) && engine.team_autoplay(team).config.enabled {
            crate::autoplay::update_team_autoplay(engine, team, dt);
        }
    }

    // 11. Kiểm tra Điều kiện Thắng / Thua
    check_game_over(engine);

    // 12. Bắn sự kiện định kỳ đồng bộ về React
    if !engine.is_headless {
        engine.sync_timer += dt;
        if engine.sync_timer >= 0.1 {
            engine.sync_timer = 0.0;
            send_sync_event(engine);
        }
    }
}
