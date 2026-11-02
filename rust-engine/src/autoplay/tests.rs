use super::*;
use crate::components::{BuildingType, Team, UnitType};
use crate::resources::TechTreeState;
use crate::GameEngine;

#[test]
fn test_default_autoplay_config() {
    let config = AutoPlayConfig::default();
    assert!(!config.enabled);
    assert_eq!(config.loops.len(), 2);
    assert_eq!(config.loops[0].name, "Sản Xuất");
    assert_eq!(config.loops[0].cards.len(), 10);
    assert!(config.loops[0]
        .cards
        .iter()
        .any(|c| c.id == "card_prod_supply"));
    assert!(config.loops[0]
        .cards
        .iter()
        .any(|c| c.id == "card_prod_shock"));
    assert!(config.loops[0]
        .cards
        .iter()
        .any(|c| c.id == "card_prod_rifle"));
    let delay_card = config.loops[0]
        .cards
        .iter()
        .find(|c| c.id == "card_prod_delay")
        .unwrap();
    assert_eq!(delay_card.action, AutoAction::Delay { seconds: 2.0 });

    assert_eq!(config.loops[1].name, "Quân Sự");
    assert_eq!(config.loops[1].cards.len(), 2);
    assert!(config.loops[1]
        .cards
        .iter()
        .any(|c| c.id == "card_mil_attack"));
    let mil_delay = config.loops[1]
        .cards
        .iter()
        .find(|c| c.id == "card_mil_delay")
        .unwrap();
    assert_eq!(mil_delay.action, AutoAction::Delay { seconds: 2.0 });
}

#[test]
fn test_can_train_unit_tech_requirements() {
    let mut tech = TechTreeState::default();
    assert!(can_train_unit(&tech, UnitType::Engineer));
    assert!(can_train_unit(&tech, UnitType::ShockTrooper));
    assert!(can_train_unit(&tech, UnitType::Rifleman));
    assert!(!can_train_unit(&tech, UnitType::Vanguard));

    tech.energy_shielding = true;
    assert!(can_train_unit(&tech, UnitType::Vanguard));
}

#[test]
fn test_converter_condition() {
    let cond = AutoCondition::ReactorPerConverterRatio;
    // Kiểm tra đúng định dạng enum
    assert_eq!(cond, AutoCondition::ReactorPerConverterRatio);
}

#[test]
fn test_compute_defense_tower_spots() {
    let line = TurretDefenseLine {
        enabled: true,
        p1: (0.0, 0.0),
        p2: (300.0, 0.0),
        spacing: 100.0,
        rows: 2,
        row_spacing: 50.0,
    };
    let spots = compute_defense_tower_spots(&line);
    // Hàng 1: x = 0, 100, 200, 300 (4 tháp)
    // Hàng 2: x = 0, 100, 200, 300 (4 tháp)
    // Tổng: 8 tháp
    assert_eq!(spots.len(), 8);
    assert_eq!(spots[0], (0.0, 0.0));
    assert_eq!(spots[1], (100.0, 0.0));
    assert_eq!(spots[2], (200.0, 0.0));
    assert_eq!(spots[3], (300.0, 0.0));
}

#[test]
fn test_parse_building_and_tech_prerequisite() {
    assert_eq!(
        parse_building_type("PowerReactor"),
        Some(BuildingType::PowerReactor)
    );
    assert_eq!(
        parse_building_type("TurretTower"),
        Some(BuildingType::TurretTower)
    );
    assert_eq!(
        parse_building_type("Barracks"),
        Some(BuildingType::Barracks)
    );
    assert_eq!(
        parse_building_type("EnergyConverter"),
        Some(BuildingType::EnergyConverter)
    );
    assert_eq!(
        get_tech_prerequisite("titan_alloy_plating"),
        Some("dark_matter_rockets".to_string())
    );
}

#[test]
fn test_create_and_adapt_training_autoplay() {
    let mut engine = GameEngine::new_headless();
    let config =
        create_training_autoplay_config(Team::Player, AutoTrainingArchetype::ScienceArtillery);
    assert!(config.enabled);
    assert!(config.priority_techs.len() >= 5);
    assert_eq!(config.turret_defense.rows, 2);

    engine.autoplay.config = config;
    // Giả lập mở khóa công nghệ
    engine.player_tech.energy_shielding = true;
    engine.player_tech.focused_laser = true;
    engine.game_time = 120.0;

    adapt_training_autoplay_mid_match(&mut engine, Team::Player);

    let prod_loop = engine
        .autoplay
        .config
        .loops
        .iter()
        .find(|l| l.id == "loop_production")
        .unwrap();
    // Kiểm tra đã tự động chèn thẻ mua Vanguard và LaserMarksman vào loop_production
    assert!(prod_loop
        .cards
        .iter()
        .any(|c| c.id == "card_prod_vanguard" || c.id == "card_mil_vanguard"));
    assert!(prod_loop
        .cards
        .iter()
        .any(|c| c.id == "card_prod_laser" || c.id == "card_mil_laser"));
}

