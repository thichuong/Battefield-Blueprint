use crate::bridge::*;
use crate::components::*;
use crate::config::GameConfig;
use crate::resources::{PlayerEconomy, TechTreeState};
use crate::simulation::economy::send_sync_event;
use crate::GameEngine;

pub fn process_commands(engine: &mut GameEngine) {
    while let Some(cmd) = engine.pending_commands.pop_front() {
        match cmd {
            ReactToEngineCommand::StartGame { difficulty, slots } => {
                engine.difficulty = difficulty.clone();
                for slot in slots {
                    let idx = slot.slot_index;
                    if idx < 8 {
                        engine.player_slots[idx] = slot;
                    }
                }
                // Dọn sạch toàn bộ thực thể & dữ liệu của màn cũ trước khi khởi động màn mới
                engine.reset_game();
                engine.is_paused = false;
                send_sync_event(engine);
            }
            ReactToEngineCommand::SetPaused { paused } => {
                engine.is_paused = paused;
                send_sync_event(engine);
            }
            ReactToEngineCommand::UpgradeTech { tech_id } => {
                try_start_research(engine, &tech_id);
                send_sync_event(engine);
            }
            ReactToEngineCommand::UpdateSettings {
                sound_volume,
                camera_speed,
                auto_pause,
                show_grid,
            } => {
                engine.settings.sound_volume = sound_volume;
                engine.settings.camera_speed = camera_speed;
                engine.settings.auto_pause_on_tech = auto_pause;
                engine.settings.show_grid = show_grid;
            }
            ReactToEngineCommand::RestartGame => {
                // Dọn sạch toàn bộ thực thể & dữ liệu của màn cũ
                engine.reset_game();
                engine.is_paused = false;
                send_sync_event(engine);
            }
            ReactToEngineCommand::UpdateAutoPlayConfig { config_json } => {
                if let Ok(cfg) =
                    serde_json::from_str::<crate::autoplay::AutoPlayConfig>(&config_json)
                {
                    engine.autoplay.config = cfg;
                }
                send_sync_event(engine);
            }
            ReactToEngineCommand::TogglePriorityTech { tech_id } => {
                if let Some(pos) = engine
                    .autoplay
                    .config
                    .priority_techs
                    .iter()
                    .position(|t| t == &tech_id)
                {
                    engine.autoplay.config.priority_techs.remove(pos);
                } else {
                    engine.autoplay.config.priority_techs.push(tech_id);
                }
                send_sync_event(engine);
            }
            ReactToEngineCommand::SetFormation { formation } => {
                let form = match formation.to_lowercase().as_str() {
                    "wedge" => FormationType::Wedge,
                    "box" => FormationType::Box,
                    "scatter" => FormationType::Scatter,
                    "circle" => FormationType::Circle,
                    "column" => FormationType::Column,
                    _ => FormationType::Line,
                };
                set_player_formation_and_reform(engine, form);
                send_sync_event(engine);
            }
        }
    }
}

pub fn set_player_formation_and_reform(engine: &mut GameEngine, form: FormationType) {
    engine.set_team_formation(Team::Player, form);

    if engine.selected_unit_ids.is_empty() {
        return;
    }

    let selected_pairs: Vec<(u32, UnitType)> = engine
        .selected_unit_ids
        .iter()
        .filter_map(|uid| {
            engine
                .units
                .iter()
                .find(|u| u.id == *uid && u.team == Team::Player)
                .map(|u| (u.id, u.unit_type))
        })
        .collect();

    if selected_pairs.is_empty() {
        return;
    }

    // Tâm và hướng nhìn hiện tại của cụm quân
    let mut sum_x = 0.0;
    let mut sum_y = 0.0;
    let mut facing_sum = 0.0;
    for (uid, _) in &selected_pairs {
        if let Some(u) = engine.units.iter().find(|u| u.id == *uid) {
            sum_x += u.x;
            sum_y += u.y;
            facing_sum += if u.facing_right { 1.0 } else { -1.0 };
        }
    }
    let center_x = sum_x / (selected_pairs.len() as f32);
    let center_y = sum_y / (selected_pairs.len() as f32);
    let fwd_dir = if facing_sum >= 0.0 { 1.0 } else { -1.0 };
    let target_ref = (center_x + fwd_dir * 100.0, center_y);

    let form_positions = crate::formation::compute_formation_positions(
        &selected_pairs,
        (center_x, center_y),
        target_ref,
        form,
    );

    for (uid, target_x, target_y, off_x, off_y) in form_positions {
        if let Some(u) = engine.units.iter_mut().find(|u| u.id == uid) {
            u.formation_offset = (off_x, off_y);
            // Dàn lại vị trí ngay tại chỗ
            u.order = UnitOrder::MoveTo {
                x: target_x,
                y: target_y,
            };
        }
    }
}

