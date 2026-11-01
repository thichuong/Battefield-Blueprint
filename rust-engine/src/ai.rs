use crate::components::*;
use crate::rl::{BotStrategicAction, RlBrain, STATE_DIM};
use crate::GameEngine;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AiState {
    pub attack_wave: u32,
    pub is_attacking: bool,
    pub wave_target_size: usize,
    pub rally_point: (f32, f32),
    pub expansion_attempted: bool,
    pub last_attack_target: Option<(f32, f32)>,

    // RL State
    pub rl_brain: RlBrain,
    pub current_action: BotStrategicAction,
    pub decision_timer: f32,
    pub last_state: Vec<f32>,
    pub cumulative_reward: f32,
    pub training_enabled: bool,
}

impl Default for AiState {
    fn default() -> Self {
        Self {
            attack_wave: 0,
            is_attacking: false,
            wave_target_size: 10,
            rally_point: (1950.0, 580.0),
            expansion_attempted: false,
            last_attack_target: None,
            rl_brain: RlBrain::default(),
            current_action: BotStrategicAction::AutoEcoExpand,
            decision_timer: 0.0,
            last_state: vec![0.0; STATE_DIM],
            cumulative_reward: 0.0,
            training_enabled: false,
        }
    }
}

pub fn extract_state_vector(engine: &GameEngine) -> Vec<f32> {
    let mut s = Vec::with_capacity(STATE_DIM);

    // 1. Tiến độ thời gian trận đấu (chuẩn hóa 0-600s)
    let time_norm = (engine.game_time / 600.0).clamp(0.0, 1.0);
    s.push(time_norm);

    // 2. Giai đoạn: Early (<90s), Mid (90-240s), Late (>240s)
    let is_early = if engine.game_time < 90.0 { 1.0 } else { 0.0 };
    let is_mid = if engine.game_time >= 90.0 && engine.game_time < 240.0 { 1.0 } else { 0.0 };
    s.push(is_early);
    s.push(is_mid);

    // 3. Kinh tế Bot: Khoáng sản & Năng lượng (chuẩn hóa)
    s.push((engine.enemy_eco.minerals / 1000.0).clamp(0.0, 2.0));
    s.push((engine.enemy_eco.energy / 500.0).clamp(0.0, 2.0));

    // 4. Dân số & Dư địa Supply
    let supply_ratio = if engine.enemy_eco.max_supply > 0 {
        (engine.enemy_eco.supply as f32 / engine.enemy_eco.max_supply as f32).clamp(0.0, 1.0)
    } else {
        0.0
    };
    s.push(supply_ratio);

    // 5. Số thợ mỏ & Số nhà chính
    let workers = engine.units.iter().filter(|u| u.team == Team::Enemy && u.unit_type == UnitType::Engineer).count();
    let ccs = engine.buildings.iter().filter(|b| b.team == Team::Enemy && b.building_type == BuildingType::CommandCenter).count();
    s.push((workers as f32 / 18.0).clamp(0.0, 1.0));
    s.push((ccs as f32 / 3.0).clamp(0.0, 1.0));

    // 6. Số PowerReactor, Barracks, TurretTower
    let reactors = engine.buildings.iter().filter(|b| b.team == Team::Enemy && b.building_type == BuildingType::PowerReactor).count();
    let barracks = engine.buildings.iter().filter(|b| b.team == Team::Enemy && b.building_type == BuildingType::Barracks).count();
    let turrets = engine.buildings.iter().filter(|b| b.team == Team::Enemy && b.building_type == BuildingType::TurretTower).count();
    s.push((reactors as f32 / 12.0).clamp(0.0, 1.0));
    s.push((barracks as f32 / 6.0).clamp(0.0, 1.0));
    s.push((turrets as f32 / 8.0).clamp(0.0, 1.0));

    // 7. Quân sự: Số lính chiến đấu phe mình và phe Player
    let enemy_combat = engine.units.iter().filter(|u| u.team == Team::Enemy && u.unit_type != UnitType::Engineer).count();
    let player_combat = engine.units.iter().filter(|u| u.team == Team::Player && u.unit_type != UnitType::Engineer).count();
    s.push((enemy_combat as f32 / 40.0).clamp(0.0, 1.5));
    let mil_ratio = ((enemy_combat as f32 + 1.0) / (player_combat as f32 + 1.0)).clamp(0.1, 5.0) / 2.5 - 1.0;
    s.push(mil_ratio);

    // 8. Số công nghệ đã unlock của Bot vs Player
    let enemy_techs = count_unlocked_techs(&engine.enemy_tech);
    let player_techs = count_unlocked_techs(&engine.player_tech);
    s.push((enemy_techs as f32 / 8.0).clamp(0.0, 1.0));
    s.push((player_techs as f32 / 8.0).clamp(0.0, 1.0));

    // 9. Căn cứ bị đe dọa (có lính Player gần CC địch trong vòng 650px)
    let base_pos = (2150.0, 350.0);
    let threatened = engine.units.iter().any(|u| {
        u.team == Team::Player && ((u.x - base_pos.0).powi(2) + (u.y - base_pos.1).powi(2)) < 650.0_f32.powi(2)
    });
    s.push(if threatened { 1.0 } else { 0.0 });

    s
}

