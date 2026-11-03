use super::placement::find_team_build_spot;
use super::types::{AutoAction, AutoCondition};
use crate::components::{BuildingType, FormationType, Team, UnitOrder, UnitType};
use crate::resources::TechTreeState;
use crate::GameEngine;

pub fn update_autoplay(engine: &mut GameEngine, dt: f32) {
    update_team_autoplay(engine, Team::Player, dt);
}

pub fn update_team_autoplay(engine: &mut GameEngine, team: Team, dt: f32) {
    let enabled = engine.team_autoplay(team).config.enabled;
    if !enabled {
        return;
    }

    // Tự động điều phối thợ mỏ rảnh cho team bật autoplay
    crate::ai::assign_idle_team_workers(engine, team);

    let timer = &mut engine.team_autoplay_mut(team).timer;
    *timer += dt;
    if *timer < 0.25 {
        return;
    }
    let elapsed = *timer;
    *timer = 0.0;

    let num_loops = engine.team_autoplay(team).config.loops.len();

    for loop_idx in 0..num_loops {
        let loop_enabled = engine.team_autoplay(team).config.loops[loop_idx].enabled;
        if !loop_enabled {
            continue;
        }
        step_team_loop(engine, team, loop_idx, elapsed);
    }
}

fn step_team_loop(engine: &mut GameEngine, team: Team, loop_idx: usize, dt: f32) {
    let loop_data = &engine.team_autoplay(team).config.loops[loop_idx];
    if loop_data.cards.is_empty() {
        return;
    }

    let current_idx = loop_data.current_card_index % loop_data.cards.len();

    // Trong train AI: khi auto chạm tới giới hạn công trình (PowerReactor, TurretTower, Barracks),
    // tự động tăng gấp đôi giới hạn ngay lập tức
    if engine.ai_state.training_enabled {
        let cur_reactors = engine
            .buildings
            .iter()
            .filter(|b| b.team == team && b.building_type == BuildingType::PowerReactor)
            .count();
        let cur_turrets = engine
            .buildings
            .iter()
            .filter(|b| b.team == team && b.building_type == BuildingType::TurretTower)
            .count();
        let cur_barracks = engine
            .buildings
            .iter()
            .filter(|b| b.team == team && b.building_type == BuildingType::Barracks)
            .count();

        let current_card =
            &mut engine.team_autoplay_mut(team).config.loops[loop_idx].cards[current_idx];
        for cond in &mut current_card.conditions {
            if let AutoCondition::BuildingCountLessThan {
                building_type,
                count,
            } = cond
            {
                let actual = match building_type.as_str() {
                    "PowerReactor" => cur_reactors,
                    "TurretTower" => cur_turrets,
                    "Barracks" => cur_barracks,
                    _ => 0,
                };
                if actual >= *count && *count > 0 {
                    *count = (*count * 2).min(128);
                }
            }
        }
    }

    let loop_data = &engine.team_autoplay(team).config.loops[loop_idx];
    let card = loop_data.cards[current_idx].clone();

    if !card.enabled {
        advance_team_loop_card(engine, team, loop_idx);
        return;
    }

    let all_conditions_met = if card.conditions.is_empty() {
        true
    } else {
        card.conditions
            .iter()
            .all(|c| evaluate_condition_for_team(engine, team, c))
    };

    if !all_conditions_met {
        advance_team_loop_card(engine, team, loop_idx);
        return;
    }

    let action_completed = execute_action_for_team(engine, team, loop_idx, &card.action, dt);
    if action_completed {
        advance_team_loop_card(engine, team, loop_idx);
    }
}

fn advance_team_loop_card(engine: &mut GameEngine, team: Team, loop_idx: usize) {
    let loop_data = &mut engine.team_autoplay_mut(team).config.loops[loop_idx];
    loop_data.current_card_progress = 0;
    loop_data.card_timer = 0.0;
    if !loop_data.cards.is_empty() {
        loop_data.current_card_index = (loop_data.current_card_index + 1) % loop_data.cards.len();
    }
}

