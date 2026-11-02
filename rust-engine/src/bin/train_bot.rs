use rust_engine::ai::extract_state_vector_for_team;
use rust_engine::autoplay::{
    adapt_training_autoplay_mid_match, create_training_autoplay_config, AutoTrainingArchetype,
};
use rust_engine::components::{BuildingType, FormationType, PlayerSlotConfig, Team, UnitType, MAX_PLAYERS};
use rust_engine::cuda_accel::{CudaDeviceManager, GpuBatchTrainer, TransitionSample};
use rust_engine::rl::{AiGameMode, BotStrategicAction, RlBrain, ACTION_DIM, FORMATION_DIM, STATE_DIM, DEFAULT_HIDDEN_SIZE, DEFAULT_HIDDEN2_SIZE};
use rust_engine::GameEngine;
use std::env;
use std::fs::File;
use std::io::Write;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Scenario {
    Duel1v1,       // 2 players: 1 vs 1
    Team2v2,       // 4 players: 2 vs 2
    Team3v3,       // 6 players: 3 vs 3
    Team4v4,       // 8 players: 4 vs 4
    ThreeWay2v2v2, // 6 players: 3 factions (2 vs 2 vs 2)
    FreeForAll8,   // 8 players: 8 factions (FFA)
}

impl Scenario {
    fn to_mode(&self) -> AiGameMode {
        match self {
            Scenario::Duel1v1 => AiGameMode::Duel1v1,
            Scenario::Team2v2 => AiGameMode::Team2v2,
            Scenario::Team3v3 => AiGameMode::Team3v3,
            Scenario::Team4v4 => AiGameMode::Team4v4,
            Scenario::ThreeWay2v2v2 => AiGameMode::ThreeWay2v2v2,
            Scenario::FreeForAll8 => AiGameMode::FreeForAll8,
        }
    }

