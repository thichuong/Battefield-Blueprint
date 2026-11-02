use super::*;
use crate::components::{FormationType, Team};
use crate::rl::{BotStrategicAction, STATE_DIM};
use crate::GameEngine;

#[test]
fn test_ai_state_default() {
    let state = AiState::default();
    assert_eq!(state.current_action, BotStrategicAction::ExpandEco);
}

#[test]
fn test_state_extraction_dimension() {
    let engine = GameEngine::new_headless();
    let s_enemy = extract_state_vector(&engine);
    assert_eq!(s_enemy.len(), STATE_DIM);

    let s_player = extract_state_vector_for_team(&engine, Team::Player);
    assert_eq!(s_player.len(), STATE_DIM);
}

#[test]
fn test_strategic_action_mapping() {
    for idx in 0..15 {
        let action = BotStrategicAction::from_index(idx);
        assert_eq!(action.to_index(), idx);
    }
}

#[test]
fn test_compute_ai_step_reward_cc_and_turrets() {
    let prev = RewardSnapshot {
        my_ccs: 1,
        my_turrets: 2,
        my_total_buildings: 5,
        ..Default::default()
    };

    let mut cur = prev.clone();
    cur.my_ccs = 2; // Xây thêm 1 CC
    cur.my_turrets = 4; // Xây thêm 2 tháp
    cur.my_total_buildings = 8; // Tăng 3 công trình

    let r = compute_ai_step_reward(
        &prev,
        &cur,
        BotStrategicAction::ExpandEco,
        FormationType::Line,
        false,
        120.0,
        1800.0,
    );

    // CC mới (+85.0), 2 tháp mới (+48.0), duy trì CC (+1.0), duy trì tháp (+0.8),
    // 3 nhà mới (+24.0), tổng nhà duy trì, action ExpandEco khi < 4 CC (+0.8)...
    assert!(
        r > 150.0,
        "Reward should be substantial for expanding base & building turrets, got {}",
        r
    );
}

#[test]
fn test_compute_ai_step_reward_penalizes_excess_banelings() {
    let prev = RewardSnapshot {
        my_banelings: 0,
        my_ccs: 1,
        my_combat: 5,
        ..Default::default()
    };

    let mut cur = prev.clone();
    cur.my_banelings = 5; // Quá nhiều baneling (vượt ngưỡng 2)

    let r = compute_ai_step_reward(
        &prev,
        &cur,
        BotStrategicAction::MilitarySwarmRush,
        FormationType::Line,
        false,
        60.0,
        1800.0,
    );

    // Phạt baneling vượt ngưỡng: -1.5 * (5 - 2) = -4.5
    // Phạt hành động tấn công khi my_combat < 18: -2.0
    assert!(
        r < 0.0,
        "Reward should be negative for spamming banelings and attacking early, got {}",
        r
    );
}

#[test]
fn test_compute_ai_step_reward_tech_branch_and_resources() {
    let prev = RewardSnapshot {
        my_unlocked_techs: 3,
        sci_techs: 3,
        minerals_rate: 10.0,
        energy_rate: 20.0,
        ..Default::default()
    };

    let mut cur = prev.clone();
    cur.my_unlocked_techs = 4;
    cur.sci_techs = 4; // Hoàn thành nhánh Science!
    cur.minerals_rate = 30.0; // Tăng tốc độ khoáng sản
    cur.energy_rate = 50.0; // Tăng tốc độ năng lượng

    let r = compute_ai_step_reward(
        &prev,
        &cur,
        BotStrategicAction::TechScienceBranch,
        FormationType::Line,
        false,
        150.0,
        1800.0,
    );

    // Mở khóa tech mới (+38.0), hoàn thành nhánh Khoa học (+90.0), tốc độ tài nguyên, duy trì nhánh...
    assert!(
        r > 120.0,
        "Reward should strongly reward completing a tech branch and resource generation, got {}",
        r
    );
}
