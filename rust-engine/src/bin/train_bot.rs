use rust_engine::ai::extract_state_vector_for_team;
use rust_engine::autoplay::{
    adapt_training_autoplay_mid_match, create_training_autoplay_config, AutoTrainingArchetype,
};
use rust_engine::components::{BuildingType, PlayerSlotConfig, Team, UnitType, MAX_PLAYERS};
use rust_engine::rl::{BotStrategicAction, RlBrain};
use rust_engine::GameEngine;
use std::env;
use std::fs::File;
use std::io::Write;
use std::time::Instant;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Scenario {
    Duel1v1,     // 2 players: 1 vs 1
    Team2v2,     // 4 players: 2 vs 2
    Team3v3,     // 6 players: 3 vs 3
    Team4v4,     // 8 players: 4 vs 4
    ThreeWay2v2v2, // 6 players: 3 factions (2 vs 2 vs 2)
    FreeForAll8, // 8 players: 8 factions (FFA)
}

impl Scenario {
    fn name(&self) -> &'static str {
        match self {
            Scenario::Duel1v1 => "1v1 Duel",
            Scenario::Team2v2 => "2v2 Battle",
            Scenario::Team3v3 => "3v3 War",
            Scenario::Team4v4 => "4v4 Grand War",
            Scenario::ThreeWay2v2v2 => "2v2v2 3-Way",
            Scenario::FreeForAll8 => "8-FFA Battle Royale",
        }
    }

    fn configure_slots(&self) -> [PlayerSlotConfig; MAX_PLAYERS] {
        let mut slots = [PlayerSlotConfig::default(); MAX_PLAYERS];
        match self {
            Scenario::Duel1v1 => {
                slots[0] = PlayerSlotConfig { slot_index: 0, active: true, faction: 0, is_human: false };
                slots[1] = PlayerSlotConfig { slot_index: 1, active: true, faction: 1, is_human: false };
                for i in 2..8 {
                    slots[i] = PlayerSlotConfig { slot_index: i, active: false, faction: i as u8, is_human: false };
                }
            }
            Scenario::Team2v2 => {
                // Faction 0: Slot 0, 2 | Faction 1: Slot 1, 3
                slots[0] = PlayerSlotConfig { slot_index: 0, active: true, faction: 0, is_human: false };
                slots[2] = PlayerSlotConfig { slot_index: 2, active: true, faction: 0, is_human: false };
                slots[1] = PlayerSlotConfig { slot_index: 1, active: true, faction: 1, is_human: false };
                slots[3] = PlayerSlotConfig { slot_index: 3, active: true, faction: 1, is_human: false };
                for i in 4..8 {
                    slots[i] = PlayerSlotConfig { slot_index: i, active: false, faction: i as u8, is_human: false };
                }
            }
            Scenario::Team3v3 => {
                // Faction 0: 0, 2, 4 | Faction 1: 1, 3, 5
                slots[0] = PlayerSlotConfig { slot_index: 0, active: true, faction: 0, is_human: false };
                slots[2] = PlayerSlotConfig { slot_index: 2, active: true, faction: 0, is_human: false };
                slots[4] = PlayerSlotConfig { slot_index: 4, active: true, faction: 0, is_human: false };
                slots[1] = PlayerSlotConfig { slot_index: 1, active: true, faction: 1, is_human: false };
                slots[3] = PlayerSlotConfig { slot_index: 3, active: true, faction: 1, is_human: false };
                slots[5] = PlayerSlotConfig { slot_index: 5, active: true, faction: 1, is_human: false };
                slots[6] = PlayerSlotConfig { slot_index: 6, active: false, faction: 6, is_human: false };
                slots[7] = PlayerSlotConfig { slot_index: 7, active: false, faction: 7, is_human: false };
            }
            Scenario::Team4v4 => {
                // Faction 0: 0, 2, 4, 6 | Faction 1: 1, 3, 5, 7
                for i in 0..8 {
                    let fac = (i % 2) as u8;
                    slots[i] = PlayerSlotConfig { slot_index: i, active: true, faction: fac, is_human: false };
                }
            }
            Scenario::ThreeWay2v2v2 => {
                // 3 Factions: 0, 1, 2
                // Faction 0: 0, 3 | Faction 1: 1, 4 | Faction 2: 2, 5
                slots[0] = PlayerSlotConfig { slot_index: 0, active: true, faction: 0, is_human: false };
                slots[3] = PlayerSlotConfig { slot_index: 3, active: true, faction: 0, is_human: false };
                slots[1] = PlayerSlotConfig { slot_index: 1, active: true, faction: 1, is_human: false };
                slots[4] = PlayerSlotConfig { slot_index: 4, active: true, faction: 1, is_human: false };
                slots[2] = PlayerSlotConfig { slot_index: 2, active: true, faction: 2, is_human: false };
                slots[5] = PlayerSlotConfig { slot_index: 5, active: true, faction: 2, is_human: false };
                slots[6] = PlayerSlotConfig { slot_index: 6, active: false, faction: 6, is_human: false };
                slots[7] = PlayerSlotConfig { slot_index: 7, active: false, faction: 7, is_human: false };
            }
            Scenario::FreeForAll8 => {
                // 8 Factions
                for i in 0..8 {
                    slots[i] = PlayerSlotConfig { slot_index: i, active: true, faction: i as u8, is_human: false };
                }
            }
        }
        slots
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum OpponentStyle {
    VsAuto(AutoTrainingArchetype),
    BotSelfPlay,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut total_episodes = 60;
    let mut time_scale_dt = 0.15_f32;
    let max_game_time = 500.0_f32;
    let mut output_path = "assets/rl_model.json".to_string();

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--episodes" | "-e" => {
                if i + 1 < args.len() {
                    total_episodes = args[i + 1].parse().unwrap_or(60);
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
            _ => {
                if let Ok(ep) = args[i].parse::<usize>() {
                    total_episodes = ep;
                }
            }
        }
        i += 1;
    }

    println!("============================================================");
    println!("🧠 BATTLEFIELD BLUEPRINT // MULTI-PLAYER & MULTI-FACTION RL TRAINER");
    println!("   Episodes:           {}", total_episodes);
    println!("   Fast-Forward dt:    {:.3}s/step", time_scale_dt);
    println!("   Max Game Time:      {:.0}s", max_game_time);
    println!("   Output Model:       {}", output_path);
    println!("   Scenarios:          1v1, 2v2, 3v3, 4v4, 2v2v2, 8-FFA");
    println!("   Map Size:           4800 x 4800 (Expanded 8-Slot Layout)");
    println!("============================================================");

    let start_wall_time = Instant::now();
    let mut total_wins = 0;
    let mut total_losses = 0;
    let mut total_draws = 0;

    let mut champion_pool: Vec<RlBrain> = Vec::new();
    let mut action_counts = [0usize; 11];

    let mut engine = GameEngine::new_headless();
    engine.ai_state.training_enabled = true;
    engine.ai_state.rl_brain.epsilon = 0.25;

    let scenarios = [
        Scenario::Duel1v1,
        Scenario::Team2v2,
        Scenario::Team3v3,
        Scenario::Team4v4,
        Scenario::ThreeWay2v2v2,
        Scenario::FreeForAll8,
    ];

    let learner_team = Team::Enemy; // Slot 1 là Agent chính học tập

    for episode in 1..=total_episodes {
        let scenario = scenarios[(episode - 1) % scenarios.len()];
        let slots = scenario.configure_slots();
        engine.player_slots = slots;

        engine.reset_game();
        engine.is_headless = true;
        engine.is_paused = false;
        engine.ai_state.training_enabled = true;

        // Chọn chiến thuật đối thủ (Auto Archetype hoặc Self-Play sparring)
        let opponent_style = if champion_pool.is_empty() || episode % 3 != 0 {
            let arch_idx = (episode % 5) as usize;
            let arch = match arch_idx {
                0 => AutoTrainingArchetype::ScienceArtillery,
                1 => AutoTrainingArchetype::MagicCelestialVoid,
                2 => AutoTrainingArchetype::MonsterSwarmBehemoth,
                3 => AutoTrainingArchetype::BioBlitzRush,
                _ => AutoTrainingArchetype::MacroFortress,
            };
            OpponentStyle::VsAuto(arch)
        } else {
            OpponentStyle::BotSelfPlay
        };

        // Cấu hình AI hoặc Autoplay cho các slot đối thủ / đồng minh
        for slot_idx in 0..MAX_PLAYERS {
            let team = Team::from_index(slot_idx);
            if !engine.is_slot_active(team) {
                continue;
            }
            if team == learner_team {
                // Learner Bot
                engine.team_autoplay_mut(team).config.enabled = false;
                continue;
            }

            if engine.are_hostile(learner_team, team) {
                // Kẻ địch
                match opponent_style {
                    OpponentStyle::VsAuto(arch) => {
                        let cfg = create_training_autoplay_config(team, arch);
                        engine.team_autoplay_mut(team).config = cfg;
                    }
                    OpponentStyle::BotSelfPlay => {
                        engine.team_autoplay_mut(team).config.enabled = false;
                        if let Some(champ) = champion_pool.last() {
                            engine.team_ai_mut(team).rl_brain = champ.clone();
                            engine.team_ai_mut(team).rl_brain.epsilon = 0.05;
                        }
                    }
                }
            } else {
                // Đồng minh: Cấu hình hỗ trợ phối hợp
                let cfg = create_training_autoplay_config(team, AutoTrainingArchetype::MacroFortress);
                engine.team_autoplay_mut(team).config = cfg;
            }
        }

        let mut ep_reward = 0.0_f32;
        let mut mid_adapt_timer = 0.0_f32;

        while !engine.game_over && engine.game_time < max_game_time {
            let prev_state = extract_state_vector_for_team(&engine, learner_team);
            let prev_opp_combat = count_hostile_combat(&engine, learner_team);
            let prev_my_combat = count_ally_combat(&engine, learner_team);
            let prev_opp_bldgs = count_hostile_buildings(&engine, learner_team);
            let prev_opp_ccs = count_hostile_ccs(&engine, learner_team);
            let prev_my_ccs = count_ally_ccs(&engine, learner_team);

            // Tùy biến thích ứng giữa trận cho Auto đối thủ
            if let OpponentStyle::VsAuto(_) = opponent_style {
                mid_adapt_timer += time_scale_dt;
                if mid_adapt_timer >= 2.0 {
                    mid_adapt_timer = 0.0;
                    for slot_idx in 0..MAX_PLAYERS {
                        let t = Team::from_index(slot_idx);
                        if engine.is_slot_active(t) && engine.are_hostile(learner_team, t) && engine.team_autoplay(t).config.enabled {
                            adapt_training_autoplay_mid_match(&mut engine, t);
                        }
                    }
                }
            }

            // Tiến hành mô phỏng 1 bước
            rust_engine::simulation::step(&mut engine, time_scale_dt);

            let cur_state = extract_state_vector_for_team(&engine, learner_team);
            let action = engine.team_ai(learner_team).current_action;
            action_counts[action.to_index()] += 1;

            // --- REWARD CALCULATION FOR MULTI-PLAYER/FACTION ---
            let mut step_reward = 0.0_f32;

            // 1. Phá hủy công trình thù địch
            let cur_opp_bldgs = count_hostile_buildings(&engine, learner_team);
            if prev_opp_bldgs > cur_opp_bldgs {
                step_reward += (prev_opp_bldgs - cur_opp_bldgs) as f32 * 25.0;
            }

            // 2. Diệt lính thù địch
            let cur_opp_combat = count_hostile_combat(&engine, learner_team);
            if prev_opp_combat > cur_opp_combat {
                step_reward += (prev_opp_combat - cur_opp_combat) as f32 * 3.5;
            }

            // 3. Phạt nếu phe mình/đồng minh mất lính
            let cur_my_combat = count_ally_combat(&engine, learner_team);
            if prev_my_combat > cur_my_combat {
                step_reward -= (prev_my_combat - cur_my_combat) as f32 * 1.5;
            }

            // 4. Phá hủy nhà chính đối địch vs Mất nhà chính phe mình
            let cur_opp_ccs = count_hostile_ccs(&engine, learner_team);
            if prev_opp_ccs > cur_opp_ccs {
                step_reward += (prev_opp_ccs - cur_opp_ccs) as f32 * 150.0;
            }
            let cur_my_ccs = count_ally_ccs(&engine, learner_team);
            if prev_my_ccs > cur_my_ccs {
                step_reward -= (prev_my_ccs - cur_my_ccs) as f32 * 150.0;
            }

            // Phạt nhẹ thời gian trôi
            step_reward -= 0.01;

            let is_done = engine.game_over || engine.game_time >= max_game_time;
            if engine.game_over {
                let any_hostile_alive = cur_opp_ccs > 0;
                let any_ally_alive = cur_my_ccs > 0;

                if !any_hostile_alive && any_ally_alive {
                    step_reward += 450.0 + (max_game_time - engine.game_time) * 0.2;
                } else if !any_ally_alive {
                    step_reward -= 250.0;
                }
            } else if engine.game_time >= max_game_time {
                step_reward -= 80.0;
            }

            ep_reward += step_reward;
            engine.team_ai_mut(learner_team).rl_brain.train_step(&prev_state, action, step_reward, &cur_state, is_done);

            if is_done {
                break;
            }
        }

        let final_opp_ccs = count_hostile_ccs(&engine, learner_team);
        let final_my_ccs = count_ally_ccs(&engine, learner_team);
        let is_learner_win = final_opp_ccs == 0 && final_my_ccs > 0;
        let is_learner_loss = final_my_ccs == 0 && final_opp_ccs > 0;

        if is_learner_win {
            total_wins += 1;
            // Thắng trận: Lưu snapshot não vào Champion Pool
            champion_pool.push(engine.team_ai(learner_team).rl_brain.clone());
            if champion_pool.len() > 6 {
                champion_pool.remove(0);
            }
        } else if is_learner_loss {
            total_losses += 1;
        } else {
            total_draws += 1;
        }

        // Decay epsilon
        let eps = &mut engine.team_ai_mut(learner_team).rl_brain.epsilon;
        *eps = (*eps * 0.985).max(0.02);
        let cur_eps = *eps;

        if episode <= 6 || episode % 10 == 0 || episode == total_episodes {
            let elapsed_sec = start_wall_time.elapsed().as_secs_f32().max(0.001);
            let win_rate = (total_wins as f32 / episode as f32) * 100.0;
            let eps_per_sec = (episode as f32) / elapsed_sec;

            println!(
                "Ep {:3}/{} [{:<18}] | Win: {:5.1}% (W:{} L:{} D:{}) | Champs: {} | Rew: {:+6.1} | Eps: {:.3} | Speed: {:.1} ep/s",
                episode,
                total_episodes,
                scenario.name(),
                win_rate,
                total_wins,
                total_losses,
                total_draws,
                champion_pool.len(),
                ep_reward,
                cur_eps,
                eps_per_sec
            );
        }
    }

    println!("------------------------------------------------------------");
    println!("📊 TỔNG KẾT CHIẾN THUẬT BOT ĐÃ SỬ DỤNG TRONG HUẤN LUYỆN ĐA PHE:");
    for a_idx in 0..11 {
        let act = BotStrategicAction::from_index(a_idx);
        println!("   - {:<32}: {:6} decisions", act.name(), action_counts[a_idx]);
    }

    let model_json = serde_json::to_string_pretty(&engine.team_ai(learner_team).rl_brain).expect("Serialize RL Brain failed");
    if let Some(parent) = std::path::Path::new(&output_path).parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let mut file = File::create(&output_path).expect("Could not create model output file");
    file.write_all(model_json.as_bytes()).expect("Could not write model file");

    println!("✅ ĐÃ XUẤT MODEL RA FILE: {}", output_path);

    // Tự động đồng bộ giữa thư mục gốc dự án và rust-engine/assets
    if output_path == "assets/rl_model.json" {
        let alt_engine_path = "rust-engine/assets/rl_model.json";
        if std::path::Path::new("rust-engine/assets").exists() {
            if let Ok(mut alt_file) = File::create(alt_engine_path) {
                let _ = alt_file.write_all(model_json.as_bytes());
                println!("🔄 Đã tự động đồng bộ vào: {}", alt_engine_path);
            }
        }
        let alt_parent_path = "../assets/rl_model.json";
        if std::path::Path::new("../assets").exists() {
            if let Ok(mut alt_file) = File::create(alt_parent_path) {
                let _ = alt_file.write_all(model_json.as_bytes());
                println!("🔄 Đã tự động đồng bộ vào: {}", alt_parent_path);
            }
        }
    }
    println!("⏱️  Tổng thời gian huấn luyện: {:.2}s", start_wall_time.elapsed().as_secs_f32());
    println!("============================================================");
}

fn count_hostile_combat(engine: &GameEngine, my_team: Team) -> usize {
    engine.units.iter().filter(|u| engine.are_hostile(my_team, u.team) && u.unit_type != UnitType::Engineer).count()
}

fn count_ally_combat(engine: &GameEngine, my_team: Team) -> usize {
    engine.units.iter().filter(|u| engine.are_allies(my_team, u.team) && u.unit_type != UnitType::Engineer).count()
}

fn count_hostile_ccs(engine: &GameEngine, my_team: Team) -> usize {
    engine.buildings.iter().filter(|b| engine.are_hostile(my_team, b.team) && b.building_type == BuildingType::CommandCenter).count()
}

fn count_ally_ccs(engine: &GameEngine, my_team: Team) -> usize {
    engine.buildings.iter().filter(|b| engine.are_allies(my_team, b.team) && b.building_type == BuildingType::CommandCenter).count()
}

fn count_hostile_buildings(engine: &GameEngine, my_team: Team) -> usize {
    engine.buildings.iter().filter(|b| engine.are_hostile(my_team, b.team)).count()
}
