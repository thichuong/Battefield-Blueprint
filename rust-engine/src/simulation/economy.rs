use crate::bridge::EngineToReactEvent;
use crate::components::{BuildingType, Team, UnitType};
use crate::map::{MAP_HEIGHT, MAP_WIDTH};
use crate::GameEngine;

pub fn update_camera(engine: &mut GameEngine, dt: f32) {
    let speed = 650.0 * engine.settings.camera_speed * dt / engine.camera.zoom;

    // Bàn phím điều khiển
    if engine.keys_down.contains("KeyW") || engine.keys_down.contains("ArrowUp") {
        engine.camera.y -= speed;
    }
    if engine.keys_down.contains("KeyS") || engine.keys_down.contains("ArrowDown") {
        engine.camera.y += speed;
    }
    if engine.keys_down.contains("ArrowLeft")
        || (engine.keys_down.contains("KeyA") && !engine.is_attack_move_mode)
    {
        engine.camera.x -= speed;
    }
    if engine.keys_down.contains("KeyD") || engine.keys_down.contains("ArrowRight") {
        engine.camera.x += speed;
    }

    // Cuộn mép màn hình (Edge-Scrolling) khi không nhấn giữ kéo chuột chọn
    if !engine.is_mouse_down && engine.camera.viewport_w > 0.0 && engine.camera.viewport_h > 0.0 {
        let edge_margin = 18.0;
        let edge_spd = speed * 1.1;
        if engine.mouse_screen_x >= 0.0 && engine.mouse_screen_x <= edge_margin {
            engine.camera.x -= edge_spd;
        } else if engine.mouse_screen_x >= engine.camera.viewport_w - edge_margin
            && engine.mouse_screen_x <= engine.camera.viewport_w
        {
            engine.camera.x += edge_spd;
        }
        if engine.mouse_screen_y >= 0.0 && engine.mouse_screen_y <= edge_margin {
            engine.camera.y -= edge_spd;
        } else if engine.mouse_screen_y >= engine.camera.viewport_h - edge_margin
            && engine.mouse_screen_y <= engine.camera.viewport_h
        {
            engine.camera.y += edge_spd;
        }
    }

    let min_x = (engine.camera.viewport_w * 0.5 / engine.camera.zoom).min(MAP_WIDTH * 0.5);
    let max_x = (MAP_WIDTH - engine.camera.viewport_w * 0.5 / engine.camera.zoom).max(min_x);
    let min_y = (engine.camera.viewport_h * 0.5 / engine.camera.zoom).min(MAP_HEIGHT * 0.5);
    let max_y = (MAP_HEIGHT - engine.camera.viewport_h * 0.5 / engine.camera.zoom).max(min_y);

    engine.camera.x = engine.camera.x.clamp(min_x, max_x);
    engine.camera.y = engine.camera.y.clamp(min_y, max_y);
}

pub use crate::config::{
    CONVERT_ENERGY_PER_SEC, CONVERT_MINERALS_PER_SEC, ENERGY_CONVERSION_THRESHOLD,
};

/// Tính toán năng lượng tiêu thụ và khoáng sản (minerals) nhận được từ quá trình chuyển đổi.
/// - Ngưỡng an toàn tối thiểu là 1000 Năng lượng. Dưới 1000 tự động dừng.
/// - Trả về (actual_energy_consumed, minerals_gained).
pub fn calculate_energy_conversion(energy: f32, num_converters: usize, dt: f32) -> (f32, f32) {
    if num_converters == 0 || energy <= ENERGY_CONVERSION_THRESHOLD {
        return (0.0, 0.0);
    }

    let surplus = energy - ENERGY_CONVERSION_THRESHOLD;
    let desired_energy = (num_converters as f32) * CONVERT_ENERGY_PER_SEC * dt;
    let actual_energy = desired_energy.min(surplus);

    if actual_energy > 0.0 {
        let minerals_gained = actual_energy * (CONVERT_MINERALS_PER_SEC / CONVERT_ENERGY_PER_SEC);
        (actual_energy, minerals_gained)
    } else {
        (0.0, 0.0)
    }
}

