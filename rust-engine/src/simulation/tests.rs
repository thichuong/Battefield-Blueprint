use super::*;
use crate::components::*;
use crate::config::GameConfig;
use crate::resources::{PlayerEconomy, TechTreeState};

#[test]
fn test_energy_conversion_normal() {
    // Có 1 nhà chuyển đổi, năng lượng 1200 (> 1000), dt = 1.0
    let (used_en, gained_minerals) = calculate_energy_conversion(1200.0, 1, 1.0);
    assert_eq!(used_en, 30.0);
    let expected_minerals = 30.0 * (CONVERT_MINERALS_PER_SEC / CONVERT_ENERGY_PER_SEC);
    assert_eq!(gained_minerals, expected_minerals);
}

#[test]
fn test_energy_conversion_at_threshold_stops() {
    // Năng lượng đúng 1000 -> Không chuyển đổi
    let (used_en, gained_minerals) = calculate_energy_conversion(1000.0, 1, 1.0);
    assert_eq!(used_en, 0.0);
    assert_eq!(gained_minerals, 0.0);

    // Năng lượng thấp hơn 1000 -> Hoàn toàn dừng chuyển đổi
    let (used_en2, gained_minerals2) = calculate_energy_conversion(999.0, 1, 1.0);
    assert_eq!(used_en2, 0.0);
    assert_eq!(gained_minerals2, 0.0);
}

#[test]
fn test_energy_conversion_partial_surplus_does_not_drop_below_1000() {
    // Năng lượng là 1010, thặng dư chỉ là 10.
    // Nhu cầu chuyển đổi là 30, nhưng chỉ được phép tiêu thụ tối đa 10 để bảo toàn ngưỡng 1000.
    let (used_en, gained_minerals) = calculate_energy_conversion(1010.0, 1, 1.0);
    assert_eq!(used_en, 10.0);
    let expected = 10.0 * (CONVERT_MINERALS_PER_SEC / CONVERT_ENERGY_PER_SEC);
    assert!((gained_minerals - expected).abs() < 1e-5);
}

#[test]
fn test_energy_conversion_multiple_converters() {
    // 2 nhà chuyển đổi, năng lượng 1200, dt = 1.0 -> Tiêu thụ 60 năng lượng
    let (used_en, gained_minerals) = calculate_energy_conversion(1200.0, 2, 1.0);
    assert_eq!(used_en, 60.0);
    let expected = 60.0 * (CONVERT_MINERALS_PER_SEC / CONVERT_ENERGY_PER_SEC);
    assert_eq!(gained_minerals, expected);
}

#[test]
fn test_energy_conversion_no_converters() {
    // Không có nhà nào -> 0
    let (used_en, gained_minerals) = calculate_energy_conversion(5000.0, 0, 1.0);
    assert_eq!(used_en, 0.0);
    assert_eq!(gained_minerals, 0.0);
}

#[test]
fn test_building_combat_command_center_fires_at_enemy_unit() {
    let cfg = crate::config::GameConfig::default();
    let cc_cfg = cfg.get_building(BuildingType::CommandCenter).unwrap();
    let mut cc = cfg.create_building(
        1,
        Team::Player,
        BuildingType::CommandCenter,
        100.0,
        100.0,
        true,
    );

    let p_tech = TechTreeState::default();
    let e_tech = TechTreeState::default();
    let enemy_units = vec![(Team::Enemy, 200.0, 100.0)];
    let enemy_buildings = vec![];

    let proj = process_building_combat(
        &mut cc,
        Some(cc_cfg),
        &p_tech,
        &e_tech,
        &enemy_units,
        &enemy_buildings,
        0.05,
    );

    assert!(proj.is_some(), "Command Center should fire at nearby enemy");
    let p = proj.unwrap();
    assert_eq!(p.team, Team::Player);
    assert_eq!(p.target_x, 200.0);
    assert_eq!(p.target_y, 100.0);
    assert_eq!(p.damage, 100.0);
    assert_eq!(p.damage_type, DamageType::Physical);
    assert_eq!(p.speed, 900.0);
    assert!(!p.is_laser);
    assert_eq!(cc.current_cooldown, 1.0);
}

#[test]
fn test_custom_barracks_fires_when_configured() {
    let mut cfg = crate::config::GameConfig::default();
    if let Some(b) = cfg.buildings.get_mut("Barracks") {
        b.turret_range = 220.0;
        b.turret_damage = 35.0;
        b.turret_cooldown = 0.8;
        b.turret_damage_type = DamageType::Arcane;
    }

    let b_cfg = cfg.get_building(BuildingType::Barracks).unwrap();
    let mut barracks =
        cfg.create_building(2, Team::Player, BuildingType::Barracks, 100.0, 100.0, true);

    let p_tech = TechTreeState::default();
    let e_tech = TechTreeState::default();
    let enemy_units = vec![(Team::Enemy, 180.0, 100.0)];
    let enemy_buildings = vec![];

    let proj = process_building_combat(
        &mut barracks,
        Some(b_cfg),
        &p_tech,
        &e_tech,
        &enemy_units,
        &enemy_buildings,
        0.05,
    );

    assert!(proj.is_some(), "Custom configured Barracks should fire");
    let p = proj.unwrap();
    assert_eq!(p.damage, 35.0);
    assert_eq!(p.damage_type, DamageType::Arcane);
    assert_eq!(barracks.current_cooldown, 0.8);
}

