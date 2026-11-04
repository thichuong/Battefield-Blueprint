use rust_engine::ai::{compute_ai_step_reward, extract_state_vector_for_team, RewardSnapshot};
use rust_engine::autoplay::{
    adapt_training_autoplay_mid_match, create_training_autoplay_config, AutoTrainingArchetype,
};
use rust_engine::components::{BuildingType, PlayerSlotConfig, Team, UnitType, MAX_PLAYERS};
use rust_engine::cuda_accel::{CudaDeviceManager, PersistentGpuTrainer, TransitionSample};
use rust_engine::rl::{
    AiGameMode, RlBrain, ACTION_DIM, DEFAULT_HIDDEN2_SIZE, DEFAULT_HIDDEN_SIZE, FORMATION_DIM,
    STATE_DIM,
};
use rust_engine::GameEngine;
use std::env;
use std::fs::File;
use std::io::Write;
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
    fn to_mode(self) -> AiGameMode {
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
                slots[0] = PlayerSlotConfig {
                    slot_index: 0,
                    active: true,
                    faction: 0,
                    is_human: false,
                };
                slots[1] = PlayerSlotConfig {
                    slot_index: 1,
                    active: true,
                    faction: 1,
                    is_human: false,
                };
                for (i, slot) in slots.iter_mut().enumerate().skip(2) {
                    *slot = PlayerSlotConfig {
                        slot_index: i,
                        active: false,
                        faction: i as u8,
                        is_human: false,
                    };
                }
            }
            Scenario::Team2v2 => {
                slots[0] = PlayerSlotConfig {
                    slot_index: 0,
                    active: true,
                    faction: 0,
                    is_human: false,
                };
                slots[2] = PlayerSlotConfig {
                    slot_index: 2,
                    active: true,
                    faction: 0,
                    is_human: false,
                };
                slots[1] = PlayerSlotConfig {
                    slot_index: 1,
                    active: true,
                    faction: 1,
                    is_human: false,
                };
                slots[3] = PlayerSlotConfig {
                    slot_index: 3,
                    active: true,
                    faction: 1,
                    is_human: false,
                };
                for (i, slot) in slots.iter_mut().enumerate().skip(4) {
                    *slot = PlayerSlotConfig {
                        slot_index: i,
                        active: false,
                        faction: i as u8,
                        is_human: false,
                    };
                }
            }
            Scenario::Team3v3 => {
                slots[0] = PlayerSlotConfig {
                    slot_index: 0,
                    active: true,
                    faction: 0,
                    is_human: false,
                };
                slots[2] = PlayerSlotConfig {
                    slot_index: 2,
                    active: true,
                    faction: 0,
                    is_human: false,
                };
                slots[4] = PlayerSlotConfig {
                    slot_index: 4,
                    active: true,
                    faction: 0,
                    is_human: false,
                };
                slots[1] = PlayerSlotConfig {
                    slot_index: 1,
                    active: true,
                    faction: 1,
                    is_human: false,
                };
                slots[3] = PlayerSlotConfig {
                    slot_index: 3,
                    active: true,
                    faction: 1,
                    is_human: false,
                };
                slots[5] = PlayerSlotConfig {
                    slot_index: 5,
                    active: true,
                    faction: 1,
                    is_human: false,
                };
                slots[6] = PlayerSlotConfig {
                    slot_index: 6,
                    active: false,
                    faction: 6,
                    is_human: false,
                };
                slots[7] = PlayerSlotConfig {
                    slot_index: 7,
                    active: false,
                    faction: 7,
                    is_human: false,
                };
            }
            Scenario::Team4v4 => {
                for (i, slot) in slots.iter_mut().enumerate() {
                    let fac = (i % 2) as u8;
                    *slot = PlayerSlotConfig {
                        slot_index: i,
                        active: true,
                        faction: fac,
                        is_human: false,
                    };
                }
            }
            Scenario::ThreeWay2v2v2 => {
                slots[0] = PlayerSlotConfig {
                    slot_index: 0,
                    active: true,
                    faction: 0,
                    is_human: false,
                };
                slots[3] = PlayerSlotConfig {
                    slot_index: 3,
                    active: true,
                    faction: 0,
                    is_human: false,
                };
                slots[1] = PlayerSlotConfig {
                    slot_index: 1,
                    active: true,
                    faction: 1,
                    is_human: false,
                };
                slots[4] = PlayerSlotConfig {
                    slot_index: 4,
                    active: true,
                    faction: 1,
                    is_human: false,
                };
                slots[2] = PlayerSlotConfig {
                    slot_index: 2,
                    active: true,
                    faction: 2,
                    is_human: false,
                };
                slots[5] = PlayerSlotConfig {
                    slot_index: 5,
                    active: true,
                    faction: 2,
                    is_human: false,
                };
                slots[6] = PlayerSlotConfig {
                    slot_index: 6,
                    active: false,
                    faction: 6,
                    is_human: false,
                };
                slots[7] = PlayerSlotConfig {
                    slot_index: 7,
                    active: false,
                    faction: 7,
                    is_human: false,
                };
            }
            Scenario::FreeForAll8 => {
                for (i, slot) in slots.iter_mut().enumerate() {
                    *slot = PlayerSlotConfig {
                        slot_index: i,
                        active: true,
                        faction: i as u8,
                        is_human: false,
                    };
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

const ALL_AUTO_ARCHETYPES: [AutoTrainingArchetype; 7] = [
    AutoTrainingArchetype::ScienceArtillery,
    AutoTrainingArchetype::MagicCelestialVoid,
    AutoTrainingArchetype::MonsterSwarmBehemoth,
    AutoTrainingArchetype::BioBlitzRush,
    AutoTrainingArchetype::MacroFortress,
    AutoTrainingArchetype::CombinedTitanArtilleryMage,
    AutoTrainingArchetype::OmniTechGrandmaster,
];

struct TrainConfig {
    episodes_per_mode: usize,
    time_scale_dt: f32,
    max_game_time: f32,
    num_threads: usize,
    drive_backup_dir: Option<String>,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let total_cpus = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(8);
    let default_threads = (total_cpus / 4).max(1);
    let mut total_episodes = 128;
    let mut time_scale_dt = 0.25_f32; // Bước nhảy mô phỏng mặc định 0.25s
    let mut max_game_time = 1800.0_f32; // Max game time 1800s (30 phút game)
    let mut num_threads = default_threads;
    let mut resume_training = true; // Mặc định tiếp tục từ model kết quả trước đó
    let mut specific_mode: Option<String> = None;
    let mut drive_backup_dir: Option<String> = env::var("DRIVE_BACKUP_DIR")
        .ok()
        .or_else(|| env::var("GOOGLE_DRIVE_DIR").ok());

    let mut i = 1;
    let mut positional_idx = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--episodes" | "-e" => {
                if i + 1 < args.len() {
                    total_episodes = args[i + 1].parse().unwrap_or(128);
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
            "--threads" | "-t" => {
                if i + 1 < args.len() {
                    let val = &args[i + 1];
                    if val == "null" || val.is_empty() {
                        num_threads = default_threads;
                    } else {
                        num_threads = val.parse().unwrap_or(default_threads).max(1);
                    }
                    i += 1;
                }
            }
            "--resume" | "-r" => {
                if i + 1 < args.len() && (args[i + 1] == "true" || args[i + 1] == "false") {
                    resume_training = args[i + 1] == "true";
                    i += 1;
                } else {
                    resume_training = true;
                }
            }
            "--scratch" | "--from-scratch" | "--fresh" => {
                resume_training = false;
            }
            "--drive-backup-dir" | "--drive-dir" | "--backup-dir" => {
                if i + 1 < args.len() {
                    drive_backup_dir = Some(args[i + 1].clone());
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
                num_threads = 1;
            }
            arg if !arg.starts_with('-') => {
                // Hỗ trợ truyền theo thứ tự tham số: [EPISODES] [DT] [MAX_TIME] [THREADS] [RESUME] [DRIVE_DIR]
                match positional_idx {
                    0 => {
                        if let Ok(ep) = arg.parse::<usize>() {
                            total_episodes = ep;
                        }
                    }
                    1 => {
                        if let Ok(dt) = arg.parse::<f32>() {
                            time_scale_dt = dt;
                        }
                    }
                    2 => {
                        if let Ok(mt) = arg.parse::<f32>() {
                            max_game_time = mt;
                        }
                    }
                    3 => {
                        if arg == "null" || arg.is_empty() {
                            num_threads = default_threads;
                        } else if let Ok(th) = arg.parse::<usize>() {
                            num_threads = th.max(1);
                        }
                    }
                    4 => {
                        resume_training = !matches!(arg, "false" | "0" | "scratch" | "no");
                    }
                    5 => {
                        drive_backup_dir = Some(arg.to_string());
                    }
                    _ => {}
                }
                positional_idx += 1;
            }
            _ => {}
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
    println!("🧠 BATTLEFIELD BLUEPRINT // UNIFIED SINGLE-MODEL RL TRAINER");
    println!(
        "   Hardware Device:    {}",
        if gpu_avail {
            format!("🚀 GPU ACCELERATED // {}", gpu_name)
        } else {
            "💻 CPU Fallback".to_string()
        }
    );
    println!(
        "   Neural Architecture:Deep Dual-Head MLP ({} -> {} -> {} -> {} + {})",
        STATE_DIM, DEFAULT_HIDDEN_SIZE, DEFAULT_HIDDEN2_SIZE, ACTION_DIM, FORMATION_DIM
    );
    println!(
        "   Hidden Neurons:     {} neurons (Layer 1: {}, Layer 2: {})",
        DEFAULT_HIDDEN_SIZE + DEFAULT_HIDDEN2_SIZE,
        DEFAULT_HIDDEN_SIZE,
        DEFAULT_HIDDEN2_SIZE
    );
    println!(
        "   Total Parameters:   {} trainable weights & biases",
        total_params
    );
    println!("   Episodes per Mode:  {}", total_episodes);
    println!(
        "   CPU Multi-threading:{} parallel worker threads (Cores / 4 = {})",
        num_threads, default_threads
    );
    println!(
        "   Fast-Forward dt:    {:.3}s/step (High-Speed Tick)",
        time_scale_dt
    );
    println!(
        "   Max Game Time:      {:.0}s (Expanded Endgame Window)",
        max_game_time
    );
    println!(
        "   Training Mode:      {}",
        if resume_training {
            "🔄 Tiếp tục huấn luyện từ model trước đó (Resume Mode)"
        } else {
            "🌱 Huấn luyện lại từ đầu (Scratch Mode - Trọng số ngẫu nhiên He/Xavier)"
        }
    );
    println!("   Curriculum Sched:   1 vòng Auto (7 archetypes) -> 100% Bot Self-Play");
    println!(
        "   State Dimension:    {} inputs (Deep Economy & Resource Flow)",
        STATE_DIM
    );
    println!("   Action Dimension:   {} strategic actions", ACTION_DIM);
    println!("   Formation Dimension:{} formations", FORMATION_DIM);
    println!("   Unified Model File: rl_model.json");
    if let Some(ref d_dir) = drive_backup_dir {
        println!(
            "   Google Drive Backup:📂 {} (Tự động đồng bộ sau mỗi kịch bản)",
            d_dir
        );
    }
    println!(
        "   Scenarios Included: {:?}",
        target_scenarios
            .iter()
            .map(|s| s.name())
            .collect::<Vec<_>>()
    );
    println!("============================================================");

    let start_wall_time = Instant::now();
    let config = TrainConfig {
        episodes_per_mode: total_episodes,
        time_scale_dt,
        max_game_time,
        num_threads,
        drive_backup_dir: drive_backup_dir.clone(),
    };

    // Khởi tạo hoặc nạp trọng số dựa trên tùy chọn resume_training (ưu tiên Google Drive nếu có)
    let mut unified_brain = if resume_training {
        let loaded_from_drive = if let Some(ref b_dir) = drive_backup_dir {
            let drive_path = format!("{}/rl_model.json", b_dir);
            if let Ok(data) = std::fs::read_to_string(&drive_path) {
                if let Ok(b) = serde_json::from_str::<RlBrain>(&data) {
                    println!(
                        "☁️ [Google Drive] Đã nạp thành công model từ Google Drive: {} ({} tham số).",
                        drive_path,
                        b.total_parameters()
                    );
                    Some(b)
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        if let Some(b) = loaded_from_drive {
            b
        } else if let Some(loaded) = RlBrain::load_saved_model() {
            println!(
                "🔄 [Resume] Đã nạp thành công model đã có từ file đĩa ({} tham số).",
                loaded.total_parameters()
            );
            loaded
        } else {
            println!("ℹ️ [Resume] Chưa tìm thấy file model trên đĩa, sử dụng trọng số mặc định.");
            RlBrain::default()
        }
    } else {
        println!("🌱 [From Scratch] Khởi tạo mạng mới hoàn toàn với trọng số chuẩn He/Xavier.");
        RlBrain::new_random(
            STATE_DIM,
            DEFAULT_HIDDEN_SIZE,
            DEFAULT_HIDDEN2_SIZE,
            ACTION_DIM,
        )
    };
    unified_brain.sanitize();

    // Tự động kiểm tra nếu model cũ bị bão hòa (toàn ±5.0 hoặc ±3.0) do gradient trước đây
    if resume_training {
        let is_saturated = {
            let count_extremes = unified_brain
                .w1
                .iter()
                .flatten()
                .filter(|&&v| v.abs() >= 2.99)
                .count();
            let total = unified_brain.w1.iter().flatten().count().max(1);
            (count_extremes as f32 / total as f32) > 0.4
        };
        if is_saturated {
            println!("⚠️ Phát hiện model nạp vào bị bão hòa trọng số (toàn ±5.0/±3.0). Tự động tái tạo trọng số chuẩn He/Xavier!");
            unified_brain = RlBrain::new_random(
                STATE_DIM,
                DEFAULT_HIDDEN_SIZE,
                DEFAULT_HIDDEN2_SIZE,
                ACTION_DIM,
            );
        }
    }

    println!(
        "🧠 Bộ não AI hoạt động sẵn sàng: {} tham số (Valid: {}).",
        unified_brain.total_parameters(),
        unified_brain.is_valid()
    );

    // Huấn luyện xoay vòng qua các kịch bản để tích lũy kinh nghiệm vào 1 model duy nhất
    for (idx, &scenario) in target_scenarios.iter().enumerate() {
        println!(
            "\n>>> [Giai đoạn {}/{}] Huấn luyện kịch bản: {} ({} ván) <<<",
            idx + 1,
            target_scenarios.len(),
            scenario.name(),
            config.episodes_per_mode
        );
        unified_brain = train_scenario_with_brain(scenario, &config, unified_brain);
        save_unified_model_with_backup(
            &unified_brain,
            config.drive_backup_dir.as_deref(),
            Some(scenario.name()),
        );
    }

    println!("\n============================================================");
    println!("⚔️  TIẾN HÀNH ĐẤU THỬ NGHIỆM ĐÁNH GIÁ (EVALUATION BENCHMARK)");
    println!("   - Trạng thái AI: Greedy Exploitation (Epsilon = 0.00)");
    println!("   - Đối thủ: Hệ thống Autoplay đa phong cách & Đại sư đa công nghệ");
    println!("============================================================");

    let eval_matches = 5; // Số ván test mỗi chế độ
    println!(
        "{:<22} | {:<7} | {:<16} | {:<12}",
        "Chế độ (Mode)", "Tỉ lệ", "Thắng/Thua/Hòa", "Avg Game Time"
    );
    println!("------------------------------------------------------------");

    let mut total_eval_wins = 0;
    let mut total_eval_matches = 0;

    for &scenario in &target_scenarios {
        let (wins, losses, draws, avg_time) =
            evaluate_model_on_scenario(scenario, &unified_brain, eval_matches, time_scale_dt);
        let win_rate = (wins as f32 / eval_matches as f32) * 100.0;
        total_eval_wins += wins;
        total_eval_matches += eval_matches;

        println!(
            "{:<22} | {:>5.1}%  | W:{} L:{} D:{}     | {:.1}s",
            scenario.name(),
            win_rate,
            wins,
            losses,
            draws,
            avg_time
        );
    }
    println!("------------------------------------------------------------");
    let overall_winrate = if total_eval_matches > 0 {
        (total_eval_wins as f32 / total_eval_matches as f32) * 100.0
    } else {
        0.0
    };
    println!(
        "🏆 TỔNG KẾT TỈ LỆ THẮNG TOÀN DIỆN: {:>5.1}% ({}/{} ván thắng)",
        overall_winrate, total_eval_wins, total_eval_matches
    );

    save_unified_model_with_backup(
        &unified_brain,
        config.drive_backup_dir.as_deref(),
        Some("final"),
    );

    println!("============================================================");
    println!("🎉 HOÀN THÀNH HUẤN LUYỆN TOÀN DIỆN CHO 1 MODEL AI DUY NHẤT!");
    println!(
        "⏱️  Tổng thời gian: {:.2}s",
        start_wall_time.elapsed().as_secs_f32()
    );
    println!("💾 File model cập nhật: assets/rl_model.json");
    if let Some(ref d_dir) = config.drive_backup_dir {
        println!("☁️ Google Drive:       {}/rl_model.json", d_dir);
    }
    println!("============================================================");
}

struct EpisodeResult {
    transitions: Vec<TransitionSample>,
    is_win: bool,
    is_loss: bool,
    _is_draw: bool,
    ep_reward: f32,
    action_counts: [usize; ACTION_DIM],
    formation_counts: [usize; FORMATION_DIM],
    _game_time: f32,
}

fn simulate_single_episode(
    scenario: Scenario,
    brain: &RlBrain,
    opp_champ: Option<&RlBrain>,
    opponent_style: OpponentStyle,
    time_scale_dt: f32,
    max_game_time: f32,
    epsilon: f32,
) -> EpisodeResult {
    let mut engine = GameEngine::new_headless();
    let slots = scenario.configure_slots();
    engine.player_slots = slots;
    engine.reset_game();
    engine.is_headless = true;
    engine.is_paused = false;
    engine.ai_state.training_enabled = true;

    let learner_team = Team::Enemy;
    let mut active_brain = brain.clone();
    active_brain.epsilon = epsilon;
    engine.team_ai_mut(learner_team).rl_brain = active_brain;

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
                    if let Some(champ) = opp_champ {
                        engine.team_ai_mut(team).rl_brain = champ.clone();
                        engine.team_ai_mut(team).rl_brain.epsilon = 0.05;
                    }
                }
            }
        } else {
            match opponent_style {
                OpponentStyle::VsAuto(_) => {
                    let cfg =
                        create_training_autoplay_config(team, AutoTrainingArchetype::MacroFortress);
                    engine.team_autoplay_mut(team).config = cfg;
                }
                OpponentStyle::BotSelfPlay => {
                    // Chuyển hẳn sang bot AI tự đối kháng cho cả đồng đội
                    engine.team_autoplay_mut(team).config.enabled = false;
                    engine.team_ai_mut(team).rl_brain = brain.clone();
                    engine.team_ai_mut(team).rl_brain.epsilon = epsilon;
                }
            }
        }
    }

    let mut ep_reward = 0.0_f32;
    let mut mid_adapt_timer = 0.0_f32;
    let mut decision_sample_timer = 0.0_f32;
    let mut transitions = Vec::with_capacity(512);
    let mut action_counts = [0usize; ACTION_DIM];
    let mut formation_counts = [0usize; FORMATION_DIM];

    // Trích xuất trạng thái và snapshot ban đầu đúng 1 lần trước vòng lặp
    let mut prev_state = extract_state_vector_for_team(&engine, learner_team);
    let mut prev_snap = RewardSnapshot::capture(&engine, learner_team);

    // Ghi nhận state ban đầu cho các bot AI khác trong self-play
    let mut other_prev_states: [Option<Vec<f32>>; MAX_PLAYERS] = Default::default();
    if matches!(opponent_style, OpponentStyle::BotSelfPlay) {
        for (slot_idx, state_slot) in other_prev_states.iter_mut().enumerate() {
            let t = Team::from_index(slot_idx);
            if engine.is_slot_active(t)
                && t != learner_team
                && !engine.team_autoplay(t).config.enabled
            {
                *state_slot = Some(extract_state_vector_for_team(&engine, t));
            }
        }
    }

    while !engine.game_over && engine.game_time < max_game_time {
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

        let action = engine.team_ai(learner_team).current_action;
        let formation = engine.team_ai(learner_team).current_formation;
        action_counts[action.to_index()] += 1;
        formation_counts[formation.to_index()] += 1;

        decision_sample_timer += time_scale_dt;
        let is_done = engine.game_over || engine.game_time >= max_game_time;

        // Lazy Extraction: Chỉ trích xuất vector 64 chiều và snapshot khi đến chu kỳ lấy mẫu quyết định hoặc kết thúc trận
        if decision_sample_timer >= 1.0 || is_done {
            decision_sample_timer = 0.0;

            let cur_state = extract_state_vector_for_team(&engine, learner_team);
            let cur_snap = RewardSnapshot::capture(&engine, learner_team);

            let step_reward = compute_ai_step_reward(
                &prev_snap,
                &cur_snap,
                action,
                formation,
                engine.game_over,
                engine.game_time,
                max_game_time,
            );
            ep_reward += step_reward;

            transitions.push(TransitionSample {
                state: prev_state,
                action: action.to_index(),
                formation: formation.to_index(),
                reward: step_reward,
                next_state: cur_state.clone(),
                done: is_done,
            });

            // Multi-Agent Sharing: Các slot AI khác trong BotSelfPlay cùng học từ kinh nghiệm
            if matches!(opponent_style, OpponentStyle::BotSelfPlay) {
                for (slot_idx, other_prev_opt) in other_prev_states.iter_mut().enumerate() {
                    let other_team = Team::from_index(slot_idx);
                    if let Some(other_prev) = other_prev_opt.take() {
                        let other_next = extract_state_vector_for_team(&engine, other_team);
                        let other_act = engine.team_ai(other_team).current_action;
                        let other_form = engine.team_ai(other_team).current_formation;
                        let is_hostile = engine.are_hostile(learner_team, other_team);
                        let other_reward = if is_hostile {
                            -step_reward * 0.8
                        } else {
                            step_reward * 0.9
                        };
                        transitions.push(TransitionSample {
                            state: other_prev,
                            action: other_act.to_index(),
                            formation: other_form.to_index(),
                            reward: other_reward,
                            next_state: other_next.clone(),
                            done: is_done,
                        });
                        *other_prev_opt = Some(other_next);
                    }
                }
            }

            prev_state = cur_state;
            prev_snap = cur_snap;
        }

        if is_done {
            break;
        }
    }

    let final_opp_ccs = count_hostile_ccs(&engine, learner_team);
    let final_my_ccs = count_ally_ccs(&engine, learner_team);
    let is_win = final_opp_ccs == 0 && final_my_ccs > 0;
    let is_loss = final_my_ccs == 0 && final_opp_ccs > 0;
    let is_draw = !is_win && !is_loss;

    EpisodeResult {
        transitions,
        is_win,
        is_loss,
        _is_draw: is_draw,
        ep_reward,
        action_counts,
        formation_counts,
        _game_time: engine.game_time,
    }
}

fn train_scenario_with_brain(
    scenario: Scenario,
    config: &TrainConfig,
    mut brain: RlBrain,
) -> RlBrain {
    let mode = scenario.to_mode();
    let total_episodes = config.episodes_per_mode;
    let time_scale_dt = config.time_scale_dt;
    let max_game_time = config.max_game_time;
    let num_threads = config.num_threads.max(1);

    let mut champion_pool: Vec<RlBrain> = vec![brain.clone()];
    let mut total_wins = 0;
    let mut total_losses = 0;
    let mut total_draws = 0;
    let mut action_counts = [0usize; ACTION_DIM];
    let mut formation_counts = [0usize; FORMATION_DIM];

    brain.epsilon = brain.epsilon.max(0.15);

    // Khởi tạo Persistent GPU Trainer giữ trọng số và bộ đệm thường trú trong VRAM (Zero cudaMalloc / cudaFree)
    let mut persistent_trainer = PersistentGpuTrainer::new(&brain, 256);

    let start_time = Instant::now();
    let mut completed_episodes = 0;

    while completed_episodes < total_episodes {
        let wave_size = (total_episodes - completed_episodes).min(num_threads);

        let wave_results: Vec<EpisodeResult> = std::thread::scope(|s| {
            let mut handles = Vec::with_capacity(wave_size);
            for ep_idx in 0..wave_size {
                let current_ep = completed_episodes + ep_idx + 1;
                // Đúng 1 vòng Auto cho đủ 7 thể loại có sẵn, sau đó chuyển hẳn 100% sang Bot tự đối kháng
                let opponent_style = if current_ep <= ALL_AUTO_ARCHETYPES.len() {
                    OpponentStyle::VsAuto(ALL_AUTO_ARCHETYPES[current_ep - 1])
                } else {
                    OpponentStyle::BotSelfPlay
                };

                let worker_brain = brain.clone();
                let opp_champ = if champion_pool.is_empty() {
                    Some(brain.clone())
                } else {
                    Some(champion_pool[current_ep % champion_pool.len()].clone())
                };
                let eps = brain.epsilon;

                handles.push(s.spawn(move || {
                    simulate_single_episode(
                        scenario,
                        &worker_brain,
                        opp_champ.as_ref(),
                        opponent_style,
                        time_scale_dt,
                        max_game_time,
                        eps,
                    )
                }));
            }
            handles.into_iter().map(|h| h.join().unwrap()).collect()
        });

        let mut wave_reward_sum = 0.0f32;
        let mut all_transitions = Vec::new();

        for res in wave_results {
            completed_episodes += 1;
            wave_reward_sum += res.ep_reward;

            if res.is_win {
                total_wins += 1;
                let mut champ = brain.clone();
                champ.sanitize();
                champion_pool.push(champ);
                if champion_pool.len() > 6 {
                    champion_pool.remove(0);
                }
            } else if res.is_loss {
                total_losses += 1;
            } else {
                total_draws += 1;
            }

            for (acc, &inc) in action_counts.iter_mut().zip(&res.action_counts) {
                *acc += inc;
            }
            for (acc, &inc) in formation_counts.iter_mut().zip(&res.formation_counts) {
                *acc += inc;
            }

            all_transitions.extend(res.transitions);

            // Decay epsilon
            brain.epsilon = (brain.epsilon * 0.985).max(0.02);
        }

        // Huấn luyện siêu tốc trên GPU VRAM theo các mini-batch 256 mẫu
        for chunk in all_transitions.chunks(persistent_trainer.max_batch_size) {
            persistent_trainer.train_batch(chunk, brain.learning_rate, brain.gamma);
        }

        // Đồng bộ trọng số từ VRAM về brain và sanitize sau mỗi đợt
        persistent_trainer.sync_to_brain(&mut brain);
        brain.sanitize();

        let elapsed_sec = start_time.elapsed().as_secs_f32().max(0.001);
        let win_rate = (total_wins as f32 / completed_episodes as f32) * 100.0;
        let eps_per_sec = (completed_episodes as f32) / elapsed_sec;
        let avg_wave_rew = wave_reward_sum / (wave_size as f32);

        if completed_episodes <= num_threads
            || completed_episodes % (num_threads * 2).max(10) == 0
            || completed_episodes == total_episodes
        {
            let phase_tag = if completed_episodes <= ALL_AUTO_ARCHETYPES.len() {
                "Auto"
            } else {
                "SelfPlay"
            };
            println!(
                "[{:<18}] Ep {:3}/{} | {:<8} | Win: {:5.1}% (W:{} L:{} D:{}) | Champs: {} | Rew: {:+6.1} | Eps: {:.3} | Speed: {:.1} ep/s ({} th)",
                mode.name(),
                completed_episodes,
                total_episodes,
                phase_tag,
                win_rate,
                total_wins,
                total_losses,
                total_draws,
                champion_pool.len(),
                avg_wave_rew,
                brain.epsilon,
                eps_per_sec,
                num_threads
            );
        }
    }

    brain
}

fn evaluate_model_on_scenario(
    scenario: Scenario,
    brain: &RlBrain,
    matches: usize,
    dt: f32,
) -> (usize, usize, usize, f32) {
    let learner_team = Team::Enemy;

    // Chạy song song các ván đánh giá trên đa luồng CPU
    let results: Vec<(bool, bool, bool, f32)> = std::thread::scope(|s| {
        let mut handles = Vec::with_capacity(matches);
        for match_idx in 1..=matches {
            let mut test_brain = brain.clone();
            test_brain.epsilon = 0.0;

            handles.push(s.spawn(move || {
                let mut engine = GameEngine::new_headless();
                let slots = scenario.configure_slots();
                engine.player_slots = slots;
                engine.reset_game();
                engine.is_headless = true;
                engine.is_paused = false;

                engine.team_ai_mut(learner_team).rl_brain = test_brain;
                engine.team_ai_mut(learner_team).training_enabled = false;
                engine.team_autoplay_mut(learner_team).config.enabled = false;

                let arch = match match_idx % 4 {
                    0 => AutoTrainingArchetype::OmniTechGrandmaster,
                    1 => AutoTrainingArchetype::CombinedTitanArtilleryMage,
                    2 => AutoTrainingArchetype::ScienceArtillery,
                    _ => AutoTrainingArchetype::MonsterSwarmBehemoth,
                };

                for slot_idx in 0..MAX_PLAYERS {
                    let team = Team::from_index(slot_idx);
                    if !engine.is_slot_active(team) || team == learner_team {
                        continue;
                    }
                    if engine.are_hostile(learner_team, team) {
                        let cfg = create_training_autoplay_config(team, arch);
                        engine.team_autoplay_mut(team).config = cfg;
                    } else {
                        let cfg = create_training_autoplay_config(
                            team,
                            AutoTrainingArchetype::MacroFortress,
                        );
                        engine.team_autoplay_mut(team).config = cfg;
                    }
                }

                while !engine.game_over && engine.game_time < 1200.0 {
                    rust_engine::simulation::step(&mut engine, dt);
                }

                let opp_ccs = count_hostile_ccs(&engine, learner_team);
                let my_ccs = count_ally_ccs(&engine, learner_team);

                let is_win = opp_ccs == 0 && my_ccs > 0;
                let is_loss = my_ccs == 0 && opp_ccs > 0;
                let is_draw = !is_win && !is_loss;
                (is_win, is_loss, is_draw, engine.game_time)
            }));
        }
        handles.into_iter().map(|h| h.join().unwrap()).collect()
    });

    let mut wins = 0;
    let mut losses = 0;
    let mut draws = 0;
    let mut total_time = 0.0;

    for (w, l, d, t) in results {
        if w {
            wins += 1;
        } else if l {
            losses += 1;
        } else if d {
            draws += 1;
        }
        total_time += t;
    }

    let avg_time = total_time / (matches.max(1) as f32);
    (wins, losses, draws, avg_time)
}

fn save_unified_model(brain: &RlBrain) {
    save_unified_model_with_backup(brain, None, None);
}

fn save_unified_model_with_backup(
    brain: &RlBrain,
    backup_dir: Option<&str>,
    scenario_tag: Option<&str>,
) {
    let mut safe_brain = brain.clone();
    safe_brain.sanitize();

    if !safe_brain.is_valid() {
        eprintln!(
            "❌ [LỖI NGHIÊM TRỌNG] Không thể lưu model: phát hiện trọng số không hợp lệ (NaN/Inf)!"
        );
        return;
    }

    let filename = "rl_model.json";
    let model_json = serde_json::to_string_pretty(&safe_brain).expect("Serialize RL Brain failed");

    for dir in &["assets", "rust-engine/assets", "../assets"] {
        if std::path::Path::new(dir).exists() {
            let path = format!("{}/{}", dir, filename);
            if let Ok(mut file) = File::create(&path) {
                let _ = file.write_all(model_json.as_bytes());
                println!("💾 [Unified Model] Đã lưu model: {}", path);
            }
        }
    }

    if let Some(backup) = backup_dir {
        let trimmed = backup.trim();
        if !trimmed.is_empty() {
            if let Err(e) = std::fs::create_dir_all(trimmed) {
                eprintln!(
                    "⚠️ [Google Drive] Không thể tạo thư mục backup {}: {}",
                    trimmed, e
                );
            } else {
                let main_backup_path = format!("{}/{}", trimmed, filename);
                if let Ok(mut file) = File::create(&main_backup_path) {
                    let _ = file.write_all(model_json.as_bytes());
                    println!(
                        "☁️ [Google Drive Auto-Sync] Đã đồng bộ an toàn: {}",
                        main_backup_path
                    );
                }
                if let Some(tag) = scenario_tag {
                    let snap_backup_path = format!("{}/rl_model_checkpoint_{}.json", trimmed, tag);
                    if let Ok(mut file) = File::create(&snap_backup_path) {
                        let _ = file.write_all(model_json.as_bytes());
                        println!(
                            "📦 [Checkpoint Snapshot] Đã lưu mốc giai đoạn: {}",
                            snap_backup_path
                        );
                    }
                }
            }
        }
    }
}

#[allow(dead_code)]
fn count_hostile_combat(engine: &GameEngine, my_team: Team) -> usize {
    engine
        .units
        .iter()
        .filter(|u| engine.are_hostile(my_team, u.team) && u.unit_type != UnitType::Engineer)
        .count()
}

#[allow(dead_code)]
fn count_ally_combat(engine: &GameEngine, my_team: Team) -> usize {
    engine
        .units
        .iter()
        .filter(|u| engine.are_allies(my_team, u.team) && u.unit_type != UnitType::Engineer)
        .count()
}

fn count_hostile_ccs(engine: &GameEngine, my_team: Team) -> usize {
    engine
        .buildings
        .iter()
        .filter(|b| {
            engine.are_hostile(my_team, b.team) && b.building_type == BuildingType::CommandCenter
        })
        .count()
}

fn count_ally_ccs(engine: &GameEngine, my_team: Team) -> usize {
    engine
        .buildings
        .iter()
        .filter(|b| {
            engine.are_allies(my_team, b.team) && b.building_type == BuildingType::CommandCenter
        })
        .count()
}

#[allow(dead_code)]
fn count_ally_reactors(engine: &GameEngine, my_team: Team) -> usize {
    engine
        .buildings
        .iter()
        .filter(|b| {
            engine.are_allies(my_team, b.team) && b.building_type == BuildingType::PowerReactor
        })
        .count()
}

#[allow(dead_code)]
fn count_ally_converters(engine: &GameEngine, my_team: Team) -> usize {
    engine
        .buildings
        .iter()
        .filter(|b| {
            engine.are_allies(my_team, b.team) && b.building_type == BuildingType::EnergyConverter
        })
        .count()
}

#[allow(dead_code)]
fn count_hostile_buildings(engine: &GameEngine, my_team: Team) -> usize {
    engine
        .buildings
        .iter()
        .filter(|b| engine.are_hostile(my_team, b.team))
        .count()
}

#[allow(dead_code)]
fn count_hostile_turrets(engine: &GameEngine, my_team: Team) -> usize {
    engine
        .buildings
        .iter()
        .filter(|b| {
            engine.are_hostile(my_team, b.team) && b.building_type == BuildingType::TurretTower
        })
        .count()
}
