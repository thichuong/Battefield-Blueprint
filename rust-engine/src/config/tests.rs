use super::*;
use crate::components::{BuildingType, DamageType, Team, UnitType};

#[test]
fn test_default_game_config_has_all_units_and_buildings() {
    let cfg = GameConfig::default();
    let unit_types = [
        UnitType::Engineer,
        UnitType::ShockTrooper,
        UnitType::Rifleman,
        UnitType::Vanguard,
        UnitType::LaserMarksman,
        UnitType::DarkRocketeer,
        UnitType::TaoistSwordmaster,
        UnitType::VoidWarlock,
        UnitType::BehemothTitan,
        UnitType::AcidBaneling,
    ];
    for u in unit_types {
        assert!(cfg.get_unit(u).is_some(), "Missing config for unit {:?}", u);
    }

    let building_types = [
        BuildingType::CommandCenter,
        BuildingType::SupplyDepot,
        BuildingType::PowerReactor,
        BuildingType::Barracks,
        BuildingType::TurretTower,
        BuildingType::EnergyConverter,
    ];
    for b in building_types {
        assert!(
            cfg.get_building(b).is_some(),
            "Missing config for building {:?}",
            b
        );
    }
}

#[test]
fn test_game_config_json_roundtrip() {
    let cfg = GameConfig::default();
    let json = cfg.to_json();
    let parsed = GameConfig::from_json(&json).expect("Should parse json cleanly");
    assert_eq!(cfg, parsed);
}

#[test]
fn test_custom_config_modifies_unit_creation() {
    let mut cfg = GameConfig::default();
    if let Some(u) = cfg.units.get_mut("Engineer") {
        u.hp = 999.0;
        u.speed = 333.0;
    }

    let unit = cfg.create_unit(1, Team::Player, UnitType::Engineer, 100.0, 100.0);
    assert_eq!(unit.hp, 999.0);
    assert_eq!(unit.speed, 333.0);
}

#[test]
fn test_custom_config_modifies_building_creation() {
    let mut cfg = GameConfig::default();
    if let Some(b) = cfg.buildings.get_mut("CommandCenter") {
        b.hp = 5000.0;
    }

    let building = cfg.create_building(
        1,
        Team::Player,
        BuildingType::CommandCenter,
        200.0,
        200.0,
        true,
    );
    assert_eq!(building.hp, 5000.0);
    assert_eq!(building.build_progress, 1.0);
}

#[test]
fn test_custom_config_modifies_building_weapon_stats() {
    let mut cfg = GameConfig::default();
    if let Some(b) = cfg.buildings.get_mut("Barracks") {
        b.turret_range = 220.0;
        b.turret_damage = 35.0;
        b.turret_cooldown = 0.8;
        b.turret_damage_type = DamageType::Arcane;
    }

    let b_cfg = cfg.get_building(BuildingType::Barracks).unwrap();
    assert_eq!(b_cfg.turret_range, 220.0);
    assert_eq!(b_cfg.turret_damage, 35.0);
    assert_eq!(b_cfg.turret_cooldown, 0.8);
    assert_eq!(b_cfg.turret_damage_type, DamageType::Arcane);
}

#[test]
fn test_building_dimensions_and_labels_from_config() {
    let mut cfg = GameConfig::default();
    assert_eq!(
        cfg.get_building_dimensions(BuildingType::CommandCenter),
        (90.0, 90.0)
    );
    assert_eq!(
        cfg.get_building_short_name(BuildingType::SupplyDepot),
        "SUPPLY"
    );
    assert!(!cfg
        .get_building_description(BuildingType::PowerReactor)
        .is_empty());

    // Custom config modification propagates
    if let Some(b) = cfg.buildings.get_mut("CommandCenter") {
        b.width = 120.0;
        b.height = 130.0;
        b.short_name = "MEGA CC".to_string();
    }
    assert_eq!(
        cfg.get_building_dimensions(BuildingType::CommandCenter),
        (120.0, 130.0)
    );
    assert_eq!(
        cfg.get_building_short_name(BuildingType::CommandCenter),
        "MEGA CC"
    );
}

#[test]
fn test_unit_labels_and_descriptions_from_config() {
    let mut cfg = GameConfig::default();
    assert_eq!(cfg.get_unit_short_name(UnitType::ShockTrooper), "STRIKER");
    assert_eq!(
        cfg.get_unit_description(UnitType::Engineer),
        "Huấn luyện thợ mỏ / kỹ sư xây dựng."
    );

    // Custom modification
    if let Some(u) = cfg.units.get_mut("ShockTrooper") {
        u.short_name = "CYBER STRIKER".to_string();
        u.description = "Chiến binh plasma tương lai".to_string();
    }
    assert_eq!(
        cfg.get_unit_short_name(UnitType::ShockTrooper),
        "CYBER STRIKER"
    );
    assert_eq!(
        cfg.get_unit_description(UnitType::ShockTrooper),
        "Chiến binh plasma tương lai"
    );
}

#[test]
fn test_render_config_customization() {
    let mut cfg = GameConfig::default();
    assert_eq!(cfg.render.grid_size, 80.0);
    assert_eq!(cfg.render.bg_color, "#030d1a");

    cfg.render.grid_size = 100.0;
    cfg.render.bg_color = "#000000".to_string();

    let json = cfg.to_json();
    let parsed = GameConfig::from_json(&json).expect("Roundtrip with custom render config");
    assert_eq!(parsed.render.grid_size, 100.0);
    assert_eq!(parsed.render.bg_color, "#000000");
}

#[test]
fn test_eight_player_colors_unique_and_non_overlapping() {
    use crate::config::DEFAULT_PLAYER_COLORS;
    use crate::renderer::gpu_cache::{get_color_index, COLOR_HIT_FLASH};
    use std::collections::HashSet;

    // 1. Kiểm tra 8 màu mặc định không trùng nhau
    assert_eq!(DEFAULT_PLAYER_COLORS.len(), 8);
    let mut stroke_set = HashSet::new();
    let mut fill_set = HashSet::new();
    for &(stroke, fill) in &DEFAULT_PLAYER_COLORS {
        assert!(stroke_set.insert(stroke), "Trùng màu stroke: {}", stroke);
        assert!(fill_set.insert(fill), "Trùng màu fill: {}", fill);
    }

    // 2. Kiểm tra get_color_index cho 8 Team
    let mut color_indices = HashSet::new();
    for i in 0..8 {
        let team = Team::from_index(i);
        let idx = get_color_index(team, false);
        assert_eq!(idx, i);
        assert!(color_indices.insert(idx), "Trùng index màu: {}", idx);

        // Trường hợp bị đánh trúng (Hit flash)
        let flash_idx = get_color_index(team, true);
        assert_eq!(flash_idx, COLOR_HIT_FLASH);
    }
}

#[test]
fn test_get_team_color_matches_eight_players() {
    use crate::renderer::get_team_color;
    use crate::GameEngine;
    use std::collections::HashSet;

    let engine = GameEngine::new_headless();
    let mut team_colors = HashSet::new();

    for i in 0..8 {
        let team = Team::from_index(i);
        let (stroke, fill) = get_team_color(&engine, team);
        assert_eq!(stroke, crate::config::DEFAULT_PLAYER_COLORS[i].0);
        assert_eq!(fill, crate::config::DEFAULT_PLAYER_COLORS[i].1);
        assert!(team_colors.insert(stroke), "Trùng màu team: {}", stroke);
    }
}