fn count_unlocked_techs(tech: &crate::resources::TechTreeState) -> usize {
    let mut count = 0;
    if tech.focused_laser { count += 1; }
    if tech.energy_shielding { count += 1; }
    if tech.dark_matter_rockets { count += 1; }
    if tech.titan_alloy_plating { count += 1; }
    if tech.celestial_flying_swords { count += 1; }
    if tech.golden_bell_shield { count += 1; }
    if tech.corrosive_curse { count += 1; }
    if tech.void_cataclysm { count += 1; }
    if tech.primeval_scales { count += 1; }
    if tech.behemoth_mutation { count += 1; }
    if tech.acid_banelings { count += 1; }
    if tech.hive_proliferation { count += 1; }
    if tech.optical_drills { count += 1; }
    if tech.overcharged_reactors { count += 1; }
    if tech.fortified_blueprints { count += 1; }
    if tech.magnetic_rounds { count += 1; }
    if tech.exo_boosters { count += 1; }
    if tech.superheated_blades { count += 1; }
    if tech.overload_strike { count += 1; }
    count
}

pub fn update_ai(engine: &mut GameEngine) {
    // 1. Phân bổ thợ mỏ rảnh (Micro reflex cơ bản giữ nguyên để bot không bao giờ đứng nhìn)
    assign_idle_enemy_workers(engine);

    // 2. Chu kỳ ra quyết định Chiến lược bằng RL (2.0s / lần)
    engine.ai_state.decision_timer += 0.5;
    if engine.ai_state.decision_timer >= 2.0 {
        engine.ai_state.decision_timer = 0.0;

        let state = extract_state_vector(engine);
        let action = engine.ai_state.rl_brain.select_action(&state, engine.ai_state.training_enabled);
        engine.ai_state.current_action = action;
        engine.ai_state.last_state = state;

        apply_strategic_action(engine, action);
    }

    // 3. Xử lý các hành động Direct / Manual nếu đang chọn chế độ Manual
    execute_active_strategy(engine);
}