pub fn is_tech_unlocked(tech_state: &TechTreeState, tech_id: &str) -> bool {
    match tech_id {
        "focused_laser" => tech_state.focused_laser,
        "energy_shielding" => tech_state.energy_shielding,
        "dark_matter_rockets" => tech_state.dark_matter_rockets,
        "titan_alloy_plating" => tech_state.titan_alloy_plating,
        "celestial_flying_swords" => tech_state.celestial_flying_swords,
        "golden_bell_shield" => tech_state.golden_bell_shield,
        "corrosive_curse" => tech_state.corrosive_curse,
        "void_cataclysm" => tech_state.void_cataclysm,
        "primeval_scales" => tech_state.primeval_scales,
        "behemoth_mutation" => tech_state.behemoth_mutation,
        "acid_banelings" => tech_state.acid_banelings,
        "hive_proliferation" => tech_state.hive_proliferation,
        "optical_drills" => tech_state.optical_drills,
        "overcharged_reactors" => tech_state.overcharged_reactors,
        "fortified_blueprints" => tech_state.fortified_blueprints,
        "magnetic_rounds" => tech_state.magnetic_rounds,
        "exo_boosters" => tech_state.exo_boosters,
        "superheated_blades" => tech_state.superheated_blades,
        "overload_strike" => tech_state.overload_strike,
        _ => false,
    }
}

fn sync_compat_research_fields(tech_state: &mut TechTreeState) {
    if let Some(first) = tech_state.active_researches.first() {
        tech_state.current_research = Some(first.tech_id.clone());
        tech_state.research_progress = first.progress;
        tech_state.research_total_time = first.total_time;
    } else {
        tech_state.current_research = None;
        tech_state.research_progress = 0.0;
        tech_state.research_total_time = 0.0;
    }
}

pub fn process_start_research(
    player_tech: &mut TechTreeState,
    player_eco: &mut PlayerEconomy,
    config: &GameConfig,
    tech_id: &str,
) -> bool {
    // Nếu đã mở khóa rồi thì bỏ qua
    if is_tech_unlocked(player_tech, tech_id) {
        return false;
    }

    // Nếu công nghệ này đang được nghiên cứu thì bỏ qua
    if player_tech
        .active_researches
        .iter()
        .any(|r| r.tech_id == tech_id)
    {
        return false;
    }

    let (cost_minerals, cost_energy, time, branch) = if let Some(t) = config.get_tech(tech_id) {
        (
            t.cost_minerals,
            t.cost_energy,
            t.research_time,
            t.branch.clone(),
        )
    } else {
        return false;
    };

    // Kiểm tra giới hạn số công nghệ đang nghiên cứu trong cùng một nhánh
    let branch_active = player_tech
        .active_researches
        .iter()
        .filter(|r| r.branch == branch)
        .count();
    if branch_active >= crate::config::MAX_CONCURRENT_RESEARCH_PER_BRANCH {
        return false;
    }

    // Kiểm tra tổng số công nghệ đang nghiên cứu đồng thời
    if player_tech.active_researches.len() >= crate::config::MAX_TOTAL_CONCURRENT_RESEARCH {
        return false;
    }

    if player_eco.minerals >= cost_minerals && player_eco.energy >= cost_energy {
        player_eco.minerals -= cost_minerals;
        player_eco.energy -= cost_energy;

        player_tech
            .active_researches
            .push(crate::resources::ActiveResearch {
                tech_id: tech_id.to_string(),
                branch,
                progress: 0.0,
                total_time: time,
            });

        sync_compat_research_fields(player_tech);
        true
    } else {
        false
    }
}