    fn name(&self) -> &'static str {
        self.to_mode().name()
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
                slots[0] = PlayerSlotConfig { slot_index: 0, active: true, faction: 0, is_human: false };
                slots[2] = PlayerSlotConfig { slot_index: 2, active: true, faction: 0, is_human: false };
                slots[1] = PlayerSlotConfig { slot_index: 1, active: true, faction: 1, is_human: false };
                slots[3] = PlayerSlotConfig { slot_index: 3, active: true, faction: 1, is_human: false };
                for i in 4..8 {
                    slots[i] = PlayerSlotConfig { slot_index: i, active: false, faction: i as u8, is_human: false };
                }
            }
            Scenario::Team3v3 => {
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
                for i in 0..8 {
                    let fac = (i % 2) as u8;
                    slots[i] = PlayerSlotConfig { slot_index: i, active: true, faction: fac, is_human: false };
                }
            }
            Scenario::ThreeWay2v2v2 => {
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

struct TrainConfig {
    episodes_per_mode: usize,
    time_scale_dt: f32,
    max_game_time: f32,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut total_episodes = 60;
    let mut time_scale_dt = 0.25_f32; // Tăng tốc bước nhảy mô phỏng mặc định lên 0.25s
    let mut max_game_time = 1800.0_f32; // Tăng max game time lên 1800s (30 phút game)
    let mut specific_mode: Option<String> = None;
    let mut parallel_modes = true;

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
                    time_scale_dt = args[i + 1].parse().unwrap_or(0.25);
                    i += 1;
                }
            }
            "--max-time" | "--max_time" => {
                if i + 1 < args.len() {
                    max_game_time = args[i + 1].parse().unwrap_or(1800.0);
                    i += 1;
                }
            }
            "--mode" | "-m" => {
                if i + 1 < args.len() {
                    specific_mode = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            "--sequential" | "--no-parallel" => {
                parallel_modes = false;
            }
            _ => {
                if let Ok(ep) = args[i].parse::<usize>() {
                    total_episodes = ep;
                }
            }
        }
        i += 1;
    }

    let all_scenarios = [
        Scenario::Duel1v1,
        Scenario::Team2v2,
        Scenario::Team3v3,
        Scenario::Team4v4,
        Scenario::ThreeWay2v2v2,
        Scenario::FreeForAll8,
    ];

    let target_scenarios: Vec<Scenario> = if let Some(ref m) = specific_mode {
        all_scenarios
            .into_iter()
            .filter(|s| match m.to_lowercase().as_str() {
                "1v1" | "duel" => *s == Scenario::Duel1v1,
                "2v2" => *s == Scenario::Team2v2,
                "3v3" => *s == Scenario::Team3v3,
                "4v4" => *s == Scenario::Team4v4,
                "2v2v2" | "3way" => *s == Scenario::ThreeWay2v2v2,
                "ffa" | "8ffa" | "ffa8" => *s == Scenario::FreeForAll8,
                _ => true,
            })
            .collect()
    } else {
        all_scenarios.to_vec()
    };

    let gpu_avail = CudaDeviceManager::is_gpu_available();
    let gpu_name = CudaDeviceManager::device_name();
    let dummy_brain = RlBrain::default();
    let total_params = dummy_brain.total_parameters();

    println!("============================================================");
    println!("🧠 BATTLEFIELD BLUEPRINT // PARALLEL MULTI-MODE RL TRAINER");
    println!("   Hardware Device:    {}", if gpu_avail { format!("🚀 GPU ACCELERATED // {}", gpu_name) } else { "💻 CPU Fallback".to_string() });
    println!("   Neural Architecture:Deep Dual-Head MLP ({} -> {} -> {} -> {} + {})", STATE_DIM, DEFAULT_HIDDEN_SIZE, DEFAULT_HIDDEN2_SIZE, ACTION_DIM, FORMATION_DIM);
    println!("   Hidden Neurons:     {} neurons (Layer 1: {}, Layer 2: {})", DEFAULT_HIDDEN_SIZE + DEFAULT_HIDDEN2_SIZE, DEFAULT_HIDDEN_SIZE, DEFAULT_HIDDEN2_SIZE);
    println!("   Total Parameters:   {} trainable weights & biases", total_params);
    println!("   Episodes per Mode:  {}", total_episodes);
    println!("   Fast-Forward dt:    {:.3}s/step (High-Speed Tick)", time_scale_dt);
    println!("   Max Game Time:      {:.0}s (Expanded Endgame Window)", max_game_time);
    println!("   State Dimension:    {} inputs (Deep Economy & Resource Flow)", STATE_DIM);
    println!("   Action Dimension:   {} strategic actions", ACTION_DIM);
    println!("   Formation Dimension:{} formations", FORMATION_DIM);
    println!("   Parallel Branches:  {}", if parallel_modes { "ENABLED (Multi-threaded)" } else { "SEQUENTIAL" });
    println!("   Target Modes:       {:?}", target_scenarios.iter().map(|s| s.name()).collect::<Vec<_>>());
    println!("============================================================");

    let start_wall_time = Instant::now();
    let config = Arc::new(TrainConfig {
        episodes_per_mode: total_episodes,
        time_scale_dt,
        max_game_time,
    });

    let completed_counter = Arc::new(AtomicUsize::new(0));

    if parallel_modes && target_scenarios.len() > 1 {
        std::thread::scope(|s| {
            for &scenario in &target_scenarios {
                let cfg = Arc::clone(&config);
                let counter = Arc::clone(&completed_counter);
                s.spawn(move || {
                    let brain = train_mode(scenario, &cfg);
                    save_mode_model(scenario, &brain);
                    counter.fetch_add(1, Ordering::SeqCst);
                });
            }
        });
    } else {
        for &scenario in &target_scenarios {
            let brain = train_mode(scenario, &config);
            save_mode_model(scenario, &brain);
            completed_counter.fetch_add(1, Ordering::SeqCst);
        }
    }

    // Sao chép model 4v4 hoặc model đầu tiên làm fallback rl_model.json
    let default_source = "assets/rl_model_4v4.json";
    let default_dest = "assets/rl_model.json";
    if std::path::Path::new(default_source).exists() {
        let _ = std::fs::copy(default_source, default_dest);
        let _ = std::fs::copy(default_source, "rust-engine/assets/rl_model.json");
    }

    println!("============================================================");
    println!("🎉 HOÀN THÀNH HUẤN LUYỆN TOÀN BỘ {} CHẾ ĐỘ CHƠI!", completed_counter.load(Ordering::SeqCst));
    println!("⏱️  Tổng thời gian: {:.2}s", start_wall_time.elapsed().as_secs_f32());
    println!("============================================================");
}

