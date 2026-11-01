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
    let mut total_episodes = 200;
    let mut time_scale_dt = 0.15_f32; // Tăng tốc thời gian mô phỏng (gấp ~3-4 lần dt thường)
    let max_game_time = 720.0_f32; // 12 phút trong game mỗi hiệp
    let mut output_path = "assets/rl_model.json".to_string();

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--episodes" | "-e" => {
                if i + 1 < args.len() {
                    total_episodes = args[i + 1].parse().unwrap_or(200);
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
    println!("🚀 BATTLEFIELD BLUEPRINT // HEADLESS RL BOT TRAINER");
    println!("   Episodes:       {}", total_episodes);
    println!("   Fast-Forward dt: {:.3}s/step", time_scale_dt);
    println!("   Max Game Time:  {:.0}s", max_game_time);
    println!("   Output Model:   {}", output_path);
    println!("============================================================");

    let start_wall_time = Instant::now();
    let mut wins = 0;
    let mut losses = 0;
    let mut draws = 0;
    let mut action_counts = [0usize; 10];

    // Khởi tạo engine mô phỏng
    let mut engine = GameEngine::new_headless();
    engine.ai_state.training_enabled = true;
    engine.ai_state.rl_brain.epsilon = 0.25; // Thử nghiệm khám phá

    for episode in 1..=total_episodes {
        engine.reset_game();
        engine.is_headless = true;
        engine.is_paused = false;
        engine.ai_state.training_enabled = true;

        // Bật sparring auto-play cho Player để huấn luyện Bot
        engine.autoplay.config.enabled = true;
        // Đa dạng hóa sparring profile cho Player qua từng ván
        match episode % 4 {
            0 => {
                // Rush Sparring Partner
                engine.autoplay.config.priority_techs = vec![];
            }
            1 => {
                // Science Tech Sparring Partner
                engine.autoplay.config.priority_techs = vec![
                    "focused_laser".to_string(),
                    "energy_shielding".to_string(),
                    "dark_matter_rockets".to_string(),
                ];
            }
            2 => {
                // Magic Tech Sparring Partner
                engine.autoplay.config.priority_techs = vec![
                    "celestial_flying_swords".to_string(),
                    "golden_bell_shield".to_string(),
                ];
            }
            _ => {
                // Monster & Eco Sparring Partner
                engine.autoplay.config.priority_techs = vec![
                    "optical_drills".to_string(),
                    "primeval_scales".to_string(),
                    "acid_banelings".to_string(),
                ];
            }
        }

        let mut ep_reward = 0.0_f32;
        let mut ep_steps = 0;

        while !engine.game_over && engine.game_time < max_game_time {
            let prev_state = extract_state_vector(&engine);
            let prev_combat_enemy = count_combat_units(&engine, Team::Enemy);
            let prev_combat_player = count_combat_units(&engine, Team::Player);
            let prev_p_cc = count_ccs(&engine, Team::Player);
            let prev_e_cc = count_ccs(&engine, Team::Enemy);

            // Bước chạy mô phỏng
            rust_engine::simulation::step(&mut engine, time_scale_dt);
            ep_steps += 1;

            let cur_state = extract_state_vector(&engine);
            let action = engine.ai_state.current_action;
            action_counts[action.to_index()] += 1;

            // Reward shaping
            let mut step_reward = 0.0_f32;

            // 1. Kinh tế & Thợ mỏ
            let worker_count = engine.units.iter().filter(|u| u.team == Team::Enemy && u.unit_type == UnitType::Engineer).count();
            step_reward += (worker_count as f32) * 0.005;

            // 2. Quân sự: Diệt lính Player
            let cur_combat_player = count_combat_units(&engine, Team::Player);
            if prev_combat_player > cur_combat_player {
                step_reward += (prev_combat_player - cur_combat_player) as f32 * 0.5;
            }

            // Mất lính Enemy
            let cur_combat_enemy = count_combat_units(&engine, Team::Enemy);
            if prev_combat_enemy > cur_combat_enemy {
                step_reward -= (prev_combat_enemy - cur_combat_enemy) as f32 * 0.3;
            }

            // 3. Phá hủy / Bảo vệ Nhà chính
            let cur_p_cc = count_ccs(&engine, Team::Player);
            if prev_p_cc > cur_p_cc {
                step_reward += 20.0;
            }
            let cur_e_cc = count_ccs(&engine, Team::Enemy);
            if prev_e_cc > cur_e_cc {
                step_reward -= 25.0;
            }

            // 4. Kết thúc ván
            let is_done = engine.game_over || engine.game_time >= max_game_time;
            if engine.game_over {
                let p_alive = engine.buildings.iter().any(|b| b.team == Team::Player && b.building_type == BuildingType::CommandCenter);
                if !p_alive {
                    step_reward += 100.0;
                } else {
                    step_reward -= 100.0;
                }
            }

            ep_reward += step_reward;

            // Học tăng cường (Q-Learning step)
            engine.ai_state.rl_brain.train_step(&prev_state, action, step_reward, &cur_state, is_done);

            if is_done {
                break;
            }
        }

        // Kết quả ván
        let p_cc_count = count_ccs(&engine, Team::Player);
        let e_cc_count = count_ccs(&engine, Team::Enemy);
        if e_cc_count > 0 && p_cc_count == 0 {
            wins += 1;
        } else if p_cc_count > 0 && e_cc_count == 0 {
            losses += 1;
        } else {
            draws += 1;
        }

        // Decay epsilon
        engine.ai_state.rl_brain.epsilon = (engine.ai_state.rl_brain.epsilon * 0.992).max(0.03);

        if episode % 25 == 0 || episode == total_episodes {
            let elapsed_sec = start_wall_time.elapsed().as_secs_f32().max(0.001);
            let win_rate = (wins as f32 / episode as f32) * 100.0;
            let eps_per_sec = (episode as f32) / elapsed_sec;
            println!(
                "Ep {:3}/{} | WinRate: {:5.1}% (W:{} L:{} D:{}) | Reward: {:+6.1} | Eps: {:.2} | Speed: {:.1} ep/s",
                episode, total_episodes, win_rate, wins, losses, draws, ep_reward, engine.ai_state.rl_brain.epsilon, eps_per_sec
            );
        }
    }

    println!("------------------------------------------------------------");
    println!("📊 TỔNG KẾT CHIẾN THUẬT BOT ĐÃ SỬ DỤNG TRONG TRAINING:");
    for a_idx in 0..10 {
        let act = BotStrategicAction::from_index(a_idx);
        println!("   - {:<30}: {:6} decisions", act.name(), action_counts[a_idx]);
    }

    // Lưu model JSON
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

fn count_ccs(engine: &GameEngine, team: Team) -> usize {
    engine.buildings.iter().filter(|b| b.team == team && b.building_type == BuildingType::CommandCenter).count()
}