#[test]
fn test_building_combat_receives_tech_upgrades() {
    let cfg = crate::config::GameConfig::default();
    let cc_cfg = cfg.get_building(BuildingType::CommandCenter).unwrap();
    let mut cc = cfg.create_building(
        1,
        Team::Player,
        BuildingType::CommandCenter,
        100.0,
        100.0,
        true,
    );

    let p_tech = TechTreeState {
        overload_strike: true,
        dark_matter_rockets: true,
        ..Default::default()
    };

    let e_tech = TechTreeState::default();
    let enemy_units = vec![(Team::Enemy, 200.0, 100.0)];
    let enemy_buildings = vec![];

    let proj = process_building_combat(
        &mut cc,
        Some(cc_cfg),
        &p_tech,
        &e_tech,
        &enemy_units,
        &enemy_buildings,
        0.05,
    );

    assert!(proj.is_some());
    let p = proj.unwrap();
    assert_eq!(p.damage, 140.0); // 100 * 1.4
    assert_eq!(p.damage_type, DamageType::Explosive);
    assert_eq!(p.aoe_radius, 40.0);
    assert_eq!(cc.current_cooldown, 0.6); // 1.0 * 0.6
}

#[test]
fn test_unarmed_building_does_not_fire() {
    let cfg = crate::config::GameConfig::default();
    let depot_cfg = cfg.get_building(BuildingType::SupplyDepot).unwrap();
    let mut depot = cfg.create_building(
        3,
        Team::Player,
        BuildingType::SupplyDepot,
        100.0,
        100.0,
        true,
    );

    let p_tech = TechTreeState::default();
    let e_tech = TechTreeState::default();
    let enemy_units = vec![(Team::Enemy, 120.0, 100.0)];
    let enemy_buildings = vec![];

    let proj = process_building_combat(
        &mut depot,
        Some(depot_cfg),
        &p_tech,
        &e_tech,
        &enemy_units,
        &enemy_buildings,
        0.05,
    );

    assert!(proj.is_none(), "Unarmed building should not fire");
}

#[test]
fn test_multi_branch_concurrent_research() {
    let config = crate::config::GameConfig::default();
    let mut p_tech = TechTreeState::default();
    let mut p_eco = PlayerEconomy {
        minerals: 1000.0,
        energy: 1000.0,
        supply: 4,
        max_supply: 10,
        minerals_per_sec: 0.0,
        energy_per_sec: 2.0,
    };
    let mut buildings = Vec::new();
    let mut units = Vec::new();

    // Bắt đầu nghiên cứu nhánh Khoa học (focused_laser)
    let ok1 = process_start_research(&mut p_tech, &mut p_eco, &config, "focused_laser");
    assert!(ok1, "Khởi động focused_laser thành công");
    assert_eq!(p_tech.active_researches.len(), 1);
    assert_eq!(p_tech.active_researches[0].tech_id, "focused_laser");

    // Cùng lúc đó, bắt đầu nghiên cứu nhánh Ma thuật (celestial_flying_swords)
    let ok2 = process_start_research(&mut p_tech, &mut p_eco, &config, "celestial_flying_swords");
    assert!(ok2, "Khởi động celestial_flying_swords cùng lúc thành công");
    assert_eq!(p_tech.active_researches.len(), 2);
    assert_eq!(
        p_tech.active_researches[1].tech_id,
        "celestial_flying_swords"
    );

    // Cố gắng bắt đầu thêm 1 công nghệ cùng nhánh Khoa học (dark_matter_rockets) -> Bị chặn
    let ok3 = process_start_research(&mut p_tech, &mut p_eco, &config, "dark_matter_rockets");
    assert!(!ok3, "Không được bắt đầu 2 công nghệ cùng nhánh Khoa học khi MAX_CONCURRENT_RESEARCH_PER_BRANCH = 1");
    assert_eq!(p_tech.active_researches.len(), 2);

    // Tiến hành nghiên cứu 15 giây (chưa tech nào xong: 20s và 22s)
    let completed_early = process_update_tech_research(
        Team::Player,
        &mut p_tech,
        &mut buildings,
        &mut units,
        &config,
        15.0,
    );
    assert!(completed_early.is_empty());
    assert!(!p_tech.focused_laser);
    assert!(!p_tech.celestial_flying_swords);

    // Tiến hành nghiên cứu thêm 10 giây nữa (tổng 25s, cả 2 đều xong)
    let completed_done = process_update_tech_research(
        Team::Player,
        &mut p_tech,
        &mut buildings,
        &mut units,
        &config,
        10.0,
    );
    assert_eq!(completed_done.len(), 2);
    assert!(p_tech.focused_laser, "Focused laser phải được mở khóa");
    assert!(
        p_tech.celestial_flying_swords,
        "Celestial flying swords phải được mở khóa"
    );
    assert_eq!(p_tech.active_researches.len(), 0);
}