fn train_mode(scenario: Scenario, config: &TrainConfig) -> RlBrain {
    let mode = scenario.to_mode();
    let total_episodes = config.episodes_per_mode;
    let time_scale_dt = config.time_scale_dt;
    let max_game_time = config.max_game_time;

    let mut champion_pool: Vec<RlBrain> = Vec::new();
    let mut total_wins = 0;
    let mut total_losses = 0;
    let mut total_draws = 0;

    let mut action_counts = [0usize; ACTION_DIM];
    let mut formation_counts = [0usize; FORMATION_DIM];

    let mut engine = GameEngine::new_headless();
    engine.ai_state.training_enabled = true;
    engine.ai_state.rl_brain.epsilon = 0.25;

    let learner_team = Team::Enemy; // Slot 1 là Agent chính

    let mut gpu_trainer = GpuBatchTrainer::new(32);

    let start_time = Instant::now();

    for episode in 1..=total_episodes {
        let slots = scenario.configure_slots();
        engine.player_slots = slots;

        engine.reset_game();
        engine.is_headless = true;
        engine.is_paused = false;
        engine.ai_state.training_enabled = true;

        // Chọn chiến thuật đối thủ
        let opponent_style = if champion_pool.is_empty() || episode % 4 != 0 {
            let arch_idx = (episode % 7) as usize;
            let arch = match arch_idx {
                0 => AutoTrainingArchetype::CombinedTitanArtilleryMage,
                1 => AutoTrainingArchetype::OmniTechGrandmaster,
                2 => AutoTrainingArchetype::ScienceArtillery,
                3 => AutoTrainingArchetype::MagicCelestialVoid,
                4 => AutoTrainingArchetype::MonsterSwarmBehemoth,
                5 => AutoTrainingArchetype::BioBlitzRush,
                _ => AutoTrainingArchetype::MacroFortress,
            };
            OpponentStyle::VsAuto(arch)
        } else {
            OpponentStyle::BotSelfPlay
        };

        // Cấu hình AI hoặc Autoplay cho tất cả các slot
        for slot_idx in 0..MAX_PLAYERS {
            let team = Team::from_index(slot_idx);
            if !engine.is_slot_active(team) {
                continue;
            }
            if team == learner_team {
                engine.team_autoplay_mut(team).config.enabled = false;
                continue;
            }

            if engine.are_hostile(learner_team, team) {
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
            let prev_opp_turrets = count_hostile_turrets(&engine, learner_team);
            let prev_opp_ccs = count_hostile_ccs(&engine, learner_team);
            let prev_my_ccs = count_ally_ccs(&engine, learner_team);
            let prev_my_reactors = count_ally_reactors(&engine, learner_team);
            let prev_my_converters = count_ally_converters(&engine, learner_team);
            let prev_my_unlocked = rust_engine::ai::count_unlocked_techs(engine.team_tech(learner_team));

            mid_adapt_timer += time_scale_dt;
            if mid_adapt_timer >= 2.0 {
                mid_adapt_timer = 0.0;
                for slot_idx in 0..MAX_PLAYERS {
                    let t = Team::from_index(slot_idx);
                    if engine.is_slot_active(t) && engine.team_autoplay(t).config.enabled {
                        adapt_training_autoplay_mid_match(&mut engine, t);
                    }
                }
            }

            rust_engine::simulation::step(&mut engine, time_scale_dt);

            let cur_state = extract_state_vector_for_team(&engine, learner_team);
            let action = engine.team_ai(learner_team).current_action;
            let formation = engine.team_ai(learner_team).current_formation;
            action_counts[action.to_index()] += 1;
            formation_counts[formation.to_index()] += 1;

            // --- REWARD CALCULATION ---
            let mut step_reward = 0.0_f32;

            // 1. Khắc chế AoE bằng Formation
            let opp_aoe_threat = prev_state[22];
            if opp_aoe_threat > 0.15 {
                if formation == FormationType::Scatter {
                    step_reward += 1.2;
                } else if formation == FormationType::Line || formation == FormationType::Box {
                    step_reward -= 0.6;
                }
            }

            // 2. Thưởng phối hợp đội hình theo hành động chiến lược
            match action {
                BotStrategicAction::AllOutAssault | BotStrategicAction::SiegeArtilleryPush => {
                    if formation == FormationType::Wedge || formation == FormationType::Line {
                        step_reward += 0.8;
                    }
                }
                BotStrategicAction::HarassEconomy | BotStrategicAction::FocusWeakestEnemy => {
                    if formation == FormationType::Column || formation == FormationType::Wedge {
                        step_reward += 0.9;
                    }
                }
                BotStrategicAction::DefendAndCounter | BotStrategicAction::CoordinateAllySupport => {
                    if formation == FormationType::Circle || formation == FormationType::Box {
                        step_reward += 0.7;
                    }
                }
                _ => {}
            }

            // 3. Phá hủy công trình thù địch & Tháp pháo
            let cur_opp_bldgs = count_hostile_buildings(&engine, learner_team);
            if prev_opp_bldgs > cur_opp_bldgs {
                step_reward += (prev_opp_bldgs - cur_opp_bldgs) as f32 * 25.0;
            }
            let cur_opp_turrets = count_hostile_turrets(&engine, learner_team);
            if prev_opp_turrets > cur_opp_turrets {
                step_reward += (prev_opp_turrets - cur_opp_turrets) as f32 * 35.0;
            }

            // 4. Diệt lính thù địch vs Mất lính phe mình
            let cur_opp_combat = count_hostile_combat(&engine, learner_team);
            if prev_opp_combat > cur_opp_combat {
                step_reward += (prev_opp_combat - cur_opp_combat) as f32 * 3.5;
            }
            let cur_my_combat = count_ally_combat(&engine, learner_team);
            if prev_my_combat > cur_my_combat {
                step_reward -= (prev_my_combat - cur_my_combat) as f32 * 1.5;
            }

            // 5. Phá hủy / Mất nhà chính
            let cur_opp_ccs = count_hostile_ccs(&engine, learner_team);
            if prev_opp_ccs > cur_opp_ccs {
                step_reward += (prev_opp_ccs - cur_opp_ccs) as f32 * 150.0;
            }
            let cur_my_ccs = count_ally_ccs(&engine, learner_team);
            if prev_my_ccs > cur_my_ccs {
                step_reward -= (prev_my_ccs - cur_my_ccs) as f32 * 150.0;
            }

            // 6. Thưởng tiến độ công nghệ
            let cur_my_unlocked = rust_engine::ai::count_unlocked_techs(engine.team_tech(learner_team));
            if cur_my_unlocked > prev_my_unlocked {
                step_reward += (cur_my_unlocked - prev_my_unlocked) as f32 * 16.0;
                if cur_my_unlocked == 19 {
                    step_reward += 60.0;
                }
            }

            // 7. Thưởng duy trì cơ cấu binh chủng đa dạng (Combined Arms)
            let count_my_u = |ut: UnitType| -> usize {
                engine.units.iter().filter(|u| u.team == learner_team && u.unit_type == ut).count()
            };
            let has_frontline = count_my_u(UnitType::BehemothTitan) > 0 || count_my_u(UnitType::Vanguard) > 0;
            let has_artillery = count_my_u(UnitType::DarkRocketeer) > 0 || count_my_u(UnitType::LaserMarksman) > 0;
            let has_magic = count_my_u(UnitType::VoidWarlock) > 0 || count_my_u(UnitType::TaoistSwordmaster) > 0;
            if has_frontline && has_artillery && has_magic {
                step_reward += 0.35;
            }

            // --- 8. PHẦN THƯỞNG KHUYẾN KHÍCH FARM TÀI NGUYÊN (POWER REACTOR & ENERGY CONVERTER) ---
            let cur_my_reactors = count_ally_reactors(&engine, learner_team);
            if cur_my_reactors > prev_my_reactors {
                let diff = (cur_my_reactors - prev_my_reactors) as f32;
                step_reward += diff * 12.0; // Thưởng ngay 12 điểm cho mỗi PowerReactor mới
            }

            let cur_my_converters = count_ally_converters(&engine, learner_team);
            if cur_my_converters > prev_my_converters {
                let diff = (cur_my_converters - prev_my_converters) as f32;
                step_reward += diff * 20.0; // Thưởng lớn 20 điểm cho mỗi EnergyConverter mới
            }

            // Thưởng theo lưu lượng sản sinh năng lượng
            let my_eco = engine.team_eco(learner_team);
            let energy_rate = my_eco.energy_per_sec;
            if energy_rate > 10.0 {
                step_reward += (energy_rate / 60.0).clamp(0.0, 1.5) * 0.4;
            }

            // Thưởng đặc biệt khi đạt "chuỗi chuyển hóa năng lượng thặng dư thành khoáng sản"
            // Khi Năng lượng vượt ngưỡng 1000.0 và có EnergyConverter hoạt động
            if my_eco.energy > rust_engine::config::ENERGY_CONVERSION_THRESHOLD && cur_my_converters > 0 {
                step_reward += 0.65; // Thưởng liên tục khi bộ máy chuyển hóa điện -> vàng hoạt động!
                if cur_my_reactors >= 4 {
                    step_reward += 0.35;
                }
            }

            // Thưởng khi bot AI ra quyết định phát triển hạ tầng kinh tế (ExpandEco / BuildInfrastructure)
            if matches!(action, BotStrategicAction::ExpandEco | BotStrategicAction::BuildInfrastructure) {
                if cur_my_reactors < 16 || cur_my_converters < 6 {
                    step_reward += 0.5;
                }
            }

            // Phạt nhẹ thời gian trôi
            step_reward -= 0.01;

            let is_done = engine.game_over || engine.game_time >= max_game_time;
            if engine.game_over {
                let any_hostile_alive = cur_opp_ccs > 0;
                let any_ally_alive = cur_my_ccs > 0;

                if !any_hostile_alive && any_ally_alive {
                    step_reward += 500.0 + (max_game_time - engine.game_time) * 0.2;
                } else if !any_ally_alive {
                    step_reward -= 300.0;
                }
            } else if engine.game_time >= max_game_time {
                step_reward -= 60.0;
            }

            ep_reward += step_reward;

            // Đẩy transition vào GPU Batch Trainer
            gpu_trainer.push(TransitionSample {
                state: prev_state.clone(),
                action: action.to_index(),
                formation: formation.to_index(),
                reward: step_reward,
                next_state: cur_state.clone(),
                done: is_done,
            });

            if gpu_trainer.can_train() {
                let brain = &mut engine.team_ai_mut(learner_team).rl_brain;
                gpu_trainer.train_batch(brain);
            } else {
                engine.team_ai_mut(learner_team).rl_brain.train_step_with_formation(
                    &prev_state,
                    action,
                    formation,
                    step_reward,
                    &cur_state,
                    is_done,
                );
            }

            // Multi-Agent Sharing: Các slot AI khác phe đối địch nếu có RL brain cũng học từ kinh nghiệm đối kháng
            for slot_idx in 0..MAX_PLAYERS {
                let other_team = Team::from_index(slot_idx);
                if engine.is_slot_active(other_team) && other_team != learner_team && !engine.team_autoplay(other_team).config.enabled {
                    let other_prev = extract_state_vector_for_team(&engine, other_team);
                    let other_act = engine.team_ai(other_team).current_action;
                    let other_form = engine.team_ai(other_team).current_formation;
                    let other_reward = -step_reward * 0.8; // Zero-sum inverted reward
                    engine.team_ai_mut(other_team).rl_brain.train_step_with_formation(
                        &other_prev,
                        other_act,
                        other_form,
                        other_reward,
                        &cur_state,
                        is_done,
                    );
                }
            }

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

        if episode <= 4 || episode % 10 == 0 || episode == total_episodes {
            let elapsed_sec = start_time.elapsed().as_secs_f32().max(0.001);
            let win_rate = (total_wins as f32 / episode as f32) * 100.0;
            let eps_per_sec = (episode as f32) / elapsed_sec;

            println!(
                "[{:<18}] Ep {:3}/{} | Win: {:5.1}% (W:{} L:{} D:{}) | Champs: {} | Rew: {:+6.1} | Eps: {:.3} | Speed: {:.1} ep/s",
                mode.name(),
                episode,
                total_episodes,
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

    engine.team_ai(learner_team).rl_brain.clone()
}

fn save_mode_model(scenario: Scenario, brain: &RlBrain) {
    let mode = scenario.to_mode();
    let filename = mode.model_filename();

    let model_json = serde_json::to_string_pretty(brain).expect("Serialize RL Brain failed");

    for dir in &["assets", "rust-engine/assets", "../assets"] {
        if std::path::Path::new(dir).exists() {
            let path = format!("{}/{}", dir, filename);
            if let Ok(mut file) = File::create(&path) {
                let _ = file.write_all(model_json.as_bytes());
                println!("💾 [{}] Đã lưu model: {}", mode.name(), path);
            }
        }
    }
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

fn count_ally_reactors(engine: &GameEngine, my_team: Team) -> usize {
    engine.buildings.iter().filter(|b| engine.are_allies(my_team, b.team) && b.building_type == BuildingType::PowerReactor).count()
}

fn count_ally_converters(engine: &GameEngine, my_team: Team) -> usize {
    engine.buildings.iter().filter(|b| engine.are_allies(my_team, b.team) && b.building_type == BuildingType::EnergyConverter).count()
}

fn count_hostile_buildings(engine: &GameEngine, my_team: Team) -> usize {
    engine.buildings.iter().filter(|b| engine.are_hostile(my_team, b.team)).count()
}

fn count_hostile_turrets(engine: &GameEngine, my_team: Team) -> usize {
    engine.buildings.iter().filter(|b| engine.are_hostile(my_team, b.team) && b.building_type == BuildingType::TurretTower).count()
}