fn assign_idle_enemy_workers(engine: &mut GameEngine) {
    let available_minerals: Vec<(u32, f32, f32)> = engine
        .minerals
        .iter()
        .filter(|m| m.amount > 0.0)
        .map(|m| (m.id, m.x, m.y))
        .collect();

    if available_minerals.is_empty() {
        return;
    }

    let enemy_workers: Vec<u32> = engine
        .units
        .iter()
        .filter(|u| u.team == Team::Enemy && u.unit_type == UnitType::Engineer && u.order == UnitOrder::Idle)
        .map(|u| u.id)
        .collect();

    for wid in enemy_workers {
        if let Some(w) = engine.units.iter_mut().find(|u| u.id == wid) {
            let nearest_mineral = available_minerals.iter().min_by(|a, b| {
                let da = (a.1 - w.x).powi(2) + (a.2 - w.y).powi(2);
                let db = (b.1 - w.x).powi(2) + (b.2 - w.y).powi(2);
                da.partial_cmp(&db).unwrap()
            }).map(|m| m.0);

            if let Some(target_mineral_id) = nearest_mineral {
                w.order = UnitOrder::Harvest {
                    target_mineral_id,
                    phase: HarvestPhase::MovingToNode,
                };
            }
        }
    }
}

pub fn apply_strategic_action(engine: &mut GameEngine, action: BotStrategicAction) {
    match action {
        BotStrategicAction::AutoEcoExpand => {
            engine.enemy_autoplay.config.enabled = true;
            engine.enemy_autoplay.config.priority_techs = vec![
                "optical_drills".to_string(),
                "overcharged_reactors".to_string(),
                "fortified_blueprints".to_string(),
            ];
            // Kích hoạt xây dựng nhà mở rộng kinh tế
            configure_enemy_autoplay_eco(&mut engine.enemy_autoplay.config);
        }
        BotStrategicAction::AutoDefenseFortify => {
            engine.enemy_autoplay.config.enabled = true;
            engine.enemy_autoplay.config.turret_defense.enabled = true;
            engine.enemy_autoplay.config.turret_defense.rows = 2;
            engine.enemy_autoplay.config.priority_techs = vec![
                "titan_alloy_plating".to_string(),
                "overload_strike".to_string(),
                "fortified_blueprints".to_string(),
            ];
            configure_enemy_autoplay_defense(&mut engine.enemy_autoplay.config);
        }
        BotStrategicAction::AutoTechScience => {
            engine.enemy_autoplay.config.enabled = true;
            engine.enemy_autoplay.config.priority_techs = vec![
                "focused_laser".to_string(),
                "energy_shielding".to_string(),
                "dark_matter_rockets".to_string(),
                "titan_alloy_plating".to_string(),
            ];
            configure_enemy_autoplay_units(&mut engine.enemy_autoplay.config, vec![
                ("LaserMarksman".to_string(), 2),
                ("Vanguard".to_string(), 1),
                ("DarkRocketeer".to_string(), 1),
            ]);
        }
        BotStrategicAction::AutoTechMagic => {
            engine.enemy_autoplay.config.enabled = true;
            engine.enemy_autoplay.config.priority_techs = vec![
                "celestial_flying_swords".to_string(),
                "golden_bell_shield".to_string(),
                "corrosive_curse".to_string(),
                "void_cataclysm".to_string(),
            ];
            configure_enemy_autoplay_units(&mut engine.enemy_autoplay.config, vec![
                ("TaoistSwordmaster".to_string(), 2),
                ("VoidWarlock".to_string(), 2),
            ]);
        }
        BotStrategicAction::AutoTechMonster => {
            engine.enemy_autoplay.config.enabled = true;
            engine.enemy_autoplay.config.priority_techs = vec![
                "primeval_scales".to_string(),
                "behemoth_mutation".to_string(),
                "acid_banelings".to_string(),
                "hive_proliferation".to_string(),
            ];
            configure_enemy_autoplay_units(&mut engine.enemy_autoplay.config, vec![
                ("AcidBaneling".to_string(), 3),
                ("BehemothTitan".to_string(), 1),
            ]);
        }
        BotStrategicAction::AutoAggressiveRush => {
            engine.enemy_autoplay.config.enabled = true;
            configure_enemy_autoplay_rush(&mut engine.enemy_autoplay.config);
        }
        BotStrategicAction::ManualDefendAndRepair => {
            // Tạm dừng tấn công của Auto, điều khiển toàn bộ quân chiến đấu về bảo vệ căn cứ
            engine.enemy_autoplay.config.enabled = false;
            let main_base_pos = (2150.0, 350.0);
            direct_defend_base(engine, main_base_pos);
        }
        BotStrategicAction::ManualStageAndHarass => {
            engine.enemy_autoplay.config.enabled = false;
            direct_stage_units(engine);
        }
        BotStrategicAction::ManualAllOutAttack => {
            engine.enemy_autoplay.config.enabled = false;
            direct_all_out_attack(engine);
        }
        BotStrategicAction::ManualRetreatAndRegroup => {
            engine.enemy_autoplay.config.enabled = false;
            direct_retreat_units(engine);
        }
    }
}

