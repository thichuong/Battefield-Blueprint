use rust_engine::ai::{extract_state_vector, update_ai_for_team};
use rust_engine::autoplay::{
    adapt_training_autoplay_mid_match, create_training_autoplay_config, AutoTrainingArchetype,
};
use rust_engine::components::{BuildingType, Team, UnitType};
use rust_engine::rl::{BotStrategicAction, RlBrain};
use rust_engine::GameEngine;
use std::env;
use std::fs::File;
use std::io::Write;
use std::time::Instant;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MatchMode {
    VsAuto(AutoTrainingArchetype),
    BotSelfPlay,
}

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
    println!("🧠 BATTLEFIELD BLUEPRINT // ADAPTIVE RL BOT & SELF-PLAY TRAINER");
    println!("   Episodes:           {}", total_episodes);
    println!("   Fast-Forward dt:    {:.3}s/step", time_scale_dt);
    println!("   Max Game Time:      {:.0}s", max_game_time);
    println!("   Output Model:       {}", output_path);
    println!("   Auto Archetypes:    5 Dynamic Branches + Mid-Match Adaptation");
    println!("   Schedule:           Unlock Self-Play on Auto Wins -> Interleaved Matches");
    println!("============================================================");

    let start_wall_time = Instant::now();
    let mut total_wins = 0;
    let mut total_losses = 0;
    let mut total_draws = 0;

    let mut auto_wins = 0;
    let mut auto_matches = 0;

    let mut selfplay_wins = 0;
    let mut selfplay_matches = 0;

    let mut self_play_unlocked = false;
    let mut champion_pool: Vec<RlBrain> = Vec::new();
    let mut action_counts = [0usize; 11];

    let mut engine = GameEngine::new_headless();
    engine.ai_state.training_enabled = true;
    engine.ai_state.rl_brain.epsilon = 0.20;

    for episode in 1..=total_episodes {
        engine.reset_game();
        engine.is_headless = true;
        engine.is_paused = false;
        engine.ai_state.training_enabled = true;

        // Xác định thể thức thi đấu vòng này (Xen kẽ nếu đã mở khóa Self-Play)
        let mode = if !self_play_unlocked {
            let arch_idx = (episode % 5) as usize;
            let arch = match arch_idx {
                0 => AutoTrainingArchetype::ScienceArtillery,
                1 => AutoTrainingArchetype::MagicCelestialVoid,
                2 => AutoTrainingArchetype::MonsterSwarmBehemoth,
                3 => AutoTrainingArchetype::BioBlitzRush,
                _ => AutoTrainingArchetype::MacroFortress,
            };
            MatchMode::VsAuto(arch)
        } else {
            // Đã mở khóa Self-Play: 2 trận Bot-vs-Bot xen kẽ 1 trận Adaptive Auto
            if episode % 3 != 0 {
                MatchMode::BotSelfPlay
            } else {
                let arch_idx = (episode % 5) as usize;
                let arch = match arch_idx {
                    0 => AutoTrainingArchetype::ScienceArtillery,
                    1 => AutoTrainingArchetype::MagicCelestialVoid,
                    2 => AutoTrainingArchetype::MonsterSwarmBehemoth,
                    3 => AutoTrainingArchetype::BioBlitzRush,
                    _ => AutoTrainingArchetype::MacroFortress,
                };
                MatchMode::VsAuto(arch)
            }
        };

        let mut player_bot_state = rust_engine::ai::AiState::default();

        match mode {
            MatchMode::VsAuto(arch) => {
                auto_matches += 1;
                engine.autoplay.config = create_training_autoplay_config(Team::Player, arch);
            }
            MatchMode::BotSelfPlay => {
                selfplay_matches += 1;
                engine.autoplay.config.enabled = false;

                // Lấy đối thủ từ Champion Pool (bản sao não bot mạnh qua các thế hệ)
                if let Some(champ) = champion_pool.last() {
                    player_bot_state.rl_brain = champ.clone();
                } else {
                    player_bot_state.rl_brain = engine.ai_state.rl_brain.clone();
                }
                player_bot_state.training_enabled = false;
                player_bot_state.rl_brain.epsilon = 0.05; // Đánh chuẩn xác để làm sparring partner
            }
        }

        let mut ep_reward = 0.0_f32;
        let mut last_print_t = 0.0_f32;
        let mut mid_adapt_timer = 0.0_f32;

        while !engine.game_over && engine.game_time < max_game_time {
            let prev_state = extract_state_vector(&engine);
            let prev_combat_enemy = count_combat_units(&engine, Team::Enemy);
            let prev_combat_player = count_combat_units(&engine, Team::Player);
            let prev_p_cc = count_ccs(&engine, Team::Player);
            let prev_e_cc = count_ccs(&engine, Team::Enemy);
            let prev_p_workers = count_workers(&engine, Team::Player);
            let prev_p_bldgs = count_buildings(&engine, Team::Player);

            // 1. Tùy biến giữa trận cho Auto (nếu đang ở chế độ VsAuto)
            if let MatchMode::VsAuto(_) = mode {
                mid_adapt_timer += time_scale_dt;
                if mid_adapt_timer >= 1.5 {
                    mid_adapt_timer = 0.0;
                    adapt_training_autoplay_mid_match(&mut engine, Team::Player);
                }
            }

            // 2. Chạy bước mô phỏng engine
            rust_engine::simulation::step(&mut engine, time_scale_dt);

            // 3. Nếu là Bot-vs-Bot: Cập nhật thêm AI cho Player Bot
            if let MatchMode::BotSelfPlay = mode {
                update_ai_for_team(&mut engine, Team::Player, &mut player_bot_state);
            }

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

            // --- REWARD SHAPING MẠNH MẼ CHO LEARNER BOT ---
            let mut step_reward = 0.0_f32;

            // 1. Khuyến khích diệt thợ mỏ và triệt tiêu kinh tế đối thủ
            let cur_p_workers = count_workers(&engine, Team::Player);
            if prev_p_workers > cur_p_workers {
                step_reward += (prev_p_workers - cur_p_workers) as f32 * 10.0;
            }

            // 2. Thưởng khi phá hủy công trình của đối thủ
            let cur_p_bldgs = count_buildings(&engine, Team::Player);
            if prev_p_bldgs > cur_p_bldgs {
                step_reward += (prev_p_bldgs - cur_p_bldgs) as f32 * 30.0;
            }

            // 3. Tiêu diệt lính chiến đấu đối thủ
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

            // 5. Phạt nhẹ thời gian để tránh Bot dây dưa
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
        let is_learner_win = e_cc_count > 0 && p_cc_count == 0;
        let is_learner_loss = p_cc_count > 0 && e_cc_count == 0;

        if is_learner_win {
            total_wins += 1;
            match mode {
                MatchMode::VsAuto(_) => {
                    auto_wins += 1;
                    // Sau khi thắng Auto, kích hoạt Self-Play và thêm vào Champion Pool
                    if !self_play_unlocked {
                        self_play_unlocked = true;
                        champion_pool.push(engine.ai_state.rl_brain.clone());
                        println!(
                            "   🔥 [MILESTONE] Bot đã thắng Auto chuyên sâu (Ep {})! MỞ KHÓA BOT SELF-PLAY VỚI VÒNG ĐẤU XEN KẼ!",
                            episode
                        );
                    }
                }
                MatchMode::BotSelfPlay => {
                    selfplay_wins += 1;
                    // Khi thắng đối thủ Self-Play, cập nhật não vào Champion Pool (tối đa giữ 5 đời vô địch)
                    champion_pool.push(engine.ai_state.rl_brain.clone());
                    if champion_pool.len() > 5 {
                        champion_pool.remove(0);
                    }
                }
            }
        } else if is_learner_loss {
            total_losses += 1;
        } else {
            total_draws += 1;
        }

        engine.ai_state.rl_brain.epsilon = (engine.ai_state.rl_brain.epsilon * 0.985).max(0.02);

        if episode <= 5 || episode % 20 == 0 || episode == total_episodes {
            let mode_str = match mode {
                MatchMode::VsAuto(arch) => format!("Auto: {}", arch.name()),
                MatchMode::BotSelfPlay => format!("Self-Play (Gen {})", champion_pool.len()),
            };
            let elapsed_sec = start_wall_time.elapsed().as_secs_f32().max(0.001);
            let win_rate = (total_wins as f32 / episode as f32) * 100.0;
            let eps_per_sec = (episode as f32) / elapsed_sec;

            println!(
                "Ep {:3}/{} [{:<30}] | Win: {:5.1}% (W:{} L:{} D:{}) | Auto: {}/{} | SelfPlay: {}/{} | Rew: {:+6.1} | Eps: {:.3} | Speed: {:.1} ep/s",
                episode,
                total_episodes,
                mode_str,
                win_rate,
                total_wins,
                total_losses,
                total_draws,
                auto_wins,
                auto_matches,
                selfplay_wins,
                selfplay_matches,
                ep_reward,
                engine.ai_state.rl_brain.epsilon,
                eps_per_sec
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
    if let Some(parent) = std::path::Path::new(&output_path).parent() {
        let _ = std::fs::create_dir_all(parent);
    }
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