#[test]
fn test_building_under_attack_slows_down() {
    // Tiến độ 1.0s bình thường không có thợ
    let progress_normal = calculate_building_construction_step(0.0, 0.0, false, 1.0);
    let expected_normal = crate::config::BUILD_SPEED_BASE * 1.0;
    assert!((progress_normal - expected_normal).abs() < 1e-4);

    // Tiến độ 1.0s khi đang bị tấn công (under_attack_timer = 5.0)
    let progress_attacked = calculate_building_construction_step(0.0, 5.0, false, 1.0);
    let expected_slowed = crate::config::BUILD_SPEED_BASE
        * crate::config::BUILDING_UNDER_ATTACK_SPEED_MULTIPLIER
        * 1.0;
    assert!(
        (progress_attacked - expected_slowed).abs() < 1e-4,
        "Tốc độ xây phải giảm còn 30% khi bị tấn công"
    );

    // Tiến độ 1.0s khi có thợ xây nhưng bị tấn công
    let progress_worker_attacked = calculate_building_construction_step(0.0, 3.5, true, 1.0);
    let expected_worker_slowed = crate::config::BUILD_SPEED_WORKER
        * crate::config::BUILDING_UNDER_ATTACK_SPEED_MULTIPLIER
        * 1.0;
    assert!(
        (progress_worker_attacked - expected_worker_slowed).abs() < 1e-4,
        "Tốc độ xây thợ phải giảm còn 30% khi bị tấn công"
    );
}

#[test]
fn test_vanguard_shield_regeneration_and_absorption() {
    let cfg = crate::config::GameConfig::default();
    let mut vanguard = cfg.create_unit(1, Team::Player, UnitType::Vanguard, 100.0, 100.0);
    assert_eq!(vanguard.shield, 100.0);
    assert_eq!(vanguard.max_shield, 100.0);

    // Chịu 60 sát thương -> trừ vào khiên, máu nguyên vẹn
    apply_damage_to_unit(&mut vanguard, 60.0);
    assert_eq!(vanguard.shield, 40.0);
    assert_eq!(vanguard.hp, vanguard.max_hp);

    // Chịu thêm 70 sát thương -> khiên hết (40), thừa 30 trừ vào máu
    apply_damage_to_unit(&mut vanguard, 70.0);
    assert_eq!(vanguard.shield, 0.0);
    assert_eq!(vanguard.hp, vanguard.max_hp - 30.0);

    // Giả lập trôi qua 5s để hồi đầy khiên
    vanguard.shield_timer = 4.9;
    let mut engine = crate::GameEngine::new_headless();
    engine.units.clear();
    engine.units.push(vanguard);
    update_units(&mut engine, 0.2); // 4.9 + 0.2 = 5.1s -> hồi lại 100 khiên
    let v = &engine.units[0];
    assert_eq!(v.shield, 100.0);
    assert_eq!(v.max_shield, 100.0);
}