#[test]
fn test_train_ai_building_limit_doubling() {
    let mut engine = GameEngine::new_headless();
    engine.ai_state.training_enabled = true;
    let config =
        create_training_autoplay_config(Team::Player, AutoTrainingArchetype::MacroFortress);
    engine.autoplay.config = config;

    // Giả lập đã xây đủ 10 PowerReactor, 12 TurretTower, 6 Barracks
    let mut b_id = 1;
    for _ in 0..10 {
        b_id += 1;
        engine.buildings.push(engine.config.create_building(
            b_id,
            Team::Player,
            BuildingType::PowerReactor,
            600.0,
            4200.0,
            true,
        ));
    }
    for _ in 0..12 {
        b_id += 1;
        engine.buildings.push(engine.config.create_building(
            b_id,
            Team::Player,
            BuildingType::TurretTower,
            700.0,
            4100.0,
            true,
        ));
    }
    for _ in 0..6 {
        b_id += 1;
        engine.buildings.push(engine.config.create_building(
            b_id,
            Team::Player,
            BuildingType::Barracks,
            800.0,
            4000.0,
            true,
        ));
    }

    // Chạy thích ứng giữa trận cho AI Training
    adapt_training_autoplay_mid_match(&mut engine, Team::Player);

    let prod_loop = engine
        .autoplay
        .config
        .loops
        .iter()
        .find(|l| l.id == "loop_production")
        .unwrap();
    let reactor_card = prod_loop
        .cards
        .iter()
        .find(|c| c.id == "card_prod_reactor")
        .unwrap();
    let turret_card = prod_loop
        .cards
        .iter()
        .find(|c| c.id == "card_prod_turret")
        .unwrap();
    let barracks_card = prod_loop
        .cards
        .iter()
        .find(|c| c.id == "card_prod_barracks")
        .unwrap();

    // Kiểm tra các giới hạn đã được gấp đôi: 10 -> 20, 12 -> 24, 6 -> 12
    let reactor_cap = reactor_card
        .conditions
        .iter()
        .find_map(|c| match c {
            AutoCondition::BuildingCountLessThan {
                building_type,
                count,
            } if building_type == "PowerReactor" => Some(*count),
            _ => None,
        })
        .unwrap();
    assert_eq!(reactor_cap, 20);

    let turret_cap = turret_card
        .conditions
        .iter()
        .find_map(|c| match c {
            AutoCondition::BuildingCountLessThan {
                building_type,
                count,
            } if building_type == "TurretTower" => Some(*count),
            _ => None,
        })
        .unwrap();
    assert_eq!(turret_cap, 24);

    let barracks_cap = barracks_card
        .conditions
        .iter()
        .find_map(|c| match c {
            AutoCondition::BuildingCountLessThan {
                building_type,
                count,
            } if building_type == "Barracks" => Some(*count),
            _ => None,
        })
        .unwrap();
    assert_eq!(barracks_cap, 12);
}

#[test]
fn test_research_tech_full_tree_fallback() {
    let mut engine = GameEngine::new_headless();
    engine.player_eco.minerals = 1000.0;
    engine.player_eco.energy = 1000.0;

    // Ưu tiên chỉ có 1 tech
    engine.autoplay.config.priority_techs = vec!["focused_laser".to_string()];
    engine.player_tech.focused_laser = true;

    // Thực thi ResearchTech với priority_queue: khi priority tech đã mở, phải tự động chọn tech tiếp theo trong ALL_TECH_IDS
    let action = AutoAction::ResearchTech {
        tech_id: "focused_laser".to_string(),
        use_priority_queue: true,
    };

    let result = execute_action_for_team(&mut engine, Team::Player, 0, &action, 0.1);
    assert!(result);
    assert!(!engine.player_tech.active_researches.is_empty());
    // Không phải focused_laser vì nó đã được mở khóa
    assert_ne!(
        engine.player_tech.active_researches[0].tech_id,
        "focused_laser"
    );
}