fn configure_enemy_autoplay_eco(config: &mut crate::autoplay::AutoPlayConfig) {
    if let Some(loop_prod) = config.loops.iter_mut().find(|l| l.id == "loop_production") {
        for card in loop_prod.cards.iter_mut() {
            if card.id == "card_prod_worker" {
                card.enabled = true;
                card.conditions = vec![crate::autoplay::AutoCondition::UnitCountLessThan {
                    unit_type: "Engineer".to_string(),
                    count: 16,
                }];
            }
            if card.id == "card_prod_reactor" {
                card.enabled = true;
            }
            if card.id == "card_prod_converter" {
                card.enabled = true;
            }
        }
    }
}

fn configure_enemy_autoplay_defense(config: &mut crate::autoplay::AutoPlayConfig) {
    if let Some(loop_prod) = config.loops.iter_mut().find(|l| l.id == "loop_production") {
        for card in loop_prod.cards.iter_mut() {
            if card.id == "card_prod_turret" {
                card.enabled = true;
                card.action = crate::autoplay::AutoAction::BuildStructure {
                    building_type: "TurretTower".to_string(),
                    count: 3,
                };
            }
        }
    }
}

fn configure_enemy_autoplay_rush(config: &mut crate::autoplay::AutoPlayConfig) {
    if let Some(loop_mil) = config.loops.iter_mut().find(|l| l.id == "loop_military") {
        for card in loop_mil.cards.iter_mut() {
            if card.id == "card_mil_attack" {
                card.conditions = vec![crate::autoplay::AutoCondition::UnitCountGreaterThan {
                    unit_type: "Combat".to_string(),
                    count: 12,
                }];
            }
        }
    }
}

fn configure_enemy_autoplay_units(
    config: &mut crate::autoplay::AutoPlayConfig,
    unit_targets: Vec<(String, u32)>,
) {
    if let Some(loop_mil) = config.loops.iter_mut().find(|l| l.id == "loop_military") {
        for (idx, (u_type, count)) in unit_targets.into_iter().enumerate() {
            let card_id = format!("card_mil_evo_{}", idx);
            if let Some(existing) = loop_mil.cards.iter_mut().find(|c| c.id == card_id) {
                existing.action = crate::autoplay::AutoAction::TrainUnit {
                    unit_type: u_type,
                    count,
                };
            } else {
                loop_mil.cards.push(crate::autoplay::AutoCard {
                    id: card_id,
                    name: format!("Sản xuất {}", u_type),
                    enabled: true,
                    conditions: vec![crate::autoplay::AutoCondition::Always],
                    action: crate::autoplay::AutoAction::TrainUnit {
                        unit_type: u_type,
                        count,
                    },
                });
            }
        }
    }
}