#[test]
fn test_laser_marksman_ramping_damage_and_reset() {
    let cfg = crate::config::GameConfig::default();
    let mut engine = crate::GameEngine::new_headless();
    engine.units.clear();
    engine.buildings.clear();
    engine.projectiles.clear();

    // LaserMarksman của Player
    let mut laser = cfg.create_unit(1, Team::Player, UnitType::LaserMarksman, 100.0, 100.0);
    // Mục tiêu 1: Dummy địch (Engineer không chủ động bắn trả)
    let target1 = cfg.create_unit(2, Team::Enemy, UnitType::Engineer, 150.0, 100.0);
    // Mục tiêu 2: Dummy địch khác
    let target2 = cfg.create_unit(3, Team::Enemy, UnitType::Engineer, 180.0, 100.0);

    laser.order = UnitOrder::AttackTarget {
        target_id: 2,
        is_building: false,
    };
    engine.units.push(laser);
    engine.units.push(target1);
    engine.units.push(target2);

    // Bắn phát 1 vào target 2
    update_units(&mut engine, 0.05);
    assert_eq!(engine.projectiles.len(), 1);
    assert_eq!(engine.projectiles[0].damage, 1.0); // Sát thương khởi đầu = 1
    assert_eq!(engine.units[0].laser_stacks, 0);
    engine.projectiles.clear();

    // Qua 0.2s, bắn phát 2 cùng mục tiêu -> tăng 2 sát thương = 3
    engine.units[0].current_cooldown = 0.0;
    update_units(&mut engine, 0.05);
    assert_eq!(engine.projectiles.len(), 1);
    assert_eq!(engine.projectiles[0].damage, 3.0);
    assert_eq!(engine.units[0].laser_stacks, 1);
    engine.projectiles.clear();

    // Qua 0.2s, bắn phát 3 cùng mục tiêu -> tăng 2 sát thương = 5
    engine.units[0].current_cooldown = 0.0;
    update_units(&mut engine, 0.05);
    assert_eq!(engine.projectiles.len(), 1);
    assert_eq!(engine.projectiles[0].damage, 5.0);
    assert_eq!(engine.units[0].laser_stacks, 2);
    engine.projectiles.clear();

    // Đổi mục tiêu sang target 3 -> kích hoạt làm nguội 1s, chưa bắn ngay
    engine.units[0].order = UnitOrder::AttackTarget {
        target_id: 3,
        is_building: false,
    };
    update_units(&mut engine, 0.05);
    // Trong thời gian làm nguội 1s, chưa bắn ra đạn và không vẽ tia laser
    assert_eq!(engine.projectiles.len(), 0);
    assert!(engine.units[0].current_cooldown > 0.0);
    assert_eq!(engine.units[0].laser_stacks, 0);
    assert_eq!(engine.units[0].last_attack_pos, None);

    // Chờ hết 1s làm nguội (khoảng 20 ticks 0.05s)
    for _ in 0..20 {
        update_units(&mut engine, 0.05);
        if !engine.projectiles.is_empty() {
            break;
        }
    }

    // Đã bắn phát đầu tiên vào target 3 sau 1s làm nguội: sát thương reset về 1, stack = 0
    assert_eq!(engine.projectiles.len(), 1);
    assert_eq!(engine.projectiles[0].damage, 1.0);
    assert_eq!(engine.units[0].laser_stacks, 0);
    assert!(engine.units[0].last_attack_pos.is_some());
    engine.projectiles.clear();

    // Phát bắn tiếp theo vào target 3: tăng sát thương lên 3, stack = 1
    engine.units[0].current_cooldown = 0.0;
    update_units(&mut engine, 0.05);
    assert_eq!(engine.projectiles.len(), 1);
    assert_eq!(engine.projectiles[0].damage, 3.0);
    assert_eq!(engine.units[0].laser_stacks, 1);
    engine.projectiles.clear();

    // KIỂM TRA KHÓA MỤC TIÊU: AI/hệ thống gán AttackMove, nhưng target 3 còn sống & trong tầm
    // LaserMarksman vẫn giữ nguyên khóa mục tiêu target 3 và không bị reset stack!
    engine.units[0].order = UnitOrder::AttackMove { x: 500.0, y: 500.0 };
    update_units(&mut engine, 0.05);
    assert_eq!(
        engine.units[0].order,
        UnitOrder::AttackTarget {
            target_id: 3,
            is_building: false
        }
    );
    assert_eq!(engine.units[0].laser_stacks, 1);

    // KIỂM TRA MỤC TIÊU BỊ TIÊU DIỆT: khi target 3 chết, khóa mục tiêu bị hủy và làm nguội 1s
    if let Some(t3) = engine.units.iter_mut().find(|u| u.id == 3) {
        t3.hp = 0.0;
    }
    update_units(&mut engine, 0.05);
    assert_eq!(engine.units[0].laser_target_id, None);
    assert_eq!(engine.units[0].laser_stacks, 0);
    assert_eq!(engine.units[0].last_attack_pos, None);
    assert!(engine.units[0].current_cooldown >= 0.9);
}

#[test]
fn test_laser_marksman_out_of_range_unlocks_and_cooldown() {
    let cfg = crate::config::GameConfig::default();
    let mut engine = crate::GameEngine::new_headless();
    engine.units.clear();
    engine.buildings.clear();
    engine.projectiles.clear();

    let mut laser = cfg.create_unit(1, Team::Player, UnitType::LaserMarksman, 100.0, 100.0);
    let target = cfg.create_unit(2, Team::Enemy, UnitType::Engineer, 150.0, 100.0);

    laser.order = UnitOrder::AttackTarget {
        target_id: 2,
        is_building: false,
    };
    engine.units.push(laser);
    engine.units.push(target);

    // Bắn 1 phát vào target -> khóa mục tiêu
    update_units(&mut engine, 0.05);
    assert_eq!(engine.units[0].laser_target_id, Some((2, false)));
    assert!(engine.units[0].last_attack_pos.is_some());
    engine.projectiles.clear();

    // Di chuyển target ra xa ngoài tầm bắn (attack_range của Laser là 220, chuyển x ra 500)
    engine.units[1].x = 500.0;
    update_units(&mut engine, 0.05);

    // Khóa mục tiêu bị hủy vì mục tiêu đã ra ngoài tầm bắn
    assert_eq!(engine.units[0].laser_target_id, None);
    assert_eq!(engine.units[0].laser_stacks, 0);
    assert_eq!(engine.units[0].last_attack_pos, None);
    assert!(engine.units[0].current_cooldown >= 0.9);
}