fn evaluate_condition_for_team(engine: &GameEngine, team: Team, condition: &AutoCondition) -> bool {
    let eco = engine.team_eco(team);
    match condition {
        AutoCondition::Always => true,
        AutoCondition::SupplyNearCap => {
            if eco.max_supply >= 200 {
                return false;
            }
            let depot_under_construction = engine
                .buildings
                .iter()
                .filter(|b| {
                    b.team == team
                        && b.building_type == BuildingType::SupplyDepot
                        && b.build_progress < 1.0
                })
                .count();

            if depot_under_construction >= 2 {
                return false;
            }

            (eco.supply + 2 >= eco.max_supply)
                || (eco.minerals >= 350.0 && eco.supply + 6 >= eco.max_supply)
        }
        AutoCondition::UnitCountLessThan { unit_type, count } => {
            let actual = count_team_units(engine, team, unit_type);
            actual < *count
        }
        AutoCondition::UnitCountGreaterThan { unit_type, count } => {
            let actual = count_team_units(engine, team, unit_type);
            actual > *count
        }
        AutoCondition::MineralsGreaterThan { amount } => eco.minerals > *amount,
        AutoCondition::EnergyGreaterThan { amount } => eco.energy > *amount,
        AutoCondition::ReactorPerConverterRatio => {
            let reactors = engine
                .buildings
                .iter()
                .filter(|b| {
                    b.team == team
                        && b.building_type == BuildingType::PowerReactor
                        && b.build_progress >= 1.0
                })
                .count();
            let converters = engine
                .buildings
                .iter()
                .filter(|b| b.team == team && b.building_type == BuildingType::EnergyConverter)
                .count();
            (reactors / 4) > converters
        }
        AutoCondition::BuildingCountLessThan {
            building_type,
            count,
        } => {
            let b_type = parse_building_type(building_type);
            if let Some(bt) = b_type {
                let actual = engine
                    .buildings
                    .iter()
                    .filter(|b| b.team == team && b.building_type == bt)
                    .count();
                actual < *count
            } else {
                false
            }
        }
    }
}

pub const ALL_TECH_IDS: [&str; 19] = [
    // Khoa học (Science)
    "focused_laser",
    "energy_shielding",
    "dark_matter_rockets",
    "titan_alloy_plating",
    // Ma thuật (Magic)
    "celestial_flying_swords",
    "golden_bell_shield",
    "corrosive_curse",
    "void_cataclysm",
    // Quái vật (Monster)
    "primeval_scales",
    "behemoth_mutation",
    "acid_banelings",
    "hive_proliferation",
    // Hậu cần & Kinh tế (Logistics)
    "optical_drills",
    "overcharged_reactors",
    "fortified_blueprints",
    "magnetic_rounds",
    "exo_boosters",
    "superheated_blades",
    "overload_strike",
];

pub fn get_tech_prerequisite(tech_id: &str) -> Option<String> {
    match tech_id {
        "energy_shielding" => Some("focused_laser".to_string()),
        "titan_alloy_plating" => Some("dark_matter_rockets".to_string()),
        "golden_bell_shield" => Some("celestial_flying_swords".to_string()),
        "void_cataclysm" => Some("corrosive_curse".to_string()),
        "behemoth_mutation" => Some("primeval_scales".to_string()),
        "hive_proliferation" => Some("acid_banelings".to_string()),
        "overcharged_reactors" => Some("optical_drills".to_string()),
        "fortified_blueprints" => Some("overcharged_reactors".to_string()),
        "exo_boosters" => Some("magnetic_rounds".to_string()),
        _ => None,
    }
}

pub fn can_train_unit(tech: &TechTreeState, u_type: UnitType) -> bool {
    match u_type {
        UnitType::Engineer | UnitType::ShockTrooper | UnitType::Rifleman => true,
        UnitType::Vanguard => tech.energy_shielding,
        UnitType::LaserMarksman => tech.focused_laser,
        UnitType::DarkRocketeer => tech.dark_matter_rockets,
        UnitType::TaoistSwordmaster => tech.celestial_flying_swords,
        UnitType::VoidWarlock => tech.void_cataclysm,
        UnitType::BehemothTitan => tech.behemoth_mutation,
        UnitType::AcidBaneling => tech.acid_banelings,
    }
}

#[allow(dead_code)]
pub fn execute_action(
    engine: &mut GameEngine,
    loop_idx: usize,
    action: &AutoAction,
    dt: f32,
) -> bool {
    execute_action_for_team(engine, Team::Player, loop_idx, action, dt)
}

