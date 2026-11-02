use super::state::find_primary_hostile_team;
use crate::components::*;
use crate::GameEngine;

pub fn execute_team_continuous_recruitment(engine: &mut GameEngine, team: Team) {
    let (supply, max_supply) = {
        let eco = engine.team_eco(team);
        (eco.supply, eco.max_supply)
    };
    if supply >= max_supply {
        return;
    }

    let team_combat = engine
        .units
        .iter()
        .filter(|u| u.team == team && u.unit_type != UnitType::Engineer)
        .count();
    let tech = engine.team_tech(team);
    let total_unlocked = super::state::count_unlocked_techs(tech);

    // Kiểm tra căn cứ có đang bị đe dọa không
    let base_pos = engine.base_pos(team);
    let base_threat = engine.units.iter().any(|u| {
        engine.are_hostile(team, u.team)
            && ((u.x - base_pos.0).powi(2) + (u.y - base_pos.1).powi(2)) < 800.0_f32.powi(2)
    });

    // Dồn tài nguyên nâng cấp công nghệ & mở rộng kinh tế:
    // Nếu chưa xong toàn bộ công nghệ (< 19) và căn cứ không bị đe dọa, duy trì quỹ dự trữ lớn
    let (mineral_reserve, energy_reserve) = if base_threat {
        (0.0, 0.0)
    } else if total_unlocked < 12 {
        (250.0, 150.0)
    } else if total_unlocked < 19 {
        (180.0, 100.0)
    } else if team_combat >= 24 {
        (100.0, 50.0)
    } else {
        (0.0, 0.0)
    };

    let max_queue = if total_unlocked < 12 && !base_threat {
        1
    } else {
        2
    };

    let completed_barracks: Vec<u32> = engine
        .buildings
        .iter()
        .filter(|b| {
            b.team == team
                && b.building_type == BuildingType::Barracks
                && b.build_progress >= 1.0
                && b.training_queue.len() < max_queue
        })
        .map(|b| b.id)
        .collect();

    let count_unit = |ut: UnitType| -> usize {
        engine
            .units
            .iter()
            .filter(|u| u.team == team && u.unit_type == ut)
            .count()
    };
    let titans = count_unit(UnitType::BehemothTitan);
    let rockets = count_unit(UnitType::DarkRocketeer);
    let lasers = count_unit(UnitType::LaserMarksman);
    let warlocks = count_unit(UnitType::VoidWarlock);
    let swords = count_unit(UnitType::TaoistSwordmaster);
    let shields = count_unit(UnitType::Vanguard);
    let banelings = count_unit(UnitType::AcidBaneling);

    let eco = engine.team_eco(team);
    let baneling_quota = if eco.minerals >= 450.0 && eco.energy >= 200.0 {
        2
    } else {
        1
    };

    // Sắp xếp thứ tự ưu tiên: Binh chủng chủ lực đứng trước, AcidBaneling đứng cuối với quota thấp
    let mut candidate_order = vec![
        (UnitType::BehemothTitan, titans, 4, 1),
        (UnitType::DarkRocketeer, rockets, 6, 2),
        (UnitType::LaserMarksman, lasers, 5, 3),
        (UnitType::VoidWarlock, warlocks, 4, 4),
        (UnitType::TaoistSwordmaster, swords, 5, 5),
        (UnitType::Vanguard, shields, 4, 6),
        (UnitType::AcidBaneling, banelings, baneling_quota, 10),
    ];
    candidate_order.sort_by_key(|&(_, cur, quota, prio)| {
        if cur >= quota {
            1000 + cur * 10 + prio
        } else {
            prio * 100 + cur
        }
    });

    for bid in completed_barracks {
        let (cur_supply, cur_max_supply) = {
            let eco = engine.team_eco(team);
            (eco.supply, eco.max_supply)
        };
        if cur_supply >= cur_max_supply {
            break;
        }

        let mut recruited = false;
        let tech = engine.team_tech(team).clone();

        for &(u_type, cur_cnt, quota, _) in &candidate_order {
            if !crate::autoplay::can_train_unit(&tech, u_type) {
                continue;
            }
            // Hạn chế chặt chẽ AcidBaneling: chỉ mua khi cực kỳ dư dả, không spam làm cạn kiệt tài nguyên
            if u_type == UnitType::AcidBaneling {
                let eco = engine.team_eco(team);
                if cur_cnt >= quota || (cur_cnt >= 1 && eco.minerals < 350.0) {
                    continue;
                }
            }

            if cur_cnt >= quota
                && candidate_order
                    .iter()
                    .any(|&(other_u, other_c, other_q, _)| {
                        other_c < other_q && crate::autoplay::can_train_unit(&tech, other_u)
                    })
            {
                continue;
            }

            let (mut min_cost, mut en_cost) = engine.config.get_unit_cost(u_type);
            if tech.hive_proliferation {
                min_cost *= 0.65;
                en_cost *= 0.65;
            }

            let eco = engine.team_eco_mut(team);
            if eco.minerals >= mineral_reserve + min_cost && eco.energy >= energy_reserve + en_cost
            {
                eco.minerals -= min_cost;
                eco.energy -= en_cost;
                if let Some(b) = engine.buildings.iter_mut().find(|b| b.id == bid) {
                    b.training_queue.push(u_type);
                    recruited = true;
                    break;
                }
            }
        }

        // Chỉ mua lính cơ bản nếu căn cứ đang bị đe dọa hoặc đã xong phần lớn công nghệ
        if !recruited && (base_threat || total_unlocked >= 12 || team_combat < 6) {
            let basic_units = [UnitType::ShockTrooper, UnitType::Rifleman];
            for &u_type in &basic_units {
                let (mut min_cost, _) = engine.config.get_unit_cost(u_type);
                if tech.hive_proliferation {
                    min_cost *= 0.65;
                }
                let eco = engine.team_eco_mut(team);
                if eco.minerals >= mineral_reserve + min_cost {
                    eco.minerals -= min_cost;
                    if let Some(b) = engine.buildings.iter_mut().find(|b| b.id == bid) {
                        b.training_queue.push(u_type);
                        break;
                    }
                }
            }
        }
    }
}