#[test]
fn test_taoist_swordmaster_sword_ammo_and_regen() {
    let cfg = crate::config::GameConfig::default();
    let mut engine = crate::GameEngine::new_headless();
    engine.units.clear();
    engine.buildings.clear();
    engine.projectiles.clear();

    let mut swordmaster =
        cfg.create_unit(1, Team::Player, UnitType::TaoistSwordmaster, 100.0, 100.0);
    assert_eq!(swordmaster.flying_swords, 7);

    let target = cfg.create_unit(2, Team::Enemy, UnitType::Engineer, 130.0, 100.0);
    swordmaster.order = UnitOrder::AttackTarget {
        target_id: 2,
        is_building: false,
    };

    engine.units.push(swordmaster);
    engine.units.push(target);

    // Bắn 1 phát -> tiêu hao 1 kiếm, còn 6 kiếm, cooldown = 0.1
    update_units(&mut engine, 0.05);
    assert_eq!(engine.units[0].flying_swords, 6);
    assert_eq!(engine.units[0].current_cooldown, 0.1);
    engine.projectiles.clear();

    // Bắn hết toàn bộ kiếm còn lại (6 phát)
    for _ in 0..6 {
        engine.units[0].current_cooldown = 0.0;
        update_units(&mut engine, 0.05);
        engine.projectiles.clear();
    }
    assert_eq!(engine.units[0].flying_swords, 0);

    // Khi hết phi kiếm, cooldown = 0 nhưng không bắn được
    engine.units[0].current_cooldown = 0.0;
    update_units(&mut engine, 0.05);
    assert_eq!(engine.projectiles.len(), 0);

    // Đặt trạng thái Idle để kiếm hồi mà không bị bắn ngay lập tức
    engine.units[0].order = UnitOrder::Idle;
    // Trôi qua 1s -> hồi lại 1 phi kiếm
    update_units(&mut engine, 1.1);
    assert_eq!(engine.units[0].flying_swords, 1);
}

#[test]
fn test_void_warlock_summons_persistent_magic_circles() {
    let cfg = crate::config::GameConfig::default();
    let mut engine = crate::GameEngine::new_headless();
    engine.units.clear();
    engine.buildings.clear();
    engine.projectiles.clear();
    engine.magic_circles.clear();

    let mut warlock = cfg.create_unit(1, Team::Player, UnitType::VoidWarlock, 100.0, 100.0);
    let target = cfg.create_unit(2, Team::Enemy, UnitType::Vanguard, 200.0, 100.0);
    let initial_target_total = target.hp + target.shield;
    let target_max_hp = target.max_hp;
    let target_resistances = target.resistances;

    warlock.order = UnitOrder::AttackTarget {
        target_id: 2,
        is_building: false,
    };
    engine.units.push(warlock);
    engine.units.push(target);

    // Warlock triệu hồi vòng ma thuật tại vị trí target
    update_units(&mut engine, 0.05);
    assert_eq!(engine.magic_circles.len(), 1);
    assert_eq!(engine.magic_circles[0].x, 200.0);
    assert_eq!(engine.magic_circles[0].y, 100.0);
    assert_eq!(engine.magic_circles[0].lifetime, 5.0);
    assert_eq!(engine.projectiles.len(), 0); // Không bắn đạn thường

    // Tiêu diệt VoidWarlock (chỉ còn target ở index 0)
    engine.units.retain(|u| u.id == 2);
    assert_eq!(engine.units.len(), 1);

    // Vòng ma thuật vẫn tồn tại và thiêu đốt 10% max HP mỗi giây trong 1s (tính qua kháng Arcane)
    update_magic_circles(&mut engine, 1.0);
    assert_eq!(engine.magic_circles.len(), 1);
    let expected_dmg = calculate_effective_damage(
        target_max_hp * 0.10,
        DamageType::Arcane,
        &target_resistances,
    );
    let current_target_total = engine.units[0].hp + engine.units[0].shield;
    let actual_dmg = initial_target_total - current_target_total;
    assert!((actual_dmg - expected_dmg).abs() < 1.0);
}

#[test]
fn test_void_warlock_custom_config_percent_damage() {
    let mut cfg = crate::config::GameConfig::default();
    if let Some(u) = cfg.units.get_mut("VoidWarlock") {
        u.percent_max_hp_dps = 0.25; // Tùy chỉnh lên 25% max HP mỗi giây
    }
    let mut engine = crate::GameEngine::new_headless();
    engine.config = cfg.clone();
    engine.units.clear();
    engine.buildings.clear();
    engine.projectiles.clear();
    engine.magic_circles.clear();

    let mut warlock = cfg.create_unit(1, Team::Player, UnitType::VoidWarlock, 100.0, 100.0);
    let target = cfg.create_unit(2, Team::Enemy, UnitType::Vanguard, 200.0, 100.0);
    let initial_target_total = target.hp + target.shield;
    let target_max_hp = target.max_hp;
    let target_resistances = target.resistances;

    warlock.order = UnitOrder::AttackTarget {
        target_id: 2,
        is_building: false,
    };
    engine.units.push(warlock);
    engine.units.push(target);

    update_units(&mut engine, 0.05);
    assert_eq!(engine.magic_circles.len(), 1);
    assert_eq!(engine.magic_circles[0].damage_pct_per_sec, 0.25);

    // Tiêu diệt VoidWarlock (chỉ còn target ở index 0)
    engine.units.retain(|u| u.id == 2);
    update_magic_circles(&mut engine, 1.0);

    let expected_dmg = calculate_effective_damage(
        target_max_hp * 0.25,
        DamageType::Arcane,
        &target_resistances,
    );
    let current_target_total = engine.units[0].hp + engine.units[0].shield;
    let actual_dmg = initial_target_total - current_target_total;
    assert!((actual_dmg - expected_dmg).abs() < 1.0);
}