fn direct_defend_base(engine: &mut GameEngine, base_pos: (f32, f32)) {
    let nearest_enemy_unit = engine
        .units
        .iter()
        .filter(|u| u.team == Team::Player)
        .min_by(|a, b| {
            let da = (a.x - base_pos.0).powi(2) + (a.y - base_pos.1).powi(2);
            let db = (b.x - base_pos.0).powi(2) + (b.y - base_pos.1).powi(2);
            da.partial_cmp(&db).unwrap()
        })
        .map(|u| (u.x, u.y));

    let threat_target = nearest_enemy_unit.unwrap_or(base_pos);

    for u in engine.units.iter_mut() {
        if u.team == Team::Enemy && u.unit_type != UnitType::Engineer {
            u.order = UnitOrder::AttackMove {
                x: threat_target.0,
                y: threat_target.1,
            };
        }
    }
}

fn direct_stage_units(engine: &mut GameEngine) {
    let rally = engine.ai_state.rally_point;
    let combat_units: Vec<u32> = engine
        .units
        .iter()
        .filter(|u| u.team == Team::Enemy && u.unit_type != UnitType::Engineer)
        .map(|u| u.id)
        .collect();

    for (i, uid) in combat_units.iter().enumerate() {
        if let Some(u) = engine.units.iter_mut().find(|u| u.id == *uid) {
            if matches!(u.order, UnitOrder::Idle) {
                let offset_x = ((i % 5) as f32 - 2.0) * 26.0;
                let offset_y = ((i / 5) as f32) * 26.0;
                u.order = UnitOrder::MoveTo {
                    x: rally.0 + offset_x,
                    y: rally.1 + offset_y,
                };
            }
        }
    }
}

fn direct_all_out_attack(engine: &mut GameEngine) {
    let player_target = engine
        .buildings
        .iter()
        .find(|b| b.team == Team::Player && b.building_type == BuildingType::CommandCenter)
        .map(|b| (b.x, b.y))
        .or_else(|| {
            engine
                .buildings
                .iter()
                .find(|b| b.team == Team::Player)
                .map(|b| (b.x, b.y))
        })
        .unwrap_or((250.0, 2050.0));

    let combat_units: Vec<u32> = engine
        .units
        .iter()
        .filter(|u| u.team == Team::Enemy && u.unit_type != UnitType::Engineer)
        .map(|u| u.id)
        .collect();

    for (i, uid) in combat_units.iter().enumerate() {
        if let Some(u) = engine.units.iter_mut().find(|u| u.id == *uid) {
            let offset_x = ((i % 6) as f32 - 2.5) * 28.0;
            let offset_y = ((i / 6) as f32) * 28.0;
            u.order = UnitOrder::AttackMove {
                x: player_target.0 + offset_x,
                y: player_target.1 + offset_y,
            };
        }
    }
}

fn direct_retreat_units(engine: &mut GameEngine) {
    let retreat_point = (2150.0, 350.0);
    for u in engine.units.iter_mut() {
        if u.team == Team::Enemy && u.unit_type != UnitType::Engineer {
            u.order = UnitOrder::MoveTo {
                x: retreat_point.0,
                y: retreat_point.1,
            };
        }
    }
}

fn execute_active_strategy(engine: &mut GameEngine) {
    match engine.ai_state.current_action {
        BotStrategicAction::ManualDefendAndRepair => {
            let base_pos = (2150.0, 350.0);
            direct_defend_base(engine, base_pos);
        }
        BotStrategicAction::ManualAllOutAttack => {
            direct_all_out_attack(engine);
        }
        BotStrategicAction::ManualRetreatAndRegroup => {
            direct_retreat_units(engine);
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ai_state_default() {
        let state = AiState::default();
        assert_eq!(state.attack_wave, 0);
        assert!(!state.is_attacking);
        assert_eq!(state.current_action, BotStrategicAction::AutoEcoExpand);
    }

    #[test]
    fn test_state_extraction_dimension() {
        let engine = GameEngine::new_headless();
        let s = extract_state_vector(&engine);
        assert_eq!(s.len(), STATE_DIM);
    }

    #[test]
    fn test_strategic_action_mapping() {
        for idx in 0..10 {
            let action = BotStrategicAction::from_index(idx);
            assert_eq!(action.to_index(), idx);
        }
    }
}