pub fn execute_team_eco(engine: &mut GameEngine, team: Team, try_expansion: bool) {
    let team_workers = engine
        .units
        .iter()
        .filter(|u| u.team == team && u.unit_type == UnitType::Engineer)
        .count();
    let team_ccs = engine
        .buildings
        .iter()
        .filter(|b| b.team == team && b.building_type == BuildingType::CommandCenter)
        .count();
    let worker_target = (12 + team_ccs * 6).min(36);

    let (supply, max_supply, minerals) = {
        let eco = engine.team_eco(team);
        (eco.supply, eco.max_supply, eco.minerals)
    };

    if team_workers < worker_target && supply < max_supply {
        let (w_cost, _) = engine.config.get_unit_cost(UnitType::Engineer);
        if minerals >= w_cost {
            let cc_id = engine
                .buildings
                .iter()
                .find(|b| {
                    b.team == team
                        && b.building_type == BuildingType::CommandCenter
                        && b.build_progress >= 1.0
                        && b.training_queue.len() < 2
                })
                .map(|b| b.id);
            if let Some(bid) = cc_id {
                let eco = engine.team_eco_mut(team);
                eco.minerals -= w_cost;
                if let Some(cc) = engine.buildings.iter_mut().find(|b| b.id == bid) {
                    cc.training_queue.push(UnitType::Engineer);
                }
            }
        }
    }

    if try_expansion && team_ccs < 5 && minerals >= 400.0 {
        let under_construction = engine.buildings.iter().any(|b| {
            b.team == team
                && b.building_type == BuildingType::CommandCenter
                && b.build_progress < 1.0
        });
        if !under_construction {
            if let Some((bx, by)) =
                crate::autoplay::find_team_build_spot(engine, team, BuildingType::CommandCenter)
            {
                crate::autoplay::try_team_auto_build(
                    engine,
                    team,
                    BuildingType::CommandCenter,
                    bx,
                    by,
                );
            }
        }
    }
}