#[test]
fn test_void_warlock_magic_circle_corrosive_and_building_effects() {
    let cfg = crate::config::GameConfig::default();
    let mut engine = crate::GameEngine::new_headless();
    engine.units.clear();
    engine.buildings.clear();
    engine.projectiles.clear();
    engine.magic_circles.clear();

    // Kích hoạt Lời nguyền ăn mòn cho Team Player
    engine.player_tech.corrosive_curse = true;

    let target_unit = cfg.create_unit(1, Team::Enemy, UnitType::Rifleman, 200.0, 200.0);
    let init_arcane_res = target_unit.resistances.arcane;
    engine.units.push(target_unit);

    let target_building = cfg.create_building(
        2,
        Team::Enemy,
        BuildingType::SupplyDepot,
        210.0,
        200.0,
        true,
    );
    let b_init_hp = target_building.hp;
    engine.buildings.push(target_building);

    // Đặt 1 vòng ma thuật của Player tại (200, 200)
    engine.magic_circles.push(crate::components::MagicCircle {
        id: 1,
        team: Team::Player,
        x: 200.0,
        y: 200.0,
        radius: 60.0,
        lifetime: 5.0,
        max_lifetime: 5.0,
        tick_timer: 0.0,
        damage_pct_per_sec: 0.10,
        damage_type: DamageType::Arcane,
    });

    // Chạy 0.6s để kích hoạt tick text và hiệu ứng đòn đánh
    update_magic_circles(&mut engine, 0.6);

    // 1. Kiểm tra Lời nguyền ăn mòn (Corrosive Curse) làm giảm chỉ số kháng
    assert!(engine.units[0].resistances.arcane < init_arcane_res);

    // 2. Kiểm tra công trình bị trừ máu
    assert!(engine.buildings[0].hp < b_init_hp);

    // 3. Kiểm tra công trình kích hoạt hiệu ứng bị tấn công (under_attack_timer)
    assert_eq!(
        engine.buildings[0].under_attack_timer,
        crate::config::BUILDING_UNDER_ATTACK_DURATION
    );
}

#[test]
fn test_acid_baneling_double_damage_to_buildings() {
    let cfg = crate::config::GameConfig::default();
    let mut engine = crate::GameEngine::new_headless();
    engine.units.clear();
    engine.buildings.clear();
    engine.projectiles.clear();

    let mut baneling = cfg.create_unit(1, Team::Player, UnitType::AcidBaneling, 100.0, 100.0);
    let mut building = cfg.create_building(
        2,
        Team::Enemy,
        BuildingType::SupplyDepot,
        110.0,
        100.0,
        true,
    );
    building.resistances.explosive = 0.0; // Để so sánh trực tiếp sát thương thuần
    let b_initial_hp = building.hp;

    baneling.order = UnitOrder::AttackTarget {
        target_id: 2,
        is_building: true,
    };
    engine.units.push(baneling);
    engine.buildings.push(building);

    // Baneling tự nổ
    update_units(&mut engine, 0.05);
    assert_eq!(engine.projectiles.len(), 1);
    let p = &engine.projectiles[0];
    assert_eq!(p.building_damage_mult, 2.0);
    assert_eq!(p.damage, 90.0); // Sát thương gốc

    // Đạn nổ áp sát thương vào công trình
    update_projectiles(&mut engine, 0.1);
    // Sát thương nhận phải = 90.0 * 2.0 = 180.0
    let damage_taken = b_initial_hp - engine.buildings[0].hp;
    assert_eq!(damage_taken, 180.0);
}

#[test]
fn test_bot_resource_multiplier_by_difficulty() {
    let mut engine = crate::GameEngine::new_headless();

    // 1. Chế độ Tân binh (easy)
    engine.difficulty = "easy".to_string();
    assert_eq!(engine.bot_resource_multiplier(Team::Player), 1.0);
    assert_eq!(engine.bot_resource_multiplier(Team::Enemy), 0.8);
    assert_eq!(engine.bot_resource_multiplier(Team::Player2), 0.8);

    // 2. Chế độ Tiêu chuẩn (normal)
    engine.difficulty = "normal".to_string();
    assert_eq!(engine.bot_resource_multiplier(Team::Player), 1.0);
    assert_eq!(engine.bot_resource_multiplier(Team::Enemy), 1.0);
    assert_eq!(engine.bot_resource_multiplier(Team::Player2), 1.0);

    // 3. Chế độ Chuyên gia (hard)
    engine.difficulty = "hard".to_string();
    assert_eq!(engine.bot_resource_multiplier(Team::Player), 1.0);
    assert_eq!(engine.bot_resource_multiplier(Team::Enemy), 1.2);
    assert_eq!(engine.bot_resource_multiplier(Team::Player2), 1.2);
}