pub fn execute_action_for_team(
    engine: &mut GameEngine,
    team: Team,
    loop_idx: usize,
    action: &AutoAction,
    dt: f32,
) -> bool {
    match action {
        AutoAction::Delay { seconds } => {
            let loop_data = &mut engine.team_autoplay_mut(team).config.loops[loop_idx];
            let timer = loop_data.card_timer + dt;
            loop_data.card_timer = timer;
            if timer >= *seconds {
                return true;
            }
            false
        }
        AutoAction::TrainUnit { unit_type, count } => {
            let u_type = parse_unit_type(unit_type);
            let u_type = match u_type {
                Some(t) => t,
                None => return true,
            };

            let tech = engine.team_tech(team);
            if !can_train_unit(tech, u_type) {
                return true;
            }

            let (mut min_cost, mut en_cost) = engine.config.get_unit_cost(u_type);
            if tech.hive_proliferation {
                min_cost *= 0.65;
                en_cost *= 0.65;
            }

            let eco = engine.team_eco(team);
            if eco.supply >= eco.max_supply {
                return false;
            }
            if eco.minerals < min_cost || eco.energy < en_cost {
                return false;
            }

            let target_btype = match u_type {
                UnitType::Engineer => BuildingType::CommandCenter,
                _ => BuildingType::Barracks,
            };

            let has_building = engine.buildings.iter().any(|b| {
                b.team == team && b.building_type == target_btype && b.build_progress >= 1.0
            });
            if !has_building {
                return true;
            }

            let maybe_b_id = engine
                .buildings
                .iter()
                .filter(|b| {
                    b.team == team
                        && b.building_type == target_btype
                        && b.build_progress >= 1.0
                        && b.training_queue.len() < 5
                })
                .min_by_key(|b| b.training_queue.len())
                .map(|b| b.id);

            if let Some(bid) = maybe_b_id {
                let eco = engine.team_eco_mut(team);
                eco.minerals -= min_cost;
                eco.energy -= en_cost;

                if let Some(b) = engine.buildings.iter_mut().find(|b| b.id == bid) {
                    b.training_queue.push(u_type);
                    let loop_data = &mut engine.team_autoplay_mut(team).config.loops[loop_idx];
                    let progress = loop_data.current_card_progress + 1;
                    loop_data.current_card_progress = progress;
                    if progress >= *count {
                        return true;
                    }
                }
            }
            false
        }
        AutoAction::BuildStructure {
            building_type,
            count,
        } => {
            let b_type = parse_building_type(building_type);
            let b_type = match b_type {
                Some(t) => t,
                None => return true,
            };

            let (min_cost, en_cost) = engine.config.get_building_cost(b_type);
            let eco = engine.team_eco(team);
            if eco.minerals < min_cost || eco.energy < en_cost {
                return false;
            }

            let spot = find_team_build_spot(engine, team, b_type);

            if let Some((bx, by)) = spot {
                if try_team_auto_build(engine, team, b_type, bx, by) {
                    let loop_data = &mut engine.team_autoplay_mut(team).config.loops[loop_idx];
                    let progress = loop_data.current_card_progress + 1;
                    loop_data.current_card_progress = progress;
                    if progress >= *count {
                        return true;
                    }
                }
                false
            } else {
                let msg = format!("Khu vực đã kín chỗ xây {}", building_type);
                engine.team_autoplay_mut(team).last_action_msg = msg;
                true
            }
        }
        AutoAction::ResearchTech {
            tech_id,
            use_priority_queue,
        } => {
            let mut target_tech = tech_id.clone();
            let priority_list = engine.team_autoplay(team).config.priority_techs.clone();
            let team_tech = engine.team_tech(team);

            let mut found_priority = false;
            if *use_priority_queue {
                for p_tech in &priority_list {
                    if !crate::simulation::is_tech_unlocked(team_tech, p_tech)
                        && !team_tech
                            .active_researches
                            .iter()
                            .any(|r| &r.tech_id == p_tech)
                    {
                        if let Some(req) = get_tech_prerequisite(p_tech) {
                            if !crate::simulation::is_tech_unlocked(team_tech, &req)
                                && !team_tech.active_researches.iter().any(|r| r.tech_id == req)
                            {
                                target_tech = req;
                                found_priority = true;
                                break;
                            }
                        }
                        target_tech = p_tech.clone();
                        found_priority = true;
                        break;
                    }
                }
            }

            // Nếu không còn priority tech nào cần nâng, tiếp tục nâng tiếp cho hết toàn bộ cây công nghệ (19/19)
            if !found_priority {
                let team_tech = engine.team_tech(team);
                for &all_t in &ALL_TECH_IDS {
                    if !crate::simulation::is_tech_unlocked(team_tech, all_t)
                        && !team_tech
                            .active_researches
                            .iter()
                            .any(|r| r.tech_id == all_t)
                    {
                        if let Some(req) = get_tech_prerequisite(all_t) {
                            if !crate::simulation::is_tech_unlocked(team_tech, &req)
                                && !team_tech.active_researches.iter().any(|r| r.tech_id == req)
                            {
                                target_tech = req;
                                break;
                            }
                        }
                        target_tech = all_t.to_string();
                        break;
                    }
                }
            }

            let team_tech = engine.team_tech(team);
            if crate::simulation::is_tech_unlocked(team_tech, &target_tech) {
                return true;
            }
            if team_tech
                .active_researches
                .iter()
                .any(|r| r.tech_id == target_tech)
            {
                return true;
            }

            let (tech_state, eco_state, cfg) = match team {
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

            if crate::simulation::process_start_research(tech_state, eco_state, cfg, &target_tech) {
                if team == Team::Player {
                    engine
                        .outgoing_events
                        .push(crate::bridge::EngineToReactEvent::PlaySound {
                            sound: "research_start".to_string(),
                        });
                }
                return true;
            }

            false
        }
        AutoAction::AttackWave {
            formation: form_override,
        } => {
            let target_pos = engine
                .buildings
                .iter()
                .find(|b| {
                    engine.are_hostile(team, b.team)
                        && b.building_type == BuildingType::CommandCenter
                })
                .map(|b| (b.x, b.y))
                .or_else(|| {
                    engine
                        .buildings
                        .iter()
                        .find(|b| engine.are_hostile(team, b.team))
                        .map(|b| (b.x, b.y))
                })
                .or_else(|| {
                    engine
                        .units
                        .iter()
                        .find(|u| engine.are_hostile(team, u.team))
                        .map(|u| (u.x, u.y))
                })
                .unwrap_or((crate::map::MAP_WIDTH * 0.5, crate::map::MAP_HEIGHT * 0.5));

            let combat_pairs: Vec<(u32, UnitType)> = engine
                .units
                .iter()
                .filter(|u| u.team == team && u.unit_type != UnitType::Engineer)
                .map(|u| (u.id, u.unit_type))
                .collect();

            if !combat_pairs.is_empty() {
                // Xác định đội hình tấn công: ưu tiên thẻ chỉ định -> cấu hình Auto -> đội hình hiện tại của team
                let chosen_form = if let Some(ref f_str) = form_override {
                    match f_str.to_lowercase().as_str() {
                        "wedge" => FormationType::Wedge,
                        "box" => FormationType::Box,
                        "scatter" => FormationType::Scatter,
                        "circle" => FormationType::Circle,
                        "column" => FormationType::Column,
                        _ => FormationType::Line,
                    }
                } else {
                    engine.team_autoplay(team).config.preferred_formation
                };

                engine.set_team_formation(team, chosen_form);

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

                let form_positions = crate::formation::compute_formation_positions(
                    &combat_pairs,
                    start_center,
                    target_pos,
                    chosen_form,
                );

                for (uid, tx, ty, off_x, off_y) in form_positions {
                    if let Some(u) = engine.units.iter_mut().find(|u| u.id == uid) {
                        if matches!(u.order, UnitOrder::AttackTarget { .. }) {
                            continue;
                        }
                        u.formation_offset = (off_x, off_y);
                        u.order = UnitOrder::AttackMove { x: tx, y: ty };
                    }
                }
            }

            if team == Team::Player {
                engine
                    .outgoing_events
                    .push(crate::bridge::EngineToReactEvent::PlaySound {
                        sound: "attack_order".to_string(),
                    });
            }
            true
        }
    }
}

pub fn try_player_auto_build(
    engine: &mut GameEngine,
    b_type: BuildingType,
    wx: f32,
    wy: f32,
) -> bool {
    try_team_auto_build(engine, Team::Player, b_type, wx, wy)
}

pub fn try_team_auto_build(
    engine: &mut GameEngine,
    team: Team,
    b_type: BuildingType,
    wx: f32,
    wy: f32,
) -> bool {
    let (cost_min, cost_en) = engine.config.get_building_cost(b_type);
    let eco = engine.team_eco_mut(team);
    if eco.minerals < cost_min || eco.energy < cost_en {
        return false;
    }

    eco.minerals -= cost_min;
    eco.energy -= cost_en;
    let bid = engine.alloc_id();

    let mut building = engine
        .config
        .create_building(bid, team, b_type, wx, wy, false);
    building.build_progress = 0.05;
    building.hp = 1.0;
    building.rally_x = if team == Team::Player {
        wx + 40.0
    } else {
        wx - 40.0
    };
    building.rally_y = if team == Team::Player {
        wy + 40.0
    } else {
        wy - 40.0
    };

    let tech = engine.team_tech(team);

    if tech.titan_alloy_plating {
        building.resistances.physical = (building.resistances.physical + 0.20).min(0.80);
    }
    let mut hp_mult = 1.0;
    if tech.fortified_blueprints {
        hp_mult += 0.3;
    }
    if tech.titan_alloy_plating {
        hp_mult += 0.25;
    }
    building.max_hp *= hp_mult;

    let is_armed = engine
        .config
        .get_building(b_type)
        .map(|c| c.turret_damage > 0.0)
        .unwrap_or(false);
    if is_armed && tech.dark_matter_rockets {
        building.damage_type = crate::components::DamageType::Explosive;
    }

    engine.buildings.push(building);

    let nearest_worker_id = engine
        .units
        .iter()
        .filter(|u| u.team == team && u.unit_type == UnitType::Engineer)
        .min_by(|a, b| {
            let da = (a.x - wx).powi(2) + (a.y - wy).powi(2);
            let db = (b.x - wx).powi(2) + (b.y - wy).powi(2);
            da.partial_cmp(&db).unwrap()
        })
        .map(|u| u.id);

    if let Some(wid) = nearest_worker_id {
        if let Some(w) = engine.units.iter_mut().find(|u| u.id == wid) {
            w.order = UnitOrder::Build {
                target_building_id: bid,
            };
        }
    }

    if team == Team::Player {
        engine
            .outgoing_events
            .push(crate::bridge::EngineToReactEvent::PlaySound {
                sound: "place_building".to_string(),
            });
    }

    true
}

pub fn count_team_units(engine: &GameEngine, team: Team, unit_type: &str) -> usize {
    if unit_type.eq_ignore_ascii_case("combat") {
        engine
            .units
            .iter()
            .filter(|u| u.team == team && u.unit_type != UnitType::Engineer)
            .count()
    } else if let Some(ut) = parse_unit_type(unit_type) {
        engine
            .units
            .iter()
            .filter(|u| u.team == team && u.unit_type == ut)
            .count()
    } else {
        0
    }
}

#[allow(dead_code)]
pub fn count_player_units(engine: &GameEngine, unit_type: &str) -> usize {
    if unit_type.eq_ignore_ascii_case("combat") {
        engine
            .units
            .iter()
            .filter(|u| u.team == Team::Player && u.unit_type != UnitType::Engineer)
            .count()
    } else if let Some(ut) = parse_unit_type(unit_type) {
        engine
            .units
            .iter()
            .filter(|u| u.team == Team::Player && u.unit_type == ut)
            .count()
    } else {
        0
    }
}

pub fn parse_unit_type(s: &str) -> Option<UnitType> {
    match s.to_lowercase().as_str() {
        "engineer" | "worker" => Some(UnitType::Engineer),
        "shocktrooper" | "striker" => Some(UnitType::ShockTrooper),
        "rifleman" | "rifle" => Some(UnitType::Rifleman),
        "vanguard" | "shield" => Some(UnitType::Vanguard),
        "lasermarksman" | "laser" => Some(UnitType::LaserMarksman),
        "darkrocketeer" | "rocket" => Some(UnitType::DarkRocketeer),
        "taoistswordmaster" | "sword" => Some(UnitType::TaoistSwordmaster),
        "voidwarlock" | "warlock" => Some(UnitType::VoidWarlock),
        "behemothtitan" | "titan" => Some(UnitType::BehemothTitan),
        "acidbaneling" | "baneling" => Some(UnitType::AcidBaneling),
        _ => None,
    }
}

pub fn parse_building_type(s: &str) -> Option<BuildingType> {
    match s.to_lowercase().as_str() {
        "commandcenter" | "cc" | "hq" => Some(BuildingType::CommandCenter),
        "supplydepot" | "supply" => Some(BuildingType::SupplyDepot),
        "powerreactor" | "reactor" => Some(BuildingType::PowerReactor),
        "barracks" => Some(BuildingType::Barracks),
        "turrettower" | "turret" => Some(BuildingType::TurretTower),
        "energyconverter" | "converter" => Some(BuildingType::EnergyConverter),
        _ => None,
    }
}