pub fn execute_team_infrastructure(engine: &mut GameEngine, team: Team) {
    let team_ccs = engine
        .buildings
        .iter()
        .filter(|b| b.team == team && b.building_type == BuildingType::CommandCenter)
        .count();
    let reactors = engine
        .buildings
        .iter()
        .filter(|b| b.team == team && b.building_type == BuildingType::PowerReactor)
        .count();
    let converters = engine
        .buildings
        .iter()
        .filter(|b| b.team == team && b.building_type == BuildingType::EnergyConverter)
        .count();
    let barracks = engine
        .buildings
        .iter()
        .filter(|b| b.team == team && b.building_type == BuildingType::Barracks)
        .count();
    let turrets = engine
        .buildings
        .iter()
        .filter(|b| b.team == team && b.building_type == BuildingType::TurretTower)
        .count();

    let (minerals, energy) = {
        let eco = engine.team_eco(team);
        (eco.minerals, eco.energy)
    };

    // Mở rộng thêm CommandCenter nếu có đủ điều kiện tài nguyên
    if team_ccs < 5 && minerals >= 400.0 {
        let under_construction_cc = engine.buildings.iter().any(|b| {
            b.team == team
                && b.building_type == BuildingType::CommandCenter
                && b.build_progress < 1.0
        });
        if !under_construction_cc {
            if let Some((bx, by)) =
                crate::autoplay::find_team_build_spot(engine, team, BuildingType::CommandCenter)
            {
                crate::autoplay::try_team_auto_build(
                    engine,
                    team,
                    BuildingType::CommandCenter,
                    bx,
                    by,
                );
            }
        }
    }

    if barracks < 5 {
        let (barracks_cost, _) = engine.config.get_building_cost(BuildingType::Barracks);
        if minerals >= barracks_cost {
            let under_construction = engine.buildings.iter().any(|b| {
                b.team == team
                    && b.building_type == BuildingType::Barracks
                    && b.build_progress < 1.0
            });
            if !under_construction {
                if let Some((bx, by)) =
                    crate::autoplay::find_team_build_spot(engine, team, BuildingType::Barracks)
                {
                    crate::autoplay::try_team_auto_build(
                        engine,
                        team,
                        BuildingType::Barracks,
                        bx,
                        by,
                    );
                }
            }
        }
    }

    if reactors < 36 {
        let (reactor_cost, _) = engine.config.get_building_cost(BuildingType::PowerReactor);
        if minerals >= reactor_cost {
            let under_construction = engine
                .buildings
                .iter()
                .filter(|b| {
                    b.team == team
                        && b.building_type == BuildingType::PowerReactor
                        && b.build_progress < 1.0
                })
                .count();
            let max_parallel = if minerals >= 300.0 { 2 } else { 1 };
            if under_construction < max_parallel {
                if let Some((bx, by)) =
                    crate::autoplay::find_team_build_spot(engine, team, BuildingType::PowerReactor)
                {
                    crate::autoplay::try_team_auto_build(
                        engine,
                        team,
                        BuildingType::PowerReactor,
                        bx,
                        by,
                    );
                }
            }
        }
    }

    if reactors >= 2 && converters < (reactors / 2).clamp(1, 16) {
        let (conv_m, conv_e) = engine
            .config
            .get_building_cost(BuildingType::EnergyConverter);
        if minerals >= conv_m && energy >= conv_e {
            let under_construction = engine
                .buildings
                .iter()
                .filter(|b| {
                    b.team == team
                        && b.building_type == BuildingType::EnergyConverter
                        && b.build_progress < 1.0
                })
                .count();
            let max_parallel = if minerals >= 350.0 && energy >= 200.0 {
                2
            } else {
                1
            };
            if under_construction < max_parallel {
                if let Some((bx, by)) = crate::autoplay::find_team_build_spot(
                    engine,
                    team,
                    BuildingType::EnergyConverter,
                ) {
                    crate::autoplay::try_team_auto_build(
                        engine,
                        team,
                        BuildingType::EnergyConverter,
                        bx,
                        by,
                    );
                }
            }
        }
    }

    // Bảo vệ hạ tầng bằng TurretTower
    let max_turrets = (10 + team_ccs * 6).min(32);
    if turrets < max_turrets {
        let (turret_m, turret_e) = engine.config.get_building_cost(BuildingType::TurretTower);
        if minerals >= turret_m && energy >= turret_e {
            let under_construction = engine.buildings.iter().any(|b| {
                b.team == team
                    && b.building_type == BuildingType::TurretTower
                    && b.build_progress < 1.0
            });
            if !under_construction {
                if let Some((bx, by)) =
                    crate::autoplay::find_team_build_spot(engine, team, BuildingType::TurretTower)
                {
                    crate::autoplay::try_team_auto_build(
                        engine,
                        team,
                        BuildingType::TurretTower,
                        bx,
                        by,
                    );
                }
            }
        }
    }
}