#[test]
fn test_ranged_units_projectile_visuals() {
    let cfg = crate::config::GameConfig::default();
    let mut engine = crate::GameEngine::new_headless();
    engine.units.clear();
    engine.buildings.clear();
    engine.projectiles.clear();

    // Target là 1 Dummy ở (200, 100)
    let target = cfg.create_unit(99, Team::Enemy, UnitType::Engineer, 200.0, 100.0);
    engine.units.push(target);

    // Test 1: Rifleman -> Bullet
    let mut rifleman = cfg.create_unit(1, Team::Player, UnitType::Rifleman, 100.0, 100.0);
    rifleman.order = UnitOrder::AttackTarget {
        target_id: 99,
        is_building: false,
    };
    engine.units.push(rifleman);
    update_units(&mut engine, 0.05);
    assert_eq!(engine.projectiles.len(), 1);
    assert_eq!(engine.projectiles[0].visual, ProjectileVisual::Bullet);
    assert!(!engine.projectiles[0].is_melee);
    engine.projectiles.clear();
    engine.units.retain(|u| u.id == 99);

    // Test 2: DarkRocketeer -> Rocket
    let mut rocketeer = cfg.create_unit(2, Team::Player, UnitType::DarkRocketeer, 100.0, 100.0);
    rocketeer.order = UnitOrder::AttackTarget {
        target_id: 99,
        is_building: false,
    };
    engine.units.push(rocketeer);
    update_units(&mut engine, 0.05);
    assert_eq!(engine.projectiles.len(), 1);
    assert_eq!(engine.projectiles[0].visual, ProjectileVisual::Rocket);
    assert_eq!(engine.projectiles[0].damage_type, DamageType::Explosive);
    engine.projectiles.clear();
    engine.units.retain(|u| u.id == 99);

    // Test 3: TaoistSwordmaster -> FlyingSword
    let mut swordmaster =
        cfg.create_unit(3, Team::Player, UnitType::TaoistSwordmaster, 100.0, 100.0);
    swordmaster.order = UnitOrder::AttackTarget {
        target_id: 99,
        is_building: false,
    };
    engine.units.push(swordmaster);
    update_units(&mut engine, 0.05);
    assert_eq!(engine.projectiles.len(), 1);
    assert_eq!(engine.projectiles[0].visual, ProjectileVisual::FlyingSword);
    assert_eq!(engine.projectiles[0].damage_type, DamageType::Arcane);
}

#[test]
fn test_unit_prev_position_and_interpolation() {
    let mut engine = GameEngine::new_headless();
    engine.units.clear();
    engine.is_paused = false;
    let cfg = GameConfig::default();
    let mut unit = cfg.create_unit(1, Team::Player, UnitType::Rifleman, 200.0, 200.0);
    unit.order = UnitOrder::MoveTo { x: 500.0, y: 200.0 };
    engine.units.push(unit);

    assert_eq!(engine.units[0].prev_x, 200.0);
    assert_eq!(engine.units[0].x, 200.0);

    // Chạy 1 step mô phỏng
    step(&mut engine, 0.1);

    // prev_x phải lưu lại vị trí ban đầu (200.0), trong khi x đã tăng lên vì di chuyển sang phải
    assert_eq!(engine.units[0].prev_x, 200.0);
    assert!(engine.units[0].x > 200.0);

    // Kiểm tra tính toán nội suy
    let alpha = 0.5;
    let interp_x = engine.units[0].prev_x + (engine.units[0].x - engine.units[0].prev_x) * alpha;
    assert!(interp_x > 200.0 && interp_x < engine.units[0].x);
}

#[test]
fn test_fog_of_war_is_explored_and_visible() {
    let mut engine = GameEngine::new_headless();
    // Vị trí (1000, 1000) ban đầu chưa được khám phá
    assert!(!engine.fog.is_visible(1000.0, 1000.0));
    assert!(!engine.fog.is_explored(1000.0, 1000.0));

    // Mở sương mù tại (1000, 1000) với bán kính 100.0
    engine.fog.reveal_circle(1000.0, 1000.0, 100.0);
    assert!(engine.fog.is_visible(1000.0, 1000.0));
    assert!(engine.fog.is_explored(1000.0, 1000.0));

    // Chuyển tầm nhìn về Explored
    engine.fog.reset_visibility_to_explored();
    assert!(!engine.fog.is_visible(1000.0, 1000.0));
    assert!(engine.fog.is_explored(1000.0, 1000.0));
}

#[test]
fn test_particle_pool_and_lifecycle() {
    let mut engine = GameEngine::new_headless();
    engine.is_headless = false; // Bật để test sinh hạt
    assert_eq!(engine.particles.len(), 0);

    engine.spawn_particles(100.0, 100.0, 20, "#f97316", 100.0, 2.5, 0.4);
    assert_eq!(engine.particles.len(), 20);

    // Test giới hạn pool 400
    for _ in 0..30 {
        engine.spawn_particles(100.0, 100.0, 20, "#f97316", 100.0, 2.5, 0.4);
    }
    assert!(engine.particles.len() <= 400);

    // Cập nhật hạt suy giảm thời gian sống
    for p in &mut engine.particles {
        p.lifetime = 0.0;
    }
    engine.particles.retain(|p| p.lifetime > 0.0);
    assert_eq!(engine.particles.len(), 0);
}

