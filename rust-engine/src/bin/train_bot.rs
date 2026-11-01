use rust_engine::ai::extract_state_vector;
use rust_engine::components::{BuildingType, Team, UnitType};
use rust_engine::rl::BotStrategicAction;
use rust_engine::GameEngine;
use std::env;
use std::fs::File;
use std::io::Write;
use std::time::Instant;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut total_episodes = 100;
    let mut time_scale_dt = 0.15_f32;
    let max_game_time = 600.0_f32; // 10 phút tối đa mỗi trận
    let mut output_path = "assets/rl_model.json".to_string();

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--episodes" | "-e" => {
                if i + 1 < args.len() {
                    total_episodes = args[i + 1].parse().unwrap_or(100);
                    i += 1;
                }
            }
            "--dt" => {
                if i + 1 < args.len() {
                    time_scale_dt = args[i + 1].parse().unwrap_or(0.15);
                    i += 1;
                }
            }
            "--output" | "-o" => {
                if i + 1 < args.len() {
                    output_path = args[i + 1].clone();
                    i += 1;
                }
            }
            _ => {}
        }
        i += 1;
    }

    println!("============================================================");
    println!("🧠 BATTLEFIELD BLUEPRINT // DIRECT RL BOT TRAINER");
    println!("   Episodes:       {}", total_episodes);
    println!("   Fast-Forward dt: {:.3}s/step", time_scale_dt);
    println!("   Max Game Time:  {:.0}s", max_game_time);
    println!("   Output Model:   {}", output_path);
    println!("============================================================");

    let start_wall_time = Instant::now();
    let mut wins = 0;
    let mut losses = 0;
    let mut draws = 0;
    let mut action_counts = [0usize; 11];

    let mut engine = GameEngine::new_headless();
    engine.ai_state.training_enabled = true;
    engine.ai_state.rl_brain.epsilon = 0.20;

    for episode in 1..=total_episodes {
        engine.reset_game();
        engine.is_headless = true;
        engine.is_paused = false;
        engine.ai_state.training_enabled = true;

        // Player chạy Auto-Play Sparring
        engine.autoplay.config.enabled = true;
        match episode % 4 {
            0 => {
                // Sparring: Rush
                engine.autoplay.config.priority_techs = vec![];
            }
            1 => {
                // Sparring: Science Tech
                engine.autoplay.config.priority_techs = vec![
                    "focused_laser".to_string(),
                    "energy_shielding".to_string(),
                    "dark_matter_rockets".to_string(),
                ];
            }
            2 => {
                // Sparring: Magic Tech
                engine.autoplay.config.priority_techs = vec![
                    "celestial_flying_swords".to_string(),
                    "golden_bell_shield".to_string(),
                ];
            }
            _ => {
                // Sparring: Swarm Eco
                engine.autoplay.config.priority_techs = vec![
                    "optical_drills".to_string(),
                    "acid_banelings".to_string(),
                ];
            }
        }

        let mut ep_reward = 0.0_f32;
        let mut last_print_t = 0.0_f32;

        while !engine.game_over && engine.game_time < max_game_time {
            let prev_state = extract_state_vector(&engine);
            let prev_combat_enemy = count_combat_units(&engine, Team::Enemy);
            let prev_combat_player = count_combat_units(&engine, Team::Player);
            let prev_p_cc = count_ccs(&engine, Team::Player);
            let prev_e_cc = count_ccs(&engine, Team::Enemy);
            let prev_p_workers = count_workers(&engine, Team::Player);
            let prev_p_bldgs = count_buildings(&engine, Team::Player);

            rust_engine::simulation::step(&mut engine, time_scale_dt);

            if episode == 1 && engine.game_time - last_print_t >= 30.0 {
                last_print_t = engine.game_time;
                println!(
                    "      [Ep1 T={:5.1}s] Bot(Min={:4.0}, En={:4.0}, Sup={:2}/{}, W={}, Mil={}, Tech={}) vs Player(Min={:4.0}, Mil={}, B={}, Tech={})",
                    engine.game_time,
                    engine.enemy_eco.minerals, engine.enemy_eco.energy, engine.enemy_eco.supply, engine.enemy_eco.max_supply,
                    count_workers(&engine, Team::Enemy), count_combat_units(&engine, Team::Enemy),
                    rust_engine::ai::count_unlocked_techs(&engine.enemy_tech),
                    engine.player_eco.minerals, count_combat_units(&engine, Team::Player), count_buildings(&engine, Team::Player),
                    rust_engine::ai::count_unlocked_techs(&engine.player_tech)
                );
            }

            let cur_state = extract_state_vector(&engine);
            let action = engine.ai_state.current_action;
            action_counts[action.to_index()] += 1;

            // --- REWARD SHAPING MẠNH MẼ CHO BOT THÔNG MINH ---
            let mut step_reward = 0.0_f32;

            // 1. Khuyến khích diệt thợ mỏ và triệt tiêu kinh tế người chơi
            let cur_p_workers = count_workers(&engine, Team::Player);
            if prev_p_workers > cur_p_workers {
                step_reward += (prev_p_workers - cur_p_workers) as f32 * 10.0;
            }

            // 2. Thưởng khi phá hủy công trình của Player (Đặc biệt là Barracks / Turrets / CC)
            let cur_p_bldgs = count_buildings(&engine, Team::Player);
            if prev_p_bldgs > cur_p_bldgs {
                step_reward += (prev_p_bldgs - cur_p_bldgs) as f32 * 30.0;
            }

            // 3. Tiêu diệt lính chiến đấu Player
            let cur_combat_player = count_combat_units(&engine, Team::Player);
            if prev_combat_player > cur_combat_player {
                step_reward += (prev_combat_player - cur_combat_player) as f32 * 4.0;
            }

            // Phạt nếu Bot mất lính
            let cur_combat_enemy = count_combat_units(&engine, Team::Enemy);
            if prev_combat_enemy > cur_combat_enemy {
                step_reward -= (prev_combat_enemy - cur_combat_enemy) as f32 * 1.5;
            }

            // 4. Phá hủy hoặc mất CommandCenter
            let cur_p_cc = count_ccs(&engine, Team::Player);
            if prev_p_cc > cur_p_cc {
                step_reward += 150.0;
            }
            let cur_e_cc = count_ccs(&engine, Team::Enemy);
            if prev_e_cc > cur_e_cc {
                step_reward -= 150.0;
            }

            // 5. Phạt nhẹ thời gian để tránh Bot dây dưa không chịu tấn công dứt điểm
            step_reward -= 0.015;

            // 6. Thưởng thắng trận dứt khoát
            let is_done = engine.game_over || engine.game_time >= max_game_time;
            if engine.game_over {
                let p_alive = engine.buildings.iter().any(|b| b.team == Team::Player && b.building_type == BuildingType::CommandCenter);
                if !p_alive {
                    // Thắng càng nhanh càng được nhiều điểm thưởng
                    step_reward += 400.0 + (max_game_time - engine.game_time) * 0.2;
                } else {
                    step_reward -= 250.0;
                }
            } else if engine.game_time >= max_game_time {
                step_reward -= 100.0;
            }

            ep_reward += step_reward;
            engine.ai_state.rl_brain.train_step(&prev_state, action, step_reward, &cur_state, is_done);

            if is_done {
                break;
            }
        }

        let p_cc_count = count_ccs(&engine, Team::Player);
        let e_cc_count = count_ccs(&engine, Team::Enemy);
        let p_comb = count_combat_units(&engine, Team::Player);
        let e_comb = count_combat_units(&engine, Team::Enemy);
        let p_w = count_workers(&engine, Team::Player);
        let e_w = count_workers(&engine, Team::Enemy);
        let p_b = count_buildings(&engine, Team::Player);
        let e_b = count_buildings(&engine, Team::Enemy);

        if episode <= 5 {
            println!(
                "   [Ep {:2} End] Time: {:.1}s | Bot: CC={} B={} W={} Mil={} | Player: CC={} B={} W={} Mil={}",
                episode, engine.game_time, e_cc_count, e_b, e_w, e_comb, p_cc_count, p_b, p_w, p_comb
            );
        }

        if e_cc_count > 0 && p_cc_count == 0 {
            wins += 1;
        } else if p_cc_count > 0 && e_cc_count == 0 {
            losses += 1;
        } else {
            draws += 1;
        }

        engine.ai_state.rl_brain.epsilon = (engine.ai_state.rl_brain.epsilon * 0.985).max(0.02);

        if episode % 20 == 0 || episode == total_episodes {
            let elapsed_sec = start_wall_time.elapsed().as_secs_f32().max(0.001);
            let win_rate = (wins as f32 / episode as f32) * 100.0;
            let eps_per_sec = (episode as f32) / elapsed_sec;
            println!(
                "Ep {:3}/{} | WinRate: {:5.1}% (W:{} L:{} D:{}) | Reward: {:+6.1} | Eps: {:.3} | Speed: {:.1} ep/s",
                episode, total_episodes, win_rate, wins, losses, draws, ep_reward, engine.ai_state.rl_brain.epsilon, eps_per_sec
            );
        }
    }

    println!("------------------------------------------------------------");
    println!("📊 TỔNG KẾT CHIẾN THUẬT BOT ĐÃ SỬ DỤNG TRONG TRAINING:");
    for a_idx in 0..11 {
        let act = BotStrategicAction::from_index(a_idx);
        println!("   - {:<32}: {:6} decisions", act.name(), action_counts[a_idx]);
    }

    let model_json = serde_json::to_string_pretty(&engine.ai_state.rl_brain).expect("Serialize RL Brain failed");
    let mut file = File::create(&output_path).expect("Could not create model output file");
    file.write_all(model_json.as_bytes()).expect("Could not write model file");

    println!("✅ ĐÃ XUẤT MODEL RA FILE: {}", output_path);
    println!("⏱️  Tổng thời gian huấn luyện: {:.2}s", start_wall_time.elapsed().as_secs_f32());
    println!("============================================================");
}

fn count_combat_units(engine: &GameEngine, team: Team) -> usize {
    engine.units.iter().filter(|u| u.team == team && u.unit_type != UnitType::Engineer).count()
}

fn count_workers(engine: &GameEngine, team: Team) -> usize {
    engine.units.iter().filter(|u| u.team == team && u.unit_type == UnitType::Engineer).count()
}

fn count_ccs(engine: &GameEngine, team: Team) -> usize {
    engine.buildings.iter().filter(|b| b.team == team && b.building_type == BuildingType::CommandCenter).count()
}

fn count_buildings(engine: &GameEngine, team: Team) -> usize {
    engine.buildings.iter().filter(|b| b.team == team).count()
}