pub fn process_update_tech_research(
    team: Team,
    player_tech: &mut TechTreeState,
    buildings: &mut [Building],
    units: &mut [Unit],
    config: &GameConfig,
    dt: f32,
) -> Vec<String> {
    if player_tech.active_researches.is_empty() {
        return Vec::new();
    }

    let mut completed_techs = Vec::new();
    for research in player_tech.active_researches.iter_mut() {
        research.progress += dt / research.total_time;
        if research.progress >= 1.0 {
            completed_techs.push(research.tech_id.clone());
        }
    }

    if !completed_techs.is_empty() {
        for tech_id in &completed_techs {
            match tech_id.as_str() {
                // Khoa học
                "focused_laser" => player_tech.focused_laser = true,
                "energy_shielding" => player_tech.energy_shielding = true,
                "dark_matter_rockets" => {
                    player_tech.dark_matter_rockets = true;
                    for b in buildings.iter_mut().filter(|b| b.team == team) {
                        let is_armed = config
                            .get_building(b.building_type)
                            .map(|c| c.turret_damage > 0.0)
                            .unwrap_or(false);
                        if is_armed {
                            b.damage_type = DamageType::Explosive;
                        }
                    }
                }
                "titan_alloy_plating" => {
                    player_tech.titan_alloy_plating = true;
                    // Tăng thêm máu cho các công trình hiện hữu
                    for b in buildings.iter_mut().filter(|b| b.team == team) {
                        b.max_hp *= 1.25;
                        b.hp *= 1.25;
                        b.resistances.physical = (b.resistances.physical + 0.20).min(0.80);
                    }
                }

                // Ma thuật
                "celestial_flying_swords" => player_tech.celestial_flying_swords = true,
                "golden_bell_shield" => {
                    player_tech.golden_bell_shield = true;
                    for u in units.iter_mut().filter(|u| u.team == team) {
                        u.resistances.energy = (u.resistances.energy + 0.25).min(0.80);
                        u.resistances.arcane = (u.resistances.arcane + 0.30).min(0.80);
                    }
                }
                "corrosive_curse" => player_tech.corrosive_curse = true,
                "void_cataclysm" => player_tech.void_cataclysm = true,

                // Quái vật
                "primeval_scales" => {
                    player_tech.primeval_scales = true;
                    for u in units.iter_mut().filter(|u| u.team == team) {
                        u.resistances.physical = (u.resistances.physical + 0.25).min(0.80);
                        u.resistances.explosive = (u.resistances.explosive + 0.25).min(0.80);
                    }
                }
                "behemoth_mutation" => player_tech.behemoth_mutation = true,
                "acid_banelings" => player_tech.acid_banelings = true,
                "hive_proliferation" => {
                    player_tech.hive_proliferation = true;
                    for u in units.iter_mut().filter(|u| u.team == team) {
                        u.speed *= 1.25;
                    }
                }

                // Kinh tế / Phụ trợ
                "optical_drills" => player_tech.optical_drills = true,
                "overcharged_reactors" => player_tech.overcharged_reactors = true,
                "fortified_blueprints" => player_tech.fortified_blueprints = true,
                "magnetic_rounds" => player_tech.magnetic_rounds = true,
                "exo_boosters" => player_tech.exo_boosters = true,
                "superheated_blades" => player_tech.superheated_blades = true,
                "overload_strike" => player_tech.overload_strike = true,
                _ => {}
            }
        }

        player_tech
            .active_researches
            .retain(|r| !completed_techs.contains(&r.tech_id));
    }

    sync_compat_research_fields(player_tech);
    completed_techs
}

pub fn try_start_research(engine: &mut GameEngine, tech_id: &str) {
    if process_start_research(
        &mut engine.player_tech,
        &mut engine.player_eco,
        &engine.config,
        tech_id,
    ) {
        engine.outgoing_events.push(EngineToReactEvent::PlaySound {
            sound: "research_start".to_string(),
        });
        send_sync_event(engine);
    }
}

pub fn update_tech_research(engine: &mut GameEngine, dt: f32) {
    let mut completed_player = Vec::new();
    for i in 0..8 {
        if !engine.player_slots[i].active {
            continue;
        }
        let team = Team::from_index(i);
        let completed = match team {
            Team::Player => process_update_tech_research(
                team,
                &mut engine.player_tech,
                &mut engine.buildings,
                &mut engine.units,
                &engine.config,
                dt,
            ),
            Team::Enemy => process_update_tech_research(
                team,
                &mut engine.enemy_tech,
                &mut engine.buildings,
                &mut engine.units,
                &engine.config,
                dt,
            ),
            _ => process_update_tech_research(
                team,
                &mut engine.extra_techs[i - 2],
                &mut engine.buildings,
                &mut engine.units,
                &engine.config,
                dt,
            ),
        };
        if team == Team::Player {
            completed_player = completed;
        }
    }

    if !completed_player.is_empty() {
        engine.outgoing_events.push(EngineToReactEvent::PlaySound {
            sound: "research_complete".to_string(),
        });
        send_sync_event(engine);
    }
}