pub fn execute_team_defense(engine: &mut GameEngine, team: Team) {
    let team_ccs = engine
        .buildings
        .iter()
        .filter(|b| b.team == team && b.building_type == BuildingType::CommandCenter)
        .count();
    let turrets = engine
        .buildings
        .iter()
        .filter(|b| b.team == team && b.building_type == BuildingType::TurretTower)
        .count();
    let under_construction_turrets = engine
        .buildings
        .iter()
        .filter(|b| {
            b.team == team && b.building_type == BuildingType::TurretTower && b.build_progress < 1.0
        })
        .count();

    let max_turrets = (12 + team_ccs * 6).min(32);
    let (minerals, energy) = {
        let eco = engine.team_eco(team);
        (eco.minerals, eco.energy)
    };

    let max_parallel = if minerals >= 280.0 && energy >= 100.0 {
        2
    } else {
        1
    };

    if turrets < max_turrets && under_construction_turrets < max_parallel {
        let (turret_m, turret_e) = engine.config.get_building_cost(BuildingType::TurretTower);
        if minerals >= turret_m && energy >= turret_e {
            if let Some((bx, by)) =
                crate::autoplay::find_team_build_spot(engine, team, BuildingType::TurretTower)
            {
                crate::autoplay::try_team_auto_build(
                    engine,
                    team,
                    BuildingType::TurretTower,
                    bx,
                    by,
                );
            }
        }
    }
}

pub fn try_team_research(engine: &mut GameEngine, team: Team, tech_chain: &[&str]) {
    for &tech_id in tech_chain {
        let (needs_research, target) = {
            let team_tech = engine.team_tech(team);
            if !crate::simulation::is_tech_unlocked(team_tech, tech_id)
                && !team_tech
                    .active_researches
                    .iter()
                    .any(|r| r.tech_id == tech_id)
            {
                let mut t = tech_id.to_string();
                if let Some(req) = crate::autoplay::get_tech_prerequisite(tech_id) {
                    if !crate::simulation::is_tech_unlocked(team_tech, &req)
                        && !team_tech.active_researches.iter().any(|r| r.tech_id == req)
                    {
                        t = req;
                    }
                }
                (true, t)
            } else {
                (false, String::new())
            }
        };

        if needs_research {
            let (tech, eco, cfg) = match team {
                Team::Player => (
                    &mut engine.player_tech,
                    &mut engine.player_eco,
                    &engine.config,
                ),
                Team::Enemy => (
                    &mut engine.enemy_tech,
                    &mut engine.enemy_eco,
                    &engine.config,
                ),
                _ => {
                    let idx = team.index() - 2;
                    (
                        &mut engine.extra_techs[idx],
                        &mut engine.extra_ecos[idx],
                        &engine.config,
                    )
                }
            };
            if crate::simulation::process_start_research(tech, eco, cfg, &target) {
                return;
            }
        }
    }

    // Nếu đã hoàn thành chuỗi chỉ định, tự động nâng tiếp các công nghệ còn lại
    for &all_t in &crate::autoplay::ALL_TECH_IDS {
        let (needs_research, target) = {
            let team_tech = engine.team_tech(team);
            if !crate::simulation::is_tech_unlocked(team_tech, all_t)
                && !team_tech
                    .active_researches
                    .iter()
                    .any(|r| r.tech_id == all_t)
            {
                let mut t = all_t.to_string();
                if let Some(req) = crate::autoplay::get_tech_prerequisite(all_t) {
                    if !crate::simulation::is_tech_unlocked(team_tech, &req)
                        && !team_tech.active_researches.iter().any(|r| r.tech_id == req)
                    {
                        t = req;
                    }
                }
                (true, t)
            } else {
                (false, String::new())
            }
        };

        if needs_research {
            let (tech, eco, cfg) = match team {
                Team::Player => (
                    &mut engine.player_tech,
                    &mut engine.player_eco,
                    &engine.config,
                ),
                Team::Enemy => (
                    &mut engine.enemy_tech,
                    &mut engine.enemy_eco,
                    &engine.config,
                ),
                _ => {
                    let idx = team.index() - 2;
                    (
                        &mut engine.extra_techs[idx],
                        &mut engine.extra_ecos[idx],
                        &engine.config,
                    )
                }
            };
            if crate::simulation::process_start_research(tech, eco, cfg, &target) {
                return;
            }
        }
    }
}