#[test]
fn test_hit_flash() {
    let cfg = GameConfig::default();
    let mut unit = cfg.create_unit(1, Team::Player, UnitType::Rifleman, 200.0, 200.0);

    assert_eq!(unit.hit_flash_timer, 0.0);
    apply_damage_to_unit(&mut unit, 10.0);
    assert!(unit.hit_flash_timer > 0.0);
}

#[test]
fn test_high_load_8_teams_simulation_performance() {
    let mut engine = GameEngine::new_headless();
    engine.units.clear();
    engine.buildings.clear();
    for i in 0..8 {
        engine.player_slots[i].active = true;
    }

    // Spawn 80 units per team (640 units total) scattered across map
    let cfg = GameConfig::default();
    let mut uid = 100;
    for team_idx in 0..8 {
        let team = Team::from_index(team_idx);
        let center_x = 500.0 + (team_idx as f32 % 4.0) * 1000.0;
        let center_y = 500.0 + (team_idx as f32 / 4.0) * 2000.0;

        // Base CC
        engine.buildings.push(cfg.create_building(
            uid,
            team,
            BuildingType::CommandCenter,
            center_x,
            center_y,
            true,
        ));
        uid += 1;

        for j in 0..80 {
            let offset_x = (j % 10) as f32 * 35.0;
            let offset_y = (j / 10) as f32 * 35.0;
            let u_type = match j % 4 {
                0 => UnitType::Rifleman,
                1 => UnitType::ShockTrooper,
                2 => UnitType::LaserMarksman,
                _ => UnitType::Engineer,
            };
            let mut u =
                cfg.create_unit(uid, team, u_type, center_x + offset_x, center_y + offset_y);
            // Issue orders to simulate real battle load
            if j % 2 == 0 {
                u.order = UnitOrder::AttackMove {
                    x: 2400.0,
                    y: 2400.0,
                };
            } else {
                u.order = UnitOrder::MoveTo {
                    x: 2400.0,
                    y: 2400.0,
                };
            }
            engine.units.push(u);
            uid += 1;
        }
    }

    assert_eq!(engine.units.len(), 640);
    assert_eq!(engine.buildings.len(), 8);

    // Run 60 simulation steps (1 full second at 60Hz)
    let start = std::time::Instant::now();
    for _ in 0..60 {
        crate::simulation::step(&mut engine, 1.0 / 60.0);
    }
    let elapsed = start.elapsed();

    // Verify simulation completed and runs within high performance threshold
    let threshold_ms = if cfg!(debug_assertions) { 2000 } else { 500 };
    assert!(
        elapsed.as_millis() < threshold_ms,
        "High-load 640-unit 8-team simulation took too long: {:?} (threshold: {}ms)",
        elapsed,
        threshold_ms
    );
}

#[test]
fn test_hud_canvas_content_visibility() {
    let mut engine = GameEngine::new_headless();
    let cfg = GameConfig::default();

    // 1. Khi chưa chọn gì -> Cả hai canvas đều ẩn
    assert!(!engine.has_selection_info_content());
    assert!(!engine.has_command_card_content());

    // 2. Chọn lính Player -> Cả hai canvas đều hiện
    let player_unit_id = engine.units.iter().find(|u| u.team == Team::Player).unwrap().id;
    engine.selected_unit_ids = vec![player_unit_id];
    assert!(engine.has_selection_info_content());
    assert!(engine.has_command_card_content());

    // 3. Chọn lính Địch -> Selection Info hiện, Command Card ẩn
    let enemy_unit_id = engine.units.iter().find(|u| u.team == Team::Enemy).unwrap().id;
    engine.selected_unit_ids = vec![enemy_unit_id];
    assert!(engine.has_selection_info_content());
    assert!(!engine.has_command_card_content());

    // 4. Chọn nhà chính Player (CommandCenter) -> Cả hai canvas đều hiện
    engine.selected_unit_ids.clear();
    let player_cc_id = engine
        .buildings
        .iter()
        .find(|b| b.team == Team::Player && b.building_type == BuildingType::CommandCenter)
        .unwrap()
        .id;
    engine.selected_building_id = Some(player_cc_id);
    assert!(engine.has_selection_info_content());
    assert!(engine.has_command_card_content());

    // 5. Chọn Trụ thủ Player (TurretTower) -> Selection Info hiện, Command Card ẩn
    let turret_id = engine.alloc_id();
    let b_turret = cfg.create_building(turret_id, Team::Player, BuildingType::TurretTower, 400.0, 400.0, true);
    engine.buildings.push(b_turret);
    engine.selected_building_id = Some(turret_id);
    assert!(engine.has_selection_info_content());
    assert!(!engine.has_command_card_content());

    // 6. Xóa chọn -> Cả hai canvas lại ẩn
    engine.selected_building_id = None;
    assert!(!engine.has_selection_info_content());
    assert!(!engine.has_command_card_content());
}