pub fn update_economy(engine: &mut GameEngine, dt: f32) {
    let mut team_reactors = [0usize; 8];
    let mut team_supply_depots = [0usize; 8];
    let mut team_converters = [0usize; 8];

    for b in &engine.buildings {
        if b.build_progress >= 1.0 {
            let t_idx = b.team.index() % 8;
            match b.building_type {
                BuildingType::PowerReactor => team_reactors[t_idx] += 1,
                BuildingType::SupplyDepot => team_supply_depots[t_idx] += 1,
                BuildingType::EnergyConverter => team_converters[t_idx] += 1,
                _ => {}
            }
        }
    }

    let base_energy = engine.config.economy.base_energy_per_sec;
    let reactor_energy = engine
        .config
        .get_building(BuildingType::PowerReactor)
        .map(|b| b.energy_per_sec)
        .unwrap_or(8.0);
    let base_supply = engine.config.economy.base_max_supply;
    let depot_supply = engine
        .config
        .get_building(BuildingType::SupplyDepot)
        .map(|b| b.supply_bonus)
        .unwrap_or(8);

    let mut unit_counts = [0u32; 8];
    for u in &engine.units {
        unit_counts[u.team.index() % 8] += 1;
    }

    let mut player_gained_minerals = 0.0;

    for i in 0..8 {
        if !engine.player_slots[i].active {
            continue;
        }
        let team = Team::from_index(i);
        let has_overcharged = engine.team_tech(team).overcharged_reactors;
        let mult = if has_overcharged { 1.5 } else { 1.0 };
        let energy_rate = (base_energy + team_reactors[i] as f32 * reactor_energy) * mult;

        let num_converters = team_converters[i];
        let eco = match team {
            Team::Player => &mut engine.player_eco,
            Team::Enemy => &mut engine.enemy_eco,
            _ => &mut engine.extra_ecos[i - 2],
        };
        eco.energy += energy_rate * dt;
        eco.energy_per_sec = energy_rate;

        // Chuyển hóa Năng lượng thành Khoáng sản
        if num_converters > 0 {
            let (used_energy, gained_minerals) =
                calculate_energy_conversion(eco.energy, num_converters, dt);
            if used_energy > 0.0 {
                eco.energy -= used_energy;
                eco.minerals += gained_minerals;
                if team == Team::Player {
                    player_gained_minerals += gained_minerals;
                }
                let consumed_per_sec =
                    (used_energy / dt).min((num_converters as f32) * CONVERT_ENERGY_PER_SEC);
                eco.energy_per_sec -= consumed_per_sec;
            }
        }

        eco.max_supply = base_supply + (team_supply_depots[i] as u32) * depot_supply;
        eco.supply = unit_counts[i];
    }

    engine.minerals_mined_accumulator += player_gained_minerals;

    // Tính tốc độ thu nhập khoáng sản cho Player
    engine.income_calc_timer += dt;
    if engine.income_calc_timer >= 1.0 {
        engine.player_eco.minerals_per_sec =
            engine.minerals_mined_accumulator / engine.income_calc_timer;
        engine.minerals_mined_accumulator = 0.0;
        engine.income_calc_timer = 0.0;
    }
}

pub fn update_fog(engine: &mut GameEngine) {
    engine.fog.reset_visibility_to_explored();
    let player_faction = engine.get_faction(Team::Player);

    // Tầm nhìn quanh công trình
    for b in &engine.buildings {
        if engine.get_faction(b.team) == player_faction {
            let range = engine
                .config
                .get_building(b.building_type)
                .map(|c| c.turret_range)
                .unwrap_or(0.0);
            let r = if range > 0.0 {
                (range + 50.0).max(320.0)
            } else {
                240.0
            };
            engine.fog.reveal_circle(b.x, b.y, r);
        }
    }

    // Tầm nhìn quanh lính
    for u in &engine.units {
        if engine.get_faction(u.team) == player_faction {
            let r = match u.unit_type {
                UnitType::LaserMarksman => 260.0,
                UnitType::Rifleman => 200.0,
                _ => 170.0,
            };
            engine.fog.reveal_circle(u.x, u.y, r);
        }
    }
}

pub fn check_game_over(engine: &mut GameEngine) {
    let player_faction = engine.get_faction(Team::Player);
    let my_faction_has_cc = engine.buildings.iter().any(|b| {
        engine.get_faction(b.team) == player_faction
            && b.building_type == BuildingType::CommandCenter
    });
    let enemy_factions_have_cc = engine.buildings.iter().any(|b| {
        engine.get_faction(b.team) != player_faction
            && b.building_type == BuildingType::CommandCenter
    });

    if !my_faction_has_cc {
        engine.game_over = true;
        engine.outgoing_events.push(EngineToReactEvent::GameOver {
            player_won: false,
            stats_json: "{}".to_string(),
        });
        engine.outgoing_events.push(EngineToReactEvent::PlaySound {
            sound: "defeat".to_string(),
        });
    } else if !enemy_factions_have_cc {
        engine.game_over = true;
        engine.outgoing_events.push(EngineToReactEvent::GameOver {
            player_won: true,
            stats_json: "{}".to_string(),
        });
        engine.outgoing_events.push(EngineToReactEvent::PlaySound {
            sound: "victory".to_string(),
        });
    }
}

pub fn send_sync_event(engine: &mut GameEngine) {
    let tech_json = serde_json::to_string(&engine.player_tech).unwrap_or_default();
    let autoplay_json = serde_json::to_string(&engine.autoplay.config).unwrap_or_default();
    let current_form = engine
        .team_formation(Team::Player)
        .button_label()
        .to_string();
    engine.outgoing_events.push(EngineToReactEvent::SyncState {
        player_minerals: engine.player_eco.minerals,
        player_energy: engine.player_eco.energy,
        player_supply: engine.player_eco.supply,
        player_max_supply: engine.player_eco.max_supply,
        tech_state: tech_json,
        autoplay_state: autoplay_json,
        player_formation: current_form,
        game_time: engine.game_time,
        is_paused: engine.is_paused,
    });
}