pub fn execute_team_training(engine: &mut GameEngine, team: Team, candidate_units: &[UnitType]) {
    let (supply, max_supply) = {
        let eco = engine.team_eco(team);
        (eco.supply, eco.max_supply)
    };
    if supply >= max_supply {
        return;
    }

    let total_unlocked = super::state::count_unlocked_techs(engine.team_tech(team));
    let base_pos = engine.base_pos(team);
    let base_threat = engine.units.iter().any(|u| {
        engine.are_hostile(team, u.team)
            && ((u.x - base_pos.0).powi(2) + (u.y - base_pos.1).powi(2)) < 800.0_f32.powi(2)
    });
    let (mineral_reserve, energy_reserve) = if base_threat {
        (0.0, 0.0)
    } else if total_unlocked < 12 {
        (250.0, 150.0)
    } else if total_unlocked < 19 {
        (180.0, 100.0)
    } else {
        (0.0, 0.0)
    };

    let max_queue = if total_unlocked < 12 && !base_threat {
        1
    } else {
        2
    };

    let completed_barracks: Vec<u32> = engine
        .buildings
        .iter()
        .filter(|b| {
            b.team == team
                && b.building_type == BuildingType::Barracks
                && b.build_progress >= 1.0
                && b.training_queue.len() < max_queue
        })
        .map(|b| b.id)
        .collect();

    for bid in completed_barracks {
        let (cur_supply, cur_max_supply) = {
            let eco = engine.team_eco(team);
            (eco.supply, eco.max_supply)
        };
        if cur_supply >= cur_max_supply {
            break;
        }

        for &u_type in candidate_units {
            let tech = engine.team_tech(team);
            if u_type == UnitType::Engineer || !crate::autoplay::can_train_unit(tech, u_type) {
                continue;
            }

            let (mut min_cost, mut en_cost) = engine.config.get_unit_cost(u_type);
            if tech.hive_proliferation {
                min_cost *= 0.65;
                en_cost *= 0.65;
            }

            let eco = engine.team_eco_mut(team);
            if eco.minerals >= mineral_reserve + min_cost && eco.energy >= energy_reserve + en_cost
            {
                eco.minerals -= min_cost;
                eco.energy -= en_cost;
                if let Some(b) = engine.buildings.iter_mut().find(|b| b.id == bid) {
                    b.training_queue.push(u_type);
                    break;
                }
            }
        }
    }
}

pub enum MilitaryCommand {
    DefendBase,
    HarassWorkers,
    HarassOrAttack,
    AllOutAssault,
    Regroup,
    SiegePush,
    SupportAlly((f32, f32)),
    AttackSpecific((f32, f32)),
}

pub fn command_team_military(engine: &mut GameEngine, team: Team, cmd: MilitaryCommand) {
    let opp_team = find_primary_hostile_team(engine, team);

    let combat_pairs: Vec<(u32, UnitType)> = engine
        .units
        .iter()
        .filter(|u| u.team == team && u.unit_type != UnitType::Engineer)
        .map(|u| (u.id, u.unit_type))
        .collect();

    if combat_pairs.is_empty() {
        return;
    }

    let base_pos = engine.base_pos(team);
    let default_opp_base = engine.base_pos(opp_team);
    let default_opp_worker_pos = (default_opp_base.0 + 50.0, default_opp_base.1 + 50.0);

    let center = (crate::map::MAP_WIDTH * 0.5, crate::map::MAP_HEIGHT * 0.5);
    let dx = center.0 - base_pos.0;
    let dy = center.1 - base_pos.1;
    let len = (dx * dx + dy * dy).sqrt().max(1.0);
    let default_rally = (base_pos.0 + dx / len * 350.0, base_pos.1 + dy / len * 350.0);

    let form = engine.team_formation(team);

    let start_center = {
        let mut sum_x = 0.0;
        let mut sum_y = 0.0;
        for (uid, _) in &combat_pairs {
            if let Some(u) = engine.units.iter().find(|u| u.id == *uid) {
                sum_x += u.x;
                sum_y += u.y;
            }
        }
        (
            sum_x / combat_pairs.len() as f32,
            sum_y / combat_pairs.len() as f32,
        )
    };

    match cmd {
        MilitaryCommand::DefendBase => {
            let nearest_threat = engine
                .units
                .iter()
                .filter(|u| engine.are_hostile(team, u.team))
                .min_by(|a, b| {
                    let da = (a.x - base_pos.0).powi(2) + (a.y - base_pos.1).powi(2);
                    let db = (b.x - base_pos.0).powi(2) + (b.y - base_pos.1).powi(2);
                    da.partial_cmp(&db).unwrap()
                })
                .map(|u| (u.x, u.y))
                .unwrap_or(base_pos);

            let form_pos = crate::formation::compute_formation_positions(
                &combat_pairs,
                start_center,
                nearest_threat,
                form,
            );

            for (uid, tx, ty, off_x, off_y) in form_pos {
                if let Some(u) = engine.units.iter_mut().find(|u| u.id == uid) {
                    u.formation_offset = (off_x, off_y);
                    u.order = UnitOrder::AttackMove { x: tx, y: ty };
                }
            }
        }
        MilitaryCommand::HarassWorkers => {
            let opp_worker_pos = engine
                .units
                .iter()
                .find(|u| engine.are_hostile(team, u.team) && u.unit_type == UnitType::Engineer)
                .map(|u| (u.x, u.y))
                .unwrap_or(default_opp_worker_pos);

            let form_pos = crate::formation::compute_formation_positions(
                &combat_pairs,
                start_center,
                opp_worker_pos,
                form,
            );

            for (uid, tx, ty, off_x, off_y) in form_pos {
                if let Some(u) = engine.units.iter_mut().find(|u| u.id == uid) {
                    u.formation_offset = (off_x, off_y);
                    u.order = UnitOrder::AttackMove { x: tx, y: ty };
                }
            }
        }
        MilitaryCommand::HarassOrAttack | MilitaryCommand::AllOutAssault => {
            let target = engine
                .buildings
                .iter()
                .find(|b| {
                    engine.are_hostile(team, b.team)
                        && b.building_type == BuildingType::CommandCenter
                })
                .map(|b| (b.x, b.y))
                .unwrap_or(default_opp_base);

            let form_pos = crate::formation::compute_formation_positions(
                &combat_pairs,
                start_center,
                target,
                form,
            );

            for (uid, tx, ty, off_x, off_y) in form_pos {
                if let Some(u) = engine.units.iter_mut().find(|u| u.id == uid) {
                    if let UnitOrder::AttackTarget { .. } = u.order {
                        continue;
                    }
                    if let UnitOrder::AttackMove { x, y } = u.order {
                        let d_sq = (x - target.0).powi(2) + (y - target.1).powi(2);
                        if d_sq < 90.0_f32.powi(2) {
                            continue;
                        }
                    }

                    u.formation_offset = (off_x, off_y);
                    u.order = UnitOrder::AttackMove { x: tx, y: ty };
                }
            }
        }
        MilitaryCommand::SiegePush => {
            // Nhắm thẳng vào tháp pháo hoặc nhà chính đối phương
            let target = engine
                .buildings
                .iter()
                .find(|b| {
                    engine.are_hostile(team, b.team) && b.building_type == BuildingType::TurretTower
                })
                .map(|b| (b.x, b.y))
                .or_else(|| {
                    engine
                        .buildings
                        .iter()
                        .find(|b| {
                            engine.are_hostile(team, b.team)
                                && b.building_type == BuildingType::CommandCenter
                        })
                        .map(|b| (b.x, b.y))
                })
                .unwrap_or(default_opp_base);

            let form_pos = crate::formation::compute_formation_positions(
                &combat_pairs,
                start_center,
                target,
                FormationType::Wedge,
            );

            for (uid, tx, ty, off_x, off_y) in form_pos {
                if let Some(u) = engine.units.iter_mut().find(|u| u.id == uid) {
                    u.formation_offset = (off_x, off_y);
                    u.order = UnitOrder::AttackMove { x: tx, y: ty };
                }
            }
        }
        MilitaryCommand::SupportAlly(target) | MilitaryCommand::AttackSpecific(target) => {
            let form_pos = crate::formation::compute_formation_positions(
                &combat_pairs,
                start_center,
                target,
                form,
            );

            for (uid, tx, ty, off_x, off_y) in form_pos {
                if let Some(u) = engine.units.iter_mut().find(|u| u.id == uid) {
                    u.formation_offset = (off_x, off_y);
                    u.order = UnitOrder::AttackMove { x: tx, y: ty };
                }
            }
        }
        MilitaryCommand::Regroup => {
            let form_pos = crate::formation::compute_formation_positions(
                &combat_pairs,
                start_center,
                default_rally,
                form,
            );

            for (uid, tx, ty, off_x, off_y) in form_pos {
                if let Some(u) = engine.units.iter_mut().find(|u| u.id == uid) {
                    if let UnitOrder::AttackTarget { .. } = u.order {
                        continue;
                    }
                    if let UnitOrder::MoveTo { x, y } = u.order {
                        let d_sq = (x - default_rally.0).powi(2) + (y - default_rally.1).powi(2);
                        if d_sq < 100.0_f32.powi(2) {
                            continue;
                        }
                    }

                    u.formation_offset = (off_x, off_y);
                    u.order = UnitOrder::MoveTo { x: tx, y: ty };
                }
            }
        }
    }
}
