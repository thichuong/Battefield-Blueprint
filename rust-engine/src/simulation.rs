use crate::bridge::*;
use crate::components::*;
use crate::config::{BuildingStatsConfig, GameConfig};
use crate::map::{MAP_HEIGHT, MAP_WIDTH};
use crate::resources::{PlayerEconomy, TechTreeState};
use crate::GameEngine;

pub fn step(engine: &mut GameEngine, dt: f32) {
    // 1. Xử lý các lệnh từ React
    if !engine.is_headless {
        process_commands(engine);
    }

    if engine.is_paused || engine.game_over {
        return;
    }

    let dt = if engine.is_headless { dt } else { dt.min(0.05) };
    engine.game_time += dt;

    // 2. Di chuyển Camera bằng phím WASD / Mũi tên và Edge Scrolling
    if !engine.is_headless {
        update_camera(engine, dt);
    }

    // 3. Cập nhật Kinh tế tự nhiên & Đo tốc độ thu nhập
    update_economy(engine, dt);

    // 4. Cập nhật Tiến trình Nghiên cứu Cây Công Nghệ
    update_tech_research(engine, dt);

    // 5. Cập nhật Đơn vị Lính (Di chuyển, Khai mỏ, Tấn công mép nhà, Attack-Move)
    update_units(engine, dt);

    // 6. Cập nhật Công trình (Xây dựng, Huấn luyện lính, Tháp pháo bắn)
    update_buildings(engine, dt);

    // 7. Cập nhật Đạn & Tia Laser (sát thương đơn vị & công trình)
    update_projectiles(engine, dt);

    // 8. Cập nhật Hiệu ứng Chữ nổi (Floating numbers) & Fog
    if !engine.is_headless {
        update_floating_texts(engine, dt);
        update_fog(engine);
    }

    // 10. Cập nhật AI Đối thủ (kinh tế + điều quân càn quét nhà)
    engine.ai_think_timer += dt;
    if engine.ai_think_timer >= 0.5 {
        engine.ai_think_timer = 0.0;
        crate::ai::update_ai(engine);
    }

    // 10.5 Cập nhật Chế độ Tự Động (Auto-Play) cho tất cả team có autoplay bật
    for i in 0..crate::components::MAX_PLAYERS {
        let team = Team::from_index(i);
        if engine.is_slot_active(team) && engine.team_autoplay(team).config.enabled {
            crate::autoplay::update_team_autoplay(engine, team, dt);
        }
    }

    // 11. Kiểm tra Điều kiện Thắng / Thua
    check_game_over(engine);

    // 12. Bắn sự kiện định kỳ đồng bộ về React
    if !engine.is_headless {
        engine.sync_timer += dt;
        if engine.sync_timer >= 0.1 {
            engine.sync_timer = 0.0;
            send_sync_event(engine);
        }
    }
}

fn process_commands(engine: &mut GameEngine) {
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
                let initial_min = if difficulty == "hard" {
                    400.0
                } else if difficulty == "easy" {
                    100.0
                } else {
                    200.0
                };
                let initial_en = if difficulty == "hard" {
                    150.0
                } else if difficulty == "easy" {
                    0.0
                } else {
                    50.0
                };
                for i in 1..8 {
                    if engine.player_slots[i].active {
                        let team = Team::from_index(i);
                        let eco = engine.team_eco_mut(team);
                        eco.minerals = initial_min;
                        eco.energy = initial_en;
                    }
                }
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
                let diff = engine.difficulty.clone();
                // Dọn sạch toàn bộ thực thể & dữ liệu của màn cũ
                engine.reset_game();
                if diff == "hard" {
                    engine.enemy_eco.minerals = 400.0;
                    engine.enemy_eco.energy = 150.0;
                } else if diff == "easy" {
                    engine.enemy_eco.minerals = 100.0;
                    engine.enemy_eco.energy = 0.0;
                }
                engine.is_paused = false;
                send_sync_event(engine);
            }
            ReactToEngineCommand::UpdateAutoPlayConfig { config_json } => {
                if let Ok(cfg) = serde_json::from_str::<crate::autoplay::AutoPlayConfig>(&config_json) {
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

fn try_start_research(engine: &mut GameEngine, tech_id: &str) {
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

fn update_tech_research(engine: &mut GameEngine, dt: f32) {
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

fn update_camera(engine: &mut GameEngine, dt: f32) {
    let speed = 650.0 * engine.settings.camera_speed * dt / engine.camera.zoom;

    // Bàn phím điều khiển
    if engine.keys_down.contains("KeyW") || engine.keys_down.contains("ArrowUp") {
        engine.camera.y -= speed;
    }
    if engine.keys_down.contains("KeyS") || engine.keys_down.contains("ArrowDown") {
        engine.camera.y += speed;
    }
    if engine.keys_down.contains("ArrowLeft")
        || (engine.keys_down.contains("KeyA") && !engine.is_attack_move_mode)
    {
        engine.camera.x -= speed;
    }
    if engine.keys_down.contains("KeyD") || engine.keys_down.contains("ArrowRight") {
        engine.camera.x += speed;
    }

    // Cuộn mép màn hình (Edge-Scrolling) khi không nhấn giữ kéo chuột chọn
    if !engine.is_mouse_down && engine.camera.viewport_w > 0.0 && engine.camera.viewport_h > 0.0 {
        let edge_margin = 18.0;
        let edge_spd = speed * 1.1;
        if engine.mouse_screen_x >= 0.0 && engine.mouse_screen_x <= edge_margin {
            engine.camera.x -= edge_spd;
        } else if engine.mouse_screen_x >= engine.camera.viewport_w - edge_margin
            && engine.mouse_screen_x <= engine.camera.viewport_w
        {
            engine.camera.x += edge_spd;
        }
        if engine.mouse_screen_y >= 0.0 && engine.mouse_screen_y <= edge_margin {
            engine.camera.y -= edge_spd;
        } else if engine.mouse_screen_y >= engine.camera.viewport_h - edge_margin
            && engine.mouse_screen_y <= engine.camera.viewport_h
        {
            engine.camera.y += edge_spd;
        }
    }

    let min_x = (engine.camera.viewport_w * 0.5 / engine.camera.zoom).min(MAP_WIDTH * 0.5);
    let max_x = (MAP_WIDTH - engine.camera.viewport_w * 0.5 / engine.camera.zoom).max(min_x);
    let min_y = (engine.camera.viewport_h * 0.5 / engine.camera.zoom).min(MAP_HEIGHT * 0.5);
    let max_y = (MAP_HEIGHT - engine.camera.viewport_h * 0.5 / engine.camera.zoom).max(min_y);

    engine.camera.x = engine.camera.x.clamp(min_x, max_x);
    engine.camera.y = engine.camera.y.clamp(min_y, max_y);
}

pub use crate::config::{
    CONVERT_ENERGY_PER_SEC, CONVERT_MINERALS_PER_SEC, ENERGY_CONVERSION_THRESHOLD,
};

/// Tính toán năng lượng tiêu thụ và khoáng sản (minerals) nhận được từ quá trình chuyển đổi.
/// - Ngưỡng an toàn tối thiểu là 1000 Năng lượng. Dưới 1000 tự động dừng.
/// - Trả về (actual_energy_consumed, minerals_gained).
pub fn calculate_energy_conversion(energy: f32, num_converters: usize, dt: f32) -> (f32, f32) {
    if num_converters == 0 || energy <= ENERGY_CONVERSION_THRESHOLD {
        return (0.0, 0.0);
    }

    let surplus = energy - ENERGY_CONVERSION_THRESHOLD;
    let desired_energy = (num_converters as f32) * CONVERT_ENERGY_PER_SEC * dt;
    let actual_energy = desired_energy.min(surplus);

    if actual_energy > 0.0 {
        let minerals_gained = actual_energy * (CONVERT_MINERALS_PER_SEC / CONVERT_ENERGY_PER_SEC);
        (actual_energy, minerals_gained)
    } else {
        (0.0, 0.0)
    }
}

fn update_economy(engine: &mut GameEngine, dt: f32) {
    let mut team_reactors = [0usize; 8];
    let mut team_supply_depots = [0usize; 8];
    let mut team_converters = [0usize; 8];

    for b in &engine.buildings {
        if b.build_progress >= 1.0 {
            let t_idx = b.team.index() % 8;
            match b.building_type {
                BuildingType::PowerReactor => team_reactors[t_idx] += 1,
                BuildingType::SupplyDepot => team_supply_depots[t_idx] += 1,
                BuildingType::EnergyConverter => team_converters[t_idx] += 1,
                _ => {}
            }
        }
    }

    let base_energy = engine.config.economy.base_energy_per_sec;
    let reactor_energy = engine
        .config
        .get_building(BuildingType::PowerReactor)
        .map(|b| b.energy_per_sec)
        .unwrap_or(8.0);
    let base_supply = engine.config.economy.base_max_supply;
    let depot_supply = engine
        .config
        .get_building(BuildingType::SupplyDepot)
        .map(|b| b.supply_bonus)
        .unwrap_or(8);

    let mut unit_counts = [0u32; 8];
    for u in &engine.units {
        unit_counts[u.team.index() % 8] += 1;
    }

    let mut player_gained_minerals = 0.0;

    for i in 0..8 {
        if !engine.player_slots[i].active {
            continue;
        }
        let team = Team::from_index(i);
        let has_overcharged = engine.team_tech(team).overcharged_reactors;
        let mult = if has_overcharged { 1.5 } else { 1.0 };
        let energy_rate = (base_energy + team_reactors[i] as f32 * reactor_energy) * mult;

        let num_converters = team_converters[i];
        let eco = match team {
            Team::Player => &mut engine.player_eco,
            Team::Enemy => &mut engine.enemy_eco,
            _ => &mut engine.extra_ecos[i - 2],
        };
        eco.energy += energy_rate * dt;
        eco.energy_per_sec = energy_rate;

        // Chuyển hóa Năng lượng thành Khoáng sản
        if num_converters > 0 {
            let (used_energy, gained_minerals) =
                calculate_energy_conversion(eco.energy, num_converters, dt);
            if used_energy > 0.0 {
                eco.energy -= used_energy;
                eco.minerals += gained_minerals;
                if team == Team::Player {
                    player_gained_minerals += gained_minerals;
                }
                let consumed_per_sec =
                    (used_energy / dt).min((num_converters as f32) * CONVERT_ENERGY_PER_SEC);
                eco.energy_per_sec -= consumed_per_sec;
            }
        }

        eco.max_supply = base_supply + (team_supply_depots[i] as u32) * depot_supply;
        eco.supply = unit_counts[i];
    }

    engine.minerals_mined_accumulator += player_gained_minerals;

    // Tính tốc độ thu nhập khoáng sản cho Player
    engine.income_calc_timer += dt;
    if engine.income_calc_timer >= 1.0 {
        engine.player_eco.minerals_per_sec =
            engine.minerals_mined_accumulator / engine.income_calc_timer;
        engine.minerals_mined_accumulator = 0.0;
        engine.income_calc_timer = 0.0;
    }
}

fn update_units(engine: &mut GameEngine, dt: f32) {
    let mut new_projectiles = Vec::new();
    let mut new_floating_texts = Vec::new();
    let mut mineral_deliveries: Vec<(Team, f32, f32, f32)> = Vec::new();
    let factions = engine.team_factions();

    let has_exo: [bool; 8] = std::array::from_fn(|i| {
        engine.team_tech(Team::from_index(i)).exo_boosters
    });
    let has_superheated: [bool; 8] = std::array::from_fn(|i| {
        engine.team_tech(Team::from_index(i)).superheated_blades
    });
    let has_magnetic: [bool; 8] = std::array::from_fn(|i| {
        engine.team_tech(Team::from_index(i)).magnetic_rounds
    });
    let has_bell: [bool; 8] = std::array::from_fn(|i| {
        engine.team_tech(Team::from_index(i)).golden_bell_shield
    });
    let has_drills: [bool; 8] = std::array::from_fn(|i| {
        engine.team_tech(Team::from_index(i)).optical_drills
    });

    engine
        .spatial_grid
        .rebuild_with_factions(&engine.units, &engine.buildings, &factions);
    let spatial_grid = &engine.spatial_grid;

    // Snapshot vị trí công trình chưa xây xong và mỏ quặng
    // (id, team, x, y, radius, build_progress)
    let building_targets: Vec<(u32, Team, f32, f32, f32, f32)> = engine
        .buildings
        .iter()
        .map(|b| {
            (
                b.id,
                b.team,
                b.x,
                b.y,
                (b.width.max(b.height) * 0.5),
                b.build_progress,
            )
        })
        .collect();
    let unit_targets: Vec<(u32, Team, f32, f32)> = engine
        .units
        .iter()
        .map(|u| (u.id, u.team, u.x, u.y))
        .collect();
    let mineral_nodes: Vec<(u32, f32, f32)> =
        engine.minerals.iter().map(|m| (m.id, m.x, m.y)).collect();
    let cc_targets: Vec<(Team, f32, f32)> = engine
        .buildings
        .iter()
        .filter(|b| b.building_type == BuildingType::CommandCenter)
        .map(|b| (b.team, b.x, b.y))
        .collect();

    for unit in engine.units.iter_mut() {
        if unit.current_cooldown > 0.0 {
            unit.current_cooldown -= dt;
        }

        let u_team_idx = unit.team.index() % 8;
        let my_faction = factions[u_team_idx];

        let mut cur_speed = unit.speed;
        if has_exo[u_team_idx] {
            cur_speed *= 1.2;
        }

        match unit.order {
            UnitOrder::Idle => {
                if unit.unit_type != UnitType::Engineer {
                    let scan_range = (unit.attack_range * 1.6).max(300.0);
                    if let Some((target_id, is_building, _)) = spatial_grid.find_nearest_hostile(
                        glam::Vec2::new(unit.x, unit.y),
                        scan_range,
                        my_faction,
                    ) {
                        unit.order = UnitOrder::AttackTarget {
                            target_id,
                            is_building,
                        };
                    }
                }
            }
            UnitOrder::AttackMove { x, y } => {
                // Tự động tìm mục tiêu lính hoặc nhà trên đường di chuyển qua Spatial Grid (Ưu tiên tháp pháo & lính)
                let scan_range = (unit.attack_range * 2.2).max(380.0);
                if let Some((target_id, is_building, _)) = spatial_grid.find_nearest_hostile(
                    glam::Vec2::new(unit.x, unit.y),
                    scan_range,
                    my_faction,
                ) {
                    unit.order = UnitOrder::AttackTarget {
                        target_id,
                        is_building,
                    };
                } else {
                    let dx = x - unit.x;
                    let dy = y - unit.y;
                    let dist = (dx * dx + dy * dy).sqrt();
                    if dist > 4.0 {
                        let move_dist = (cur_speed * dt).min(dist);
                        unit.x += (dx / dist) * move_dist;
                        unit.y += (dy / dist) * move_dist;
                        unit.walk_cycle += dt * 10.0;
                        unit.facing_right = dx >= 0.0;
                    } else {
                        unit.order = UnitOrder::Idle;
                    }
                }
            }
            UnitOrder::MoveTo { x, y } => {
                let dx = x - unit.x;
                let dy = y - unit.y;
                let dist = (dx * dx + dy * dy).sqrt();
                if dist > 4.0 {
                    let move_dist = (cur_speed * dt).min(dist);
                    unit.x += (dx / dist) * move_dist;
                    unit.y += (dy / dist) * move_dist;
                    unit.walk_cycle += dt * 10.0;
                    unit.facing_right = dx >= 0.0;
                } else {
                    unit.order = UnitOrder::Idle;
                }
            }
            UnitOrder::Harvest {
                target_mineral_id,
                phase,
            } => {
                match phase {
                    HarvestPhase::MovingToNode => {
                        if let Some((_, mx, my)) = mineral_nodes
                            .iter()
                            .find(|(id, _, _)| *id == target_mineral_id)
                        {
                            let dx = mx - unit.x;
                            let dy = my - unit.y;
                            let dist = (dx * dx + dy * dy).sqrt();
                            if dist > 35.0 {
                                let move_dist = (cur_speed * dt).min(dist);
                                unit.x += (dx / dist) * move_dist;
                                unit.y += (dy / dist) * move_dist;
                                unit.walk_cycle += dt * 10.0;
                                unit.facing_right = dx >= 0.0;
                            } else {
                                unit.order = UnitOrder::Harvest {
                                    target_mineral_id,
                                    phase: HarvestPhase::Mining(0.0),
                                };
                            }
                        } else {
                            // Mỏ không còn -> Tìm mỏ gần nhất kế tiếp
                            let nearest = mineral_nodes.iter().min_by(|a, b| {
                                let da = (a.1 - unit.x).powi(2) + (a.2 - unit.y).powi(2);
                                let db = (b.1 - unit.x).powi(2) + (b.2 - unit.y).powi(2);
                                da.partial_cmp(&db).unwrap()
                            });
                            if let Some((nid, _, _)) = nearest {
                                unit.order = UnitOrder::Harvest {
                                    target_mineral_id: *nid,
                                    phase: HarvestPhase::MovingToNode,
                                };
                            } else {
                                unit.order = UnitOrder::Idle;
                            }
                        }
                    }
                    HarvestPhase::Mining(mut t) => {
                        let has_drill = has_drills[u_team_idx];
                        let mining_duration = if has_drill {
                            engine.config.economy.worker_mining_time_upgraded
                        } else {
                            engine.config.economy.worker_mining_time
                        };
                        t += dt;
                        if t >= mining_duration {
                            unit.carried_minerals = if has_drill {
                                engine.config.economy.worker_carry_capacity_upgraded
                            } else {
                                engine.config.economy.worker_carry_capacity
                            };
                            // Trừ trữ lượng mỏ quặng
                            if let Some(m) = engine
                                .minerals
                                .iter_mut()
                                .find(|m| m.id == target_mineral_id)
                            {
                                m.amount -= unit.carried_minerals;
                            }
                            unit.order = UnitOrder::Harvest {
                                target_mineral_id,
                                phase: HarvestPhase::ReturningToBase,
                            };
                        } else {
                            unit.order = UnitOrder::Harvest {
                                target_mineral_id,
                                phase: HarvestPhase::Mining(t),
                            };
                        }
                    }
                    HarvestPhase::ReturningToBase => {
                        // Tìm Command Center gần nhất của phe mình để giao khoáng sản
                        let my_cc = cc_targets
                            .iter()
                            .filter(|(t, _, _)| factions[t.index() % 8] == my_faction)
                            .min_by(|a, b| {
                                let da = (a.1 - unit.x).powi(2) + (a.2 - unit.y).powi(2);
                                let db = (b.1 - unit.x).powi(2) + (b.2 - unit.y).powi(2);
                                da.partial_cmp(&db).unwrap()
                            });

                        if let Some((_, cx, cy)) = my_cc {
                            let dx = *cx - unit.x;
                            let dy = *cy - unit.y;
                            let dist = (dx * dx + dy * dy).sqrt();
                            if dist > 55.0 {
                                let move_dist = (cur_speed * dt).min(dist);
                                unit.x += (dx / dist) * move_dist;
                                unit.y += (dy / dist) * move_dist;
                                unit.walk_cycle += dt * 10.0;
                                unit.facing_right = dx >= 0.0;
                            } else {
                                // Giao hàng thành công!
                                let carried = unit.carried_minerals;
                                mineral_deliveries.push((unit.team, carried, unit.x, unit.y - 15.0));
                                unit.carried_minerals = 0.0;
                                unit.order = UnitOrder::Harvest {
                                    target_mineral_id,
                                    phase: HarvestPhase::MovingToNode,
                                };
                            }
                        }
                    }
                }
            }
            UnitOrder::Build { target_building_id } => {
                let target_building = building_targets
                    .iter()
                    .find(|(id, _, _, _, _, _)| *id == target_building_id);

                let is_still_building = if let Some((_, _, bx, by, _, build_prog)) = target_building {
                    if *build_prog < 1.0 {
                        let dx = bx - unit.x;
                        let dy = by - unit.y;
                        let dist = (dx * dx + dy * dy).sqrt();
                        if dist > 60.0 {
                            let move_dist = (cur_speed * dt).min(dist);
                            unit.x += (dx / dist) * move_dist;
                            unit.y += (dy / dist) * move_dist;
                            unit.walk_cycle += dt * 10.0;
                            unit.facing_right = dx >= 0.0;
                        }
                        true
                    } else {
                        false // Công trình đã hoàn thành (build_prog >= 1.0)
                    }
                } else {
                    false // Công trình không tồn tại nữa
                };

                if !is_still_building {
                    // Công trình đã xây xong! Engineer lập tức chuyển sang khai thác mỏ quặng gần nhất
                    let nearest_mineral = mineral_nodes
                        .iter()
                        .min_by(|a, b| {
                            let da = (a.1 - unit.x).powi(2) + (a.2 - unit.y).powi(2);
                            let db = (b.1 - unit.x).powi(2) + (b.2 - unit.y).powi(2);
                            da.partial_cmp(&db).unwrap()
                        })
                        .map(|m| m.0);

                    if let Some(target_mineral_id) = nearest_mineral {
                        unit.order = UnitOrder::Harvest {
                            target_mineral_id,
                            phase: HarvestPhase::MovingToNode,
                        };
                    } else {
                        unit.order = UnitOrder::Idle;
                    }
                }
            }
            UnitOrder::AttackTarget {
                target_id,
                is_building,
            } => {
                let target_info = if is_building {
                    building_targets
                        .iter()
                        .find(|(id, t, _, _, _, _)| *id == target_id && factions[t.index() % 8] != my_faction)
                        .map(|(_, _, x, y, r, _)| (*x, *y, *r))
                } else {
                    unit_targets
                        .iter()
                        .find(|(id, t, _, _)| *id == target_id && factions[t.index() % 8] != my_faction)
                        .map(|(_, _, x, y)| (*x, *y, 10.0))
                };

                if let Some((tx, ty, target_radius)) = target_info {
                    let dx = tx - unit.x;
                    let dy = ty - unit.y;
                    let dist = (dx * dx + dy * dy).sqrt();
                    unit.facing_right = dx >= 0.0;

                    // KHOẢNG CÁCH TÍNH TỚI MÉP MỤC TIÊU
                    let effective_dist = (dist - target_radius).max(0.0);

                    if effective_dist > unit.attack_range {
                        // Di chuyển tiếp cận mép mục tiêu
                        let move_dist =
                            (cur_speed * dt).min(effective_dist - unit.attack_range + 5.0);
                        unit.x += (dx / dist) * move_dist;
                        unit.y += (dy / dist) * move_dist;
                        unit.walk_cycle += dt * 10.0;
                    } else {
                        // TẤN CÔNG ĐÍCH XÁC
                        if unit.current_cooldown <= 0.0 {
                            unit.current_cooldown = unit.attack_cooldown;
                            let mut dmg = unit.attack_damage;
                            let has_mag = has_magnetic[u_team_idx];
                            let has_super = has_superheated[u_team_idx];
                            if unit.unit_type == UnitType::Rifleman && has_mag {
                                dmg *= 1.25;
                            }
                            if unit.unit_type == UnitType::ShockTrooper && has_super {
                                dmg *= 1.35;
                            }

                            // Điểm va chạm nằm ở mép mục tiêu
                            let hit_dist = dist.min(unit.attack_range + target_radius);
                            let hit_x = if dist > 0.0 {
                                unit.x + (dx / dist) * hit_dist
                            } else {
                                tx
                            };
                            let hit_y = if dist > 0.0 {
                                unit.y + (dy / dist) * hit_dist
                            } else {
                                ty
                            };

                            if unit.unit_type == UnitType::AcidBaneling {
                                // Bọ nổ cảm tử: nổ tung ngay lập tức
                                unit.hp = 0.0;
                                new_projectiles.push(Projectile {
                                    id: 0,
                                    team: unit.team,
                                    x: unit.x,
                                    y: unit.y,
                                    target_x: hit_x,
                                    target_y: hit_y,
                                    speed: 1500.0,
                                    damage: dmg,
                                    damage_type: DamageType::Explosive,
                                    aoe_radius: unit.aoe_radius,
                                    is_laser: false,
                                    lifetime: 0.1,
                                });
                            } else if unit.unit_type == UnitType::ShockTrooper
                                || unit.unit_type == UnitType::Vanguard
                                || unit.unit_type == UnitType::Engineer
                                || unit.unit_type == UnitType::BehemothTitan
                            {
                                // Cận chiến: tạo vệt chém hoặc dậm chấn động
                                new_projectiles.push(Projectile {
                                    id: 0,
                                    team: unit.team,
                                    x: unit.x,
                                    y: unit.y,
                                    target_x: hit_x,
                                    target_y: hit_y,
                                    speed: 900.0,
                                    damage: dmg,
                                    damage_type: unit.damage_type,
                                    aoe_radius: unit.aoe_radius,
                                    is_laser: false,
                                    lifetime: 0.15,
                                });
                            } else {
                                // Tầm xa
                                let is_laser = unit.unit_type == UnitType::LaserMarksman;
                                let speed = if is_laser {
                                    1800.0
                                } else if unit.unit_type == UnitType::TaoistSwordmaster {
                                    1100.0
                                } else if unit.unit_type == UnitType::DarkRocketeer {
                                    650.0
                                } else {
                                    750.0
                                };
                                new_projectiles.push(Projectile {
                                    id: 0,
                                    team: unit.team,
                                    x: unit.x,
                                    y: unit.y - 12.0,
                                    target_x: hit_x,
                                    target_y: hit_y,
                                    speed,
                                    damage: dmg,
                                    damage_type: unit.damage_type,
                                    aoe_radius: unit.aoe_radius,
                                    is_laser,
                                    lifetime: if is_laser { 0.25 } else { 1.5 },
                                });
                            }
                        }
                    }
                } else {
                    // Mục tiêu đã bị hạ gục! Nếu là đơn vị chiến đấu, tiếp tục AttackMove về căn cứ thù địch gần nhất
                    if unit.unit_type != UnitType::Engineer {
                        let nearest_enemy_cc = cc_targets
                            .iter()
                            .filter(|(t, _, _)| factions[t.index() % 8] != my_faction)
                            .min_by(|a, b| {
                                let da = (a.1 - unit.x).powi(2) + (a.2 - unit.y).powi(2);
                                let db = (b.1 - unit.x).powi(2) + (b.2 - unit.y).powi(2);
                                da.partial_cmp(&db).unwrap()
                            });

                        let (tx, ty) = if let Some((_, ex, ey)) = nearest_enemy_cc {
                            (*ex + unit.formation_offset.0, *ey + unit.formation_offset.1)
                        } else {
                            (MAP_WIDTH * 0.5 + unit.formation_offset.0, MAP_HEIGHT * 0.5 + unit.formation_offset.1)
                        };

                        unit.order = UnitOrder::AttackMove {
                            x: tx.clamp(60.0, MAP_WIDTH - 60.0),
                            y: ty.clamp(60.0, MAP_HEIGHT - 60.0),
                        };
                    } else {
                        unit.order = UnitOrder::Idle;
                    }
                }
            }
        }
    }

    // Giao khoáng sản vào kinh tế từng team
    for (team, amount, fx, fy) in mineral_deliveries {
        let eco = match team {
            Team::Player => {
                engine.minerals_mined_accumulator += amount;
                new_floating_texts.push(FloatingText {
                    id: 0,
                    text: format!("+{} 💎", amount as u32),
                    x: fx,
                    y: fy,
                    color: "#00e5ff".to_string(),
                    lifetime: 1.0,
                    max_lifetime: 1.0,
                });
                &mut engine.player_eco
            }
            Team::Enemy => &mut engine.enemy_eco,
            _ => &mut engine.extra_ecos[team.index() - 2],
        };
        eco.minerals += amount;
    }

    // Xóa lính chết do tự nổ
    engine.units.retain(|u| u.hp > 0.0);

    // Hồi máu linh quang từ Kim cang hộ thể (3 HP/s)
    for u in engine.units.iter_mut() {
        let t_idx = u.team.index() % 8;
        if has_bell[t_idx] && u.hp > 0.0 && u.hp < u.max_hp {
            u.hp = (u.hp + 3.0 * dt).min(u.max_hp);
        }
    }

    for mut p in new_projectiles {
        p.id = engine.alloc_id();
        engine.projectiles.push(p);
    }

    for mut ft in new_floating_texts {
        ft.id = engine.alloc_id();
        engine.floating_texts.push(ft);
    }

    // Xóa quặng đã khai thác hết
    engine.minerals.retain(|m| m.amount > 0.0);

    // Xóa lính tử trận
    engine.units.retain(|u| u.hp > 0.0);
}

pub fn calculate_building_construction_step(
    build_progress: f32,
    under_attack_timer: f32,
    is_worker_near: bool,
    dt: f32,
) -> f32 {
    let mut build_speed = if is_worker_near {
        crate::config::BUILD_SPEED_WORKER
    } else {
        crate::config::BUILD_SPEED_BASE
    };
    if under_attack_timer > 0.0 {
        build_speed *= crate::config::BUILDING_UNDER_ATTACK_SPEED_MULTIPLIER;
    }
    (build_progress + build_speed * dt).min(1.0)
}

fn update_buildings(engine: &mut GameEngine, dt: f32) {
    let mut new_spawned_units = Vec::new();
    let mut new_turret_projectiles = Vec::new();
    let factions = engine.team_factions();
    let team_techs: [TechTreeState; 8] =
        std::array::from_fn(|i| engine.team_tech(Team::from_index(i)).clone());

    let engineer_pos: Vec<(Team, f32, f32)> = engine
        .units
        .iter()
        .filter(|u| u.unit_type == UnitType::Engineer)
        .map(|u| (u.team, u.x, u.y))
        .collect();

    let enemy_unit_targets: Vec<(Team, f32, f32)> =
        engine.units.iter().map(|u| (u.team, u.x, u.y)).collect();
    let enemy_building_targets: Vec<(Team, f32, f32)> = engine
        .buildings
        .iter()
        .map(|b| (b.team, b.x, b.y))
        .collect();

    for b in engine.buildings.iter_mut() {
        if b.under_attack_timer > 0.0 {
            b.under_attack_timer = (b.under_attack_timer - dt).max(0.0);
        }

        let b_team_tech = &team_techs[b.team.index() % 8];

        if b.build_progress < 1.0 {
            let is_worker_near = engineer_pos.iter().any(|(t, x, y)| {
                *t == b.team
                    && ((*x - b.x).powi(2) + (*y - b.y).powi(2)).sqrt()
                        <= crate::config::WORKER_BUILD_RANGE
            });
            let next_progress = calculate_building_construction_step(
                b.build_progress,
                b.under_attack_timer,
                is_worker_near,
                dt,
            );
            let added_hp = b.max_hp * (next_progress - b.build_progress).max(0.0);
            b.build_progress = next_progress;
            b.hp = (b.hp + added_hp).min(b.max_hp * b.build_progress);
        } else {
            // Huấn luyện lính
            if !b.training_queue.is_empty() {
                b.training_timer += dt;
                let mut required_time = engine.config.get_unit_training_time(b.training_queue[0]);
                if b_team_tech.hive_proliferation {
                    required_time *= 0.65;
                }
                if b.training_timer >= required_time {
                    let u_type = b.training_queue.remove(0);
                    b.training_timer = 0.0;
                    new_spawned_units.push((
                        b.team,
                        u_type,
                        b.x + 10.0,
                        b.y + 40.0,
                        b.rally_x,
                        b.rally_y,
                    ));
                }
            }

            // Công trình phòng thủ / tấn công tự động
            if let Some(p) = process_building_combat_with_factions(
                b,
                engine.config.get_building(b.building_type),
                &b_team_tech,
                Some(&factions),
                &enemy_unit_targets,
                &enemy_building_targets,
                dt,
            ) {
                new_turret_projectiles.push(p);
            }
        }
    }

    for (team, u_type, sx, sy, rx, ry) in new_spawned_units {
        let u_id = engine.alloc_id();
        let mut unit = engine.config.create_unit(u_id, team, u_type, sx, sy);
        unit.target_x = rx;
        unit.target_y = ry;
        unit.facing_right = rx >= sx;

        // Nếu là Engineer, tự động tìm mỏ quặng gần nhất để khai thác ngay lập tức
        if u_type == UnitType::Engineer {
            let nearest_mineral = engine
                .minerals
                .iter()
                .filter(|m| m.amount > 0.0)
                .min_by(|a, b| {
                    let da = (a.x - sx).powi(2) + (a.y - sy).powi(2);
                    let db = (b.x - sx).powi(2) + (b.y - sy).powi(2);
                    da.partial_cmp(&db).unwrap()
                })
                .map(|m| m.id);

            if let Some(target_mineral_id) = nearest_mineral {
                unit.order = UnitOrder::Harvest {
                    target_mineral_id,
                    phase: HarvestPhase::MovingToNode,
                };
            } else {
                unit.order = UnitOrder::MoveTo { x: rx, y: ry };
            }
        } else {
            unit.order = UnitOrder::MoveTo { x: rx, y: ry };
        }

        // Áp dụng nâng cấp công nghệ (Tech upgrades cho team)
        let team_tech = engine.team_tech(team);
        if u_type == UnitType::ShockTrooper && team_tech.energy_shielding {
            unit.hp += 30.0;
            unit.max_hp += 30.0;
        }
        if team_tech.exo_boosters || team_tech.hive_proliferation {
            unit.speed *= 1.25;
        }
        if team_tech.titan_alloy_plating {
            unit.resistances.physical = (unit.resistances.physical + 0.25).min(0.80);
        }
        if team_tech.golden_bell_shield {
            unit.resistances.energy = (unit.resistances.energy + 0.25).min(0.80);
            unit.resistances.arcane = (unit.resistances.arcane + 0.30).min(0.80);
        }
        if team_tech.primeval_scales
            && (u_type == UnitType::ShockTrooper
                || u_type == UnitType::Vanguard
                || u_type == UnitType::BehemothTitan
                || u_type == UnitType::AcidBaneling)
        {
            unit.resistances.physical = (unit.resistances.physical + 0.25).min(0.80);
            unit.resistances.explosive = (unit.resistances.explosive + 0.25).min(0.80);
        }

        engine.units.push(unit);
    }

    for mut p in new_turret_projectiles {
        p.id = engine.alloc_id();
        engine.projectiles.push(p);
    }

    // Xóa nhà sập
    engine.buildings.retain(|b| b.hp > 0.0);
}

pub fn process_building_combat(
    b: &mut Building,
    b_cfg: Option<&BuildingStatsConfig>,
    player_tech: &TechTreeState,
    enemy_tech: &TechTreeState,
    enemy_unit_targets: &[(Team, f32, f32)],
    enemy_building_targets: &[(Team, f32, f32)],
    dt: f32,
) -> Option<Projectile> {
    let b_tech = if b.team == Team::Player {
        player_tech
    } else {
        enemy_tech
    };
    process_building_combat_with_factions(
        b,
        b_cfg,
        b_tech,
        None,
        enemy_unit_targets,
        enemy_building_targets,
        dt,
    )
}

pub fn process_building_combat_with_factions(
    b: &mut Building,
    b_cfg: Option<&BuildingStatsConfig>,
    b_tech: &TechTreeState,
    factions: Option<&[u8; 8]>,
    enemy_unit_targets: &[(Team, f32, f32)],
    enemy_building_targets: &[(Team, f32, f32)],
    dt: f32,
) -> Option<Projectile> {
    let range = b_cfg.map(|c| c.turret_range).unwrap_or(0.0);
    let base_damage = b_cfg.map(|c| c.turret_damage).unwrap_or(0.0);

    if range <= 0.0 || base_damage <= 0.0 {
        return None;
    }

    if b.current_cooldown > 0.0 {
        b.current_cooldown -= dt;
        return None;
    }

    let base_cd = b_cfg.map(|c| c.turret_cooldown).unwrap_or(1.0).max(0.1);
    let base_dmg_type = b_cfg.map(|c| c.turret_damage_type).unwrap_or(b.damage_type);
    let base_speed = b_cfg
        .map(|c| c.turret_projectile_speed)
        .unwrap_or(900.0)
        .max(100.0);

    let is_hostile = |t: Team| {
        if let Some(factions) = factions {
            let my_faction = factions[b.team.index() % 8];
            factions[t.index() % 8] != my_faction
        } else {
            t != b.team
        }
    };

    // Ưu tiên ngắm lính đối phương gần nhất, nếu không có mới ngắm công trình đối phương
    let target_pos = enemy_unit_targets
        .iter()
        .find(|(t, x, y)| is_hostile(*t) && ((*x - b.x).powi(2) + (*y - b.y).powi(2)).sqrt() <= range)
        .or_else(|| {
            enemy_building_targets.iter().find(|(t, x, y)| {
                is_hostile(*t) && ((*x - b.x).powi(2) + (*y - b.y).powi(2)).sqrt() <= range
            })
        });

    if let Some((_, ex, ey)) = target_pos {
        let has_overload = b_tech.overload_strike;
        b.current_cooldown = if has_overload { base_cd * 0.6 } else { base_cd };

        let has_dark_rockets = b_tech.dark_matter_rockets;

        let spawn_y = b.y - (b.height * 0.25).min(20.0);
        let is_laser = !has_dark_rockets && base_dmg_type == DamageType::Energy;

        Some(Projectile {
            id: 0,
            team: b.team,
            x: b.x,
            y: spawn_y,
            target_x: *ex,
            target_y: *ey,
            speed: if has_dark_rockets {
                base_speed * (750.0 / 900.0)
            } else {
                base_speed
            },
            damage: if has_dark_rockets {
                base_damage * 1.4
            } else {
                base_damage
            },
            damage_type: if has_dark_rockets {
                DamageType::Explosive
            } else {
                base_dmg_type
            },
            aoe_radius: if has_dark_rockets { 40.0 } else { 0.0 },
            is_laser,
            lifetime: if has_dark_rockets {
                1.2
            } else if is_laser {
                0.3
            } else {
                1.0
            },
        })
    } else {
        None
    }
}

pub fn calculate_effective_damage(raw_dmg: f32, dmg_type: DamageType, res: &Resistances) -> f32 {
    let r = match dmg_type {
        DamageType::Physical => res.physical,
        DamageType::Energy => res.energy,
        DamageType::Explosive => res.explosive,
        DamageType::Arcane => res.arcane,
    };
    let factor = (1.0 - r.clamp(-0.5, 0.85)).max(0.1);
    (raw_dmg * factor).max(1.0)
}

fn update_projectiles(engine: &mut GameEngine, dt: f32) {
    let factions = engine.team_factions();
    let has_corrosive: [bool; 8] = std::array::from_fn(|i| {
        engine.team_tech(Team::from_index(i)).corrosive_curse
    });
    let mut new_texts = Vec::new();

    for p in engine.projectiles.iter_mut() {
        p.lifetime -= dt;
        let dx = p.target_x - p.x;
        let dy = p.target_y - p.y;
        let dist = (dx * dx + dy * dy).sqrt();
        if dist > 10.0 {
            let step = (p.speed * dt).min(dist);
            p.x += (dx / dist) * step;
            p.y += (dy / dist) * step;
        }

        if dist <= 18.0 || p.lifetime <= 0.0 {
            let is_aoe = p.aoe_radius > 0.0;
            let aoe_range = if is_aoe { p.aoe_radius.max(30.0) } else { 24.0 };
            let p_team_idx = p.team.index() % 8;
            let p_faction = factions[p_team_idx];

            // Sát thương lính địch
            for u in engine.units.iter_mut() {
                if factions[u.team.index() % 8] != p_faction {
                    let d = ((u.x - p.target_x).powi(2) + (u.y - p.target_y).powi(2)).sqrt();
                    if d <= aoe_range {
                        let falloff = if is_aoe {
                            (1.0 - (d / aoe_range) * 0.35).max(0.65)
                        } else {
                            1.0
                        };
                        let taken = calculate_effective_damage(
                            p.damage * falloff,
                            p.damage_type,
                            &u.resistances,
                        );
                        u.hp -= taken;

                        if has_corrosive[p_team_idx] {
                            u.resistances.physical = (u.resistances.physical - 0.04).max(0.0);
                            u.resistances.energy = (u.resistances.energy - 0.04).max(0.0);
                            u.resistances.explosive = (u.resistances.explosive - 0.04).max(0.0);
                            u.resistances.arcane = (u.resistances.arcane - 0.04).max(0.0);
                        }

                        new_texts.push(FloatingText {
                            id: 0,
                            text: format!("-{:.0}", taken),
                            x: u.x,
                            y: u.y - 12.0,
                            color: p.damage_type.color_code().to_string(),
                            lifetime: 0.7,
                            max_lifetime: 0.7,
                        });

                        if !is_aoe {
                            break;
                        }
                    }
                }
            }

            // Sát thương công trình địch
            for b in engine.buildings.iter_mut() {
                if factions[b.team.index() % 8] != p_faction {
                    let b_rad = b.width.max(b.height) * 0.5;
                    let dist_to_b =
                        ((b.x - p.target_x).powi(2) + (b.y - p.target_y).powi(2)).sqrt();
                    if dist_to_b <= b_rad + aoe_range {
                        let taken =
                            calculate_effective_damage(p.damage, p.damage_type, &b.resistances);
                        b.hp -= taken;
                        b.under_attack_timer = crate::config::BUILDING_UNDER_ATTACK_DURATION;

                        new_texts.push(FloatingText {
                            id: 0,
                            text: format!("-{:.0}", taken),
                            x: b.x,
                            y: b.y - b_rad,
                            color: p.damage_type.color_code().to_string(),
                            lifetime: 0.8,
                            max_lifetime: 0.8,
                        });

                        if !is_aoe {
                            break;
                        }
                    }
                }
            }

            p.lifetime = -1.0;
        }
    }

    for mut ft in new_texts {
        ft.id = engine.alloc_id();
        engine.floating_texts.push(ft);
    }

    engine.projectiles.retain(|p| p.lifetime > 0.0);
}

fn update_floating_texts(engine: &mut GameEngine, dt: f32) {
    for ft in engine.floating_texts.iter_mut() {
        ft.lifetime -= dt;
        ft.y -= 25.0 * dt;
    }
    engine.floating_texts.retain(|ft| ft.lifetime > 0.0);
}

pub fn update_fog(engine: &mut GameEngine) {
    engine.fog.reset_visibility_to_explored();
    let player_faction = engine.get_faction(Team::Player);

    // Tầm nhìn quanh công trình
    for b in &engine.buildings {
        if engine.get_faction(b.team) == player_faction {
            let range = engine
                .config
                .get_building(b.building_type)
                .map(|c| c.turret_range)
                .unwrap_or(0.0);
            let r = if range > 0.0 {
                (range + 50.0).max(320.0)
            } else {
                240.0
            };
            engine.fog.reveal_circle(b.x, b.y, r);
        }
    }

    // Tầm nhìn quanh lính
    for u in &engine.units {
        if engine.get_faction(u.team) == player_faction {
            let r = match u.unit_type {
                UnitType::LaserMarksman => 260.0,
                UnitType::Rifleman => 200.0,
                _ => 170.0,
            };
            engine.fog.reveal_circle(u.x, u.y, r);
        }
    }
}

fn check_game_over(engine: &mut GameEngine) {
    let player_faction = engine.get_faction(Team::Player);
    let my_faction_has_cc = engine.buildings.iter().any(|b| {
        engine.get_faction(b.team) == player_faction
            && b.building_type == BuildingType::CommandCenter
    });
    let enemy_factions_have_cc = engine.buildings.iter().any(|b| {
        engine.get_faction(b.team) != player_faction
            && b.building_type == BuildingType::CommandCenter
    });

    if !my_faction_has_cc {
        engine.game_over = true;
        engine.outgoing_events.push(EngineToReactEvent::GameOver {
            player_won: false,
            stats_json: "{}".to_string(),
        });
        engine.outgoing_events.push(EngineToReactEvent::PlaySound {
            sound: "defeat".to_string(),
        });
    } else if !enemy_factions_have_cc {
        engine.game_over = true;
        engine.outgoing_events.push(EngineToReactEvent::GameOver {
            player_won: true,
            stats_json: "{}".to_string(),
        });
        engine.outgoing_events.push(EngineToReactEvent::PlaySound {
            sound: "victory".to_string(),
        });
    }
}

pub fn send_sync_event(engine: &mut GameEngine) {
    let tech_json = serde_json::to_string(&engine.player_tech).unwrap_or_default();
    let autoplay_json = serde_json::to_string(&engine.autoplay.config).unwrap_or_default();
    let current_form = engine.team_formation(Team::Player).button_label().to_string();
    engine.outgoing_events.push(EngineToReactEvent::SyncState {
        player_minerals: engine.player_eco.minerals,
        player_energy: engine.player_eco.energy,
        player_supply: engine.player_eco.supply,
        player_max_supply: engine.player_eco.max_supply,
        tech_state: tech_json,
        autoplay_state: autoplay_json,
        player_formation: current_form,
        game_time: engine.game_time,
        is_paused: engine.is_paused,
    });
}

// Xử lý Input từ Chuột và Phím
pub fn handle_mouse_down(engine: &mut GameEngine, button: i32, cx: f32, cy: f32) {
    let vh = engine.camera.viewport_h;

    // Kiểm tra bấm vào Minimap để kéo thả camera
    if (10.0..=170.0).contains(&cx) && (vh - 170.0..=vh - 10.0).contains(&cy) {
        engine.is_minimap_dragging = true;
        let norm_x = ((cx - 10.0) / 160.0).clamp(0.0, 1.0);
        let norm_y = ((cy - (vh - 170.0)) / 160.0).clamp(0.0, 1.0);
        engine.camera.x = norm_x * MAP_WIDTH;
        engine.camera.y = norm_y * MAP_HEIGHT;
        return;
    }

    if button == 0 {
        // Chuột trái
        if engine.is_attack_move_mode {
            engine.is_attack_move_mode = false;
            let (wx, wy) = engine.screen_to_world(cx, cy);
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

            if !selected_pairs.is_empty() {
                let start_center = {
                    let mut sum_x = 0.0;
                    let mut sum_y = 0.0;
                    let mut count = 0.0;
                    for uid in &engine.selected_unit_ids {
                        if let Some(u) = engine.units.iter().find(|u| u.id == *uid && u.team == Team::Player) {
                            sum_x += u.x;
                            sum_y += u.y;
                            count += 1.0;
                        }
                    }
                    if count > 0.0 { (sum_x / count, sum_y / count) } else { (wx, wy) }
                };

                let form = engine.team_formation(Team::Player);
                let form_positions = crate::formation::compute_formation_positions(
                    &selected_pairs,
                    start_center,
                    (wx, wy),
                    form,
                );

                for (uid, target_x, target_y, off_x, off_y) in form_positions {
                    if let Some(u) = engine.units.iter_mut().find(|u| u.id == uid) {
                        u.formation_offset = (off_x, off_y);
                        u.order = UnitOrder::AttackMove {
                            x: target_x,
                            y: target_y,
                        };
                    }
                }
            }
            engine.outgoing_events.push(EngineToReactEvent::PlaySound {
                sound: "attack_order".to_string(),
            });
            return;
        }

        engine.is_mouse_down = true;
        engine.mouse_down_start = (cx, cy);
        engine.selection_box = Some((cx, cy, cx, cy));

        // Kiểm tra xem có bấm vào các nút trên HUD của Bevy không
        if check_bevy_hud_click(engine, cx, cy) {
            engine.selection_box = None;
            engine.is_mouse_down = false;
        }
    } else if button == 2 {
        let vw = engine.camera.viewport_w;
        let vh = engine.camera.viewport_h;

        // Bấm chuột phải trên Command Card để hủy bớt lính trong hàng đợi và hoàn tiền
        if cx >= vw - 220.0 && cx <= vw - 10.0 && cy >= vh - 140.0 && cy <= vh - 10.0 {
            handle_command_card_right_click(engine, cx - (vw - 220.0), cy - (vh - 140.0));
            return;
        }

        // Chuột phải: Hủy Attack-Move hoặc ra lệnh di chuyển/tấn công/khai thác
        engine.is_attack_move_mode = false;
        let (wx, wy) = engine.screen_to_world(cx, cy);
        issue_right_click_order(engine, wx, wy);
    }
}

pub fn handle_mouse_up(engine: &mut GameEngine, button: i32, cx: f32, cy: f32) {
    if engine.is_minimap_dragging {
        engine.is_minimap_dragging = false;
        return;
    }

    if button == 0 && engine.is_mouse_down {
        engine.is_mouse_down = false;
        let (sx1, sy1) = engine.mouse_down_start;
        let (sx2, cy) = (cx, cy);

        let dx = (sx2 - sx1).abs();
        let dy = (cy - sy1).abs();

        if let Some(b_type) = engine.build_mode {
            // Đặt móng xây nhà
            let (wx, wy) = engine.screen_to_world(cx, cy);
            try_place_building(engine, b_type, wx, wy);
            engine.build_mode = None;
        } else if dx > 8.0 || dy > 8.0 {
            // Box selection
            let (w_min_x, w_min_y) = engine.screen_to_world(sx1.min(sx2), sy1.min(cy));
            let (w_max_x, w_max_y) = engine.screen_to_world(sx1.max(sx2), sy1.max(cy));

            engine.selected_unit_ids = engine
                .units
                .iter()
                .filter(|u| {
                    u.team == Team::Player
                        && u.x >= w_min_x
                        && u.x <= w_max_x
                        && u.y >= w_min_y
                        && u.y <= w_max_y
                })
                .map(|u| u.id)
                .collect();
            engine.selected_building_id = None;
        } else {
            // Single click selection
            let (wx, wy) = engine.screen_to_world(cx, cy);
            select_at_point(engine, wx, wy);
        }

        engine.selection_box = None;
    }
}

pub fn handle_mouse_move(engine: &mut GameEngine, cx: f32, cy: f32) {
    engine.mouse_screen_x = cx;
    engine.mouse_screen_y = cy;

    let vh = engine.camera.viewport_h;
    let vw = engine.camera.viewport_w;

    if engine.is_minimap_dragging {
        let norm_x = ((cx - 10.0) / 160.0).clamp(0.0, 1.0);
        let norm_y = ((cy - (vh - 170.0)) / 160.0).clamp(0.0, 1.0);
        engine.camera.x = norm_x * MAP_WIDTH;
        engine.camera.y = norm_y * MAP_HEIGHT;
        return;
    }

    if engine.is_mouse_down {
        let (sx1, sy1) = engine.mouse_down_start;
        engine.selection_box = Some((sx1, sy1, cx, cy));
    }

    // Hover trên Command Card
    if cx >= vw - 225.0 && cx <= vw - 10.0 && cy >= vh - 150.0 && cy <= vh - 10.0 {
        let rel_x = cx - (vw - 225.0);
        let rel_y = cy - (vh - 150.0);
        let col = ((rel_x - 5.0).max(0.0) / 67.0).min(2.0) as usize;
        let row = ((rel_y - 6.0).max(0.0) / 44.0).min(2.0) as usize;
        engine.hovered_command_button = Some(row * 3 + col);
    } else {
        engine.hovered_command_button = None;
    }
}

pub fn handle_hotkeys(engine: &mut GameEngine, key: &str) {
    match key {
        "KeyA" => {
            // Bật chế độ Attack-Move
            if !engine.selected_unit_ids.is_empty() {
                engine.is_attack_move_mode = true;
                engine.outgoing_events.push(EngineToReactEvent::PlaySound {
                    sound: "click".to_string(),
                });
            }
        }
        "KeyT" => {
            // Mở Modal Cây Công Nghệ trong React
            if engine.settings.auto_pause_on_tech {
                engine.is_paused = true;
            }
            engine
                .outgoing_events
                .push(EngineToReactEvent::OpenTechTreeModal);
            send_sync_event(engine);
        }
        "KeyB" => {
            // Mở Bảng Chế độ Tự Động (Auto-Play)
            engine
                .outgoing_events
                .push(EngineToReactEvent::OpenAutoPlayModal);
            send_sync_event(engine);
        }
        "KeyF" => {
            // Chuyển đổi nhanh Đội hình chiến thuật kế tiếp
            let next_form = engine.team_formation(Team::Player).next();
            set_player_formation_and_reform(engine, next_form);
            engine.outgoing_events.push(EngineToReactEvent::PlaySound {
                sound: "click".to_string(),
            });
            send_sync_event(engine);
        }
        "KeyC" | "KeyH" => {
            // Xây Nhà Chính CommandCenter nếu đang chọn Engineer
            let has_worker = engine.selected_unit_ids.iter().any(|uid| {
                engine
                    .units
                    .iter()
                    .any(|u| u.id == *uid && u.unit_type == UnitType::Engineer && u.team == Team::Player)
            });
            if has_worker {
                engine.build_mode = Some(BuildingType::CommandCenter);
                engine.outgoing_events.push(EngineToReactEvent::PlaySound {
                    sound: "click".to_string(),
                });
            }
        }
        "Escape" => {
            if engine.is_attack_move_mode {
                engine.is_attack_move_mode = false;
            } else if engine.build_mode.is_some() {
                engine.build_mode = None;
            } else {
                engine
                    .outgoing_events
                    .push(EngineToReactEvent::OpenSettingsModal);
            }
        }
        "Space" => {
            // Tạm dừng / Tiếp tục
            engine.is_paused = !engine.is_paused;
        }
        _ => {}
    }
}

fn select_at_point(engine: &mut GameEngine, wx: f32, wy: f32) {
    // Nếu đang chọn Worker và click chuột trái vào mỏ khoáng sản -> Hỗ trợ ra lệnh khai mỏ luôn
    let has_worker_selected = engine.selected_unit_ids.iter().any(|uid| {
        engine
            .units
            .iter()
            .any(|u| u.id == *uid && u.unit_type == UnitType::Engineer && u.team == Team::Player)
    });

    if has_worker_selected {
        let clicked_mineral = engine
            .minerals
            .iter()
            .find(|m| ((m.x - wx).powi(2) + (m.y - wy).powi(2)).sqrt() <= 50.0);
        if let Some(m) = clicked_mineral {
            let m_id = m.id;
            for uid in &engine.selected_unit_ids {
                if let Some(u) = engine
                    .units
                    .iter_mut()
                    .find(|u| u.id == *uid && u.team == Team::Player)
                {
                    if u.unit_type == UnitType::Engineer {
                        u.order = UnitOrder::Harvest {
                            target_mineral_id: m_id,
                            phase: HarvestPhase::MovingToNode,
                        };
                    }
                }
            }
            engine.outgoing_events.push(EngineToReactEvent::PlaySound {
                sound: "order".to_string(),
            });
            return;
        }
    }

    // 1. Kiểm tra bấm vào lính Player (dung sai 24px)
    let clicked_unit = engine
        .units
        .iter()
        .find(|u| ((u.x - wx).powi(2) + (u.y - wy).powi(2)).sqrt() <= 24.0);

    if let Some(u) = clicked_unit {
        engine.selected_unit_ids = vec![u.id];
        engine.selected_building_id = None;
        return;
    }

    // 2. Kiểm tra bấm vào công trình Player (có dung sai)
    let clicked_building = engine.buildings.iter().find(|b| {
        (wx - b.x).abs() <= b.width * 0.5 + 8.0 && (wy - b.y).abs() <= b.height * 0.5 + 8.0
    });

    if let Some(b) = clicked_building {
        engine.selected_building_id = Some(b.id);
        engine.selected_unit_ids.clear();
        return;
    }

    // Bấm ra bãi đất trống
    engine.selected_unit_ids.clear();
    engine.selected_building_id = None;
}

fn issue_right_click_order(engine: &mut GameEngine, wx: f32, wy: f32) {
    if engine.selected_unit_ids.is_empty() {
        if let Some(bid) = engine.selected_building_id {
            if let Some(b) = engine
                .buildings
                .iter_mut()
                .find(|b| b.id == bid && b.team == Team::Player)
            {
                b.rally_x = wx;
                b.rally_y = wy;
                engine.outgoing_events.push(EngineToReactEvent::PlaySound {
                    sound: "click".to_string(),
                });
            }
        }
        return;
    }

    // 1. Kiểm tra bấm vào mỏ khoáng sản (dung sai 55px)
    let clicked_mineral = engine
        .minerals
        .iter()
        .find(|m| ((m.x - wx).powi(2) + (m.y - wy).powi(2)).sqrt() <= 55.0);

    if let Some(m) = clicked_mineral {
        let m_id = m.id;
        for uid in &engine.selected_unit_ids {
            if let Some(u) = engine
                .units
                .iter_mut()
                .find(|u| u.id == *uid && u.team == Team::Player)
            {
                if u.unit_type == UnitType::Engineer {
                    u.order = UnitOrder::Harvest {
                        target_mineral_id: m_id,
                        phase: HarvestPhase::MovingToNode,
                    };
                }
            }
        }
        engine.outgoing_events.push(EngineToReactEvent::PlaySound {
            sound: "order".to_string(),
        });
        return;
    }

    // 2. Kiểm tra bấm vào nhà đang xây dở của mình hoặc đồng minh để thợ mỏ đến xây
    let unfinished_building = engine.buildings.iter().find(|b| {
        engine.are_allies(Team::Player, b.team)
            && b.build_progress < 1.0
            && (wx - b.x).abs() <= b.width * 0.5 + 10.0
            && (wy - b.y).abs() <= b.height * 0.5 + 10.0
    });

    if let Some(b) = unfinished_building {
        let bid = b.id;
        for uid in &engine.selected_unit_ids {
            if let Some(u) = engine
                .units
                .iter_mut()
                .find(|u| u.id == *uid && u.team == Team::Player)
            {
                if u.unit_type == UnitType::Engineer {
                    u.order = UnitOrder::Build {
                        target_building_id: bid,
                    };
                }
            }
        }
        engine.outgoing_events.push(EngineToReactEvent::PlaySound {
            sound: "order".to_string(),
        });
        return;
    }

    // 3. Kiểm tra bấm vào lính địch (Tấn công)
    let enemy_unit = engine.units.iter().find(|u| {
        engine.are_hostile(Team::Player, u.team) && ((u.x - wx).powi(2) + (u.y - wy).powi(2)).sqrt() <= 26.0
    });
    if let Some(eu) = enemy_unit {
        let eid = eu.id;
        for uid in &engine.selected_unit_ids {
            if let Some(u) = engine
                .units
                .iter_mut()
                .find(|u| u.id == *uid && u.team == Team::Player)
            {
                u.order = UnitOrder::AttackTarget {
                    target_id: eid,
                    is_building: false,
                };
            }
        }
        engine.outgoing_events.push(EngineToReactEvent::PlaySound {
            sound: "attack_order".to_string(),
        });
        return;
    }

    // 4. Kiểm tra bấm vào CÔNG TRÌNH ĐỊCH (Tấn công nhà địch hoàn chỉnh)
    let enemy_b = engine.buildings.iter().find(|b| {
        engine.are_hostile(Team::Player, b.team)
            && (wx - b.x).abs() <= b.width * 0.5 + 12.0
            && (wy - b.y).abs() <= b.height * 0.5 + 12.0
    });
    if let Some(eb) = enemy_b {
        let ebid = eb.id;
        for uid in &engine.selected_unit_ids {
            if let Some(u) = engine
                .units
                .iter_mut()
                .find(|u| u.id == *uid && u.team == Team::Player)
            {
                u.order = UnitOrder::AttackTarget {
                    target_id: ebid,
                    is_building: true,
                };
            }
        }
        engine.outgoing_events.push(EngineToReactEvent::PlaySound {
            sound: "attack_order".to_string(),
        });
        return;
    }

    // 5. Di chuyển thông thường tới điểm bấm (dàn đội hình chiến thuật thông minh)
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

    if !selected_pairs.is_empty() {
        let start_center = {
            let mut sum_x = 0.0;
            let mut sum_y = 0.0;
            let mut count = 0.0;
            for uid in &engine.selected_unit_ids {
                if let Some(u) = engine.units.iter().find(|u| u.id == *uid && u.team == Team::Player) {
                    sum_x += u.x;
                    sum_y += u.y;
                    count += 1.0;
                }
            }
            if count > 0.0 { (sum_x / count, sum_y / count) } else { (wx, wy) }
        };

        let form = engine.team_formation(Team::Player);
        let form_positions = crate::formation::compute_formation_positions(
            &selected_pairs,
            start_center,
            (wx, wy),
            form,
        );

        for (uid, target_x, target_y, off_x, off_y) in form_positions {
            if let Some(u) = engine.units.iter_mut().find(|u| u.id == uid) {
                u.formation_offset = (off_x, off_y);
                u.order = UnitOrder::MoveTo {
                    x: target_x,
                    y: target_y,
                };
            }
        }
    }
    engine.outgoing_events.push(EngineToReactEvent::PlaySound {
        sound: "order".to_string(),
    });
}

fn try_place_building(engine: &mut GameEngine, b_type: BuildingType, wx: f32, wy: f32) {
    let (cost_min, cost_en) = engine.config.get_building_cost(b_type);

    if engine.player_eco.minerals >= cost_min && engine.player_eco.energy >= cost_en {
        engine.player_eco.minerals -= cost_min;
        engine.player_eco.energy -= cost_en;
        let bid = engine.alloc_id();

        let mut building = engine
            .config
            .create_building(bid, Team::Player, b_type, wx, wy, false);
        building.build_progress = 0.05;
        building.hp = 1.0;
        building.rally_x = wx + 40.0;
        building.rally_y = wy + 40.0;

        if engine.player_tech.titan_alloy_plating {
            building.resistances.physical = (building.resistances.physical + 0.20).min(0.80);
        }

        let mut hp_mult = 1.0;
        if engine.player_tech.fortified_blueprints {
            hp_mult += 0.3;
        }
        if engine.player_tech.titan_alloy_plating {
            hp_mult += 0.25;
        }
        building.max_hp *= hp_mult;

        let is_armed = engine
            .config
            .get_building(b_type)
            .map(|c| c.turret_damage > 0.0)
            .unwrap_or(false);
        if is_armed && engine.player_tech.dark_matter_rockets {
            building.damage_type = DamageType::Explosive;
        }

        engine.buildings.push(building);

        // Tự động phân công kỹ sư đang chọn đến xây
        for uid in &engine.selected_unit_ids {
            if let Some(u) = engine
                .units
                .iter_mut()
                .find(|u| u.id == *uid && u.unit_type == UnitType::Engineer)
            {
                u.order = UnitOrder::Build {
                    target_building_id: bid,
                };
            }
        }

        engine.outgoing_events.push(EngineToReactEvent::PlaySound {
            sound: "place_building".to_string(),
        });
    }
}

// Bấm nút UI trên HUD Bevy (Góc trên, Minimap, Bảng nút lệnh góc dưới phải)
fn check_bevy_hud_click(engine: &mut GameEngine, sx: f32, sy: f32) -> bool {
    let vw = engine.camera.viewport_w;
    let vh = engine.camera.viewport_h;

    // 0. Nút mở Auto-Play [B] ở thanh trên cùng (x ~ vw - 295, y: 10..42, w: 95)
    if (vw - 295.0..=vw - 200.0).contains(&sx) && (10.0..=42.0).contains(&sy) {
        engine
            .outgoing_events
            .push(EngineToReactEvent::OpenAutoPlayModal);
        send_sync_event(engine);
        return true;
    }

    // 1. Nút mở Tech Tree [T] ở thanh trên cùng (x ~ vw - 190, y: 10..42, w: 90)
    if (vw - 190.0..=vw - 100.0).contains(&sx) && (10.0..=42.0).contains(&sy) {
        if engine.settings.auto_pause_on_tech {
            engine.is_paused = true;
        }
        engine
            .outgoing_events
            .push(EngineToReactEvent::OpenTechTreeModal);
        send_sync_event(engine);
        return true;
    }

    // 2. Nút Cài đặt [ESC] ở thanh trên cùng (x ~ vw - 90, y: 10..42, w: 80)
    if (vw - 90.0..=vw - 10.0).contains(&sx) && (10.0..=42.0).contains(&sy) {
        engine
            .outgoing_events
            .push(EngineToReactEvent::OpenSettingsModal);
        return true;
    }

    // 3. Minimap góc dưới trái (x: 10..170, y: vh - 170..vh - 10)
    if (10.0..=170.0).contains(&sx) && (vh - 170.0..=vh - 10.0).contains(&sy) {
        let norm_x = (sx - 10.0) / 160.0;
        let norm_y = (sy - (vh - 170.0)) / 160.0;
        engine.camera.x = norm_x * MAP_WIDTH;
        engine.camera.y = norm_y * MAP_HEIGHT;
        return true;
    }

    // 4. Command Card góc dưới phải (x: vw - 225..vw - 10, y: vh - 150..vh - 10)
    if (vw - 225.0..=vw - 10.0).contains(&sx) && (vh - 150.0..=vh - 10.0).contains(&sy) {
        handle_command_card_click(engine, sx - (vw - 225.0), sy - (vh - 150.0));
        return true;
    }

    false
}

fn handle_command_card_click(engine: &mut GameEngine, rel_x: f32, rel_y: f32) {
    let col = ((rel_x - 5.0).max(0.0) / 67.0).min(2.0) as usize;
    let row = ((rel_y - 6.0).max(0.0) / 44.0).min(2.0) as usize;
    let btn_idx = row * 3 + col;

    // Nếu đang chọn Worker -> Nút xây nhà
    let has_worker = engine.selected_unit_ids.iter().any(|uid| {
        engine
            .units
            .iter()
            .any(|u| u.id == *uid && u.unit_type == UnitType::Engineer)
    });

    if has_worker {
        match btn_idx {
            0 => engine.build_mode = Some(BuildingType::SupplyDepot),
            1 => engine.build_mode = Some(BuildingType::PowerReactor),
            2 => engine.build_mode = Some(BuildingType::Barracks),
            3 => engine.build_mode = Some(BuildingType::TurretTower),
            4 => engine.build_mode = Some(BuildingType::CommandCenter),
            5 => engine.build_mode = Some(BuildingType::EnergyConverter),
            6 => {
                let next_form = engine.team_formation(Team::Player).next();
                set_player_formation_and_reform(engine, next_form);
                engine.outgoing_events.push(EngineToReactEvent::PlaySound {
                    sound: "click".to_string(),
                });
                send_sync_event(engine);
            }
            _ => {}
        }
        return;
    }

    // Nếu đang chọn Lính chiến đấu (không có Worker) -> 6 nút chọn trực tiếp đội hình
    if !engine.selected_unit_ids.is_empty() {
        if btn_idx < 6 {
            let target_form = FormationType::from_index(btn_idx);
            set_player_formation_and_reform(engine, target_form);
            engine.outgoing_events.push(EngineToReactEvent::PlaySound {
                sound: "click".to_string(),
            });
            send_sync_event(engine);
            return;
        }
    }

    // Nếu đang chọn Nhà -> Mua lính
    if let Some(bid) = engine.selected_building_id {
        let b = engine
            .buildings
            .iter()
            .find(|b| b.id == bid && b.team == Team::Player && b.build_progress >= 1.0)
            .cloned();
        if let Some(b) = b {
            match b.building_type {
                BuildingType::CommandCenter => {
                    if btn_idx == 0 {
                        try_queue_unit(engine, bid, UnitType::Engineer);
                    }
                }
                BuildingType::Barracks => match btn_idx {
                    0 => try_queue_unit(engine, bid, UnitType::ShockTrooper),
                    1 => try_queue_unit(engine, bid, UnitType::Rifleman),
                    2 if engine.player_tech.energy_shielding => {
                        try_queue_unit(engine, bid, UnitType::Vanguard);
                    }
                    3 if engine.player_tech.focused_laser => {
                        try_queue_unit(engine, bid, UnitType::LaserMarksman);
                    }
                    4 if engine.player_tech.dark_matter_rockets => {
                        try_queue_unit(engine, bid, UnitType::DarkRocketeer);
                    }
                    5 if engine.player_tech.celestial_flying_swords => {
                        try_queue_unit(engine, bid, UnitType::TaoistSwordmaster);
                    }
                    6 if engine.player_tech.void_cataclysm => {
                        try_queue_unit(engine, bid, UnitType::VoidWarlock);
                    }
                    7 if engine.player_tech.behemoth_mutation => {
                        try_queue_unit(engine, bid, UnitType::BehemothTitan);
                    }
                    8 if engine.player_tech.acid_banelings => {
                        try_queue_unit(engine, bid, UnitType::AcidBaneling);
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }
}

fn handle_command_card_right_click(engine: &mut GameEngine, rel_x: f32, rel_y: f32) {
    let col = ((rel_x - 5.0).max(0.0) / 67.0).min(2.0) as usize;
    let row = ((rel_y - 6.0).max(0.0) / 44.0).min(2.0) as usize;
    let btn_idx = row * 3 + col;

    if let Some(bid) = engine.selected_building_id {
        let b = engine
            .buildings
            .iter()
            .find(|b| b.id == bid && b.team == Team::Player && b.build_progress >= 1.0)
            .cloned();
        if let Some(b) = b {
            match b.building_type {
                BuildingType::CommandCenter => {
                    if btn_idx == 0 {
                        try_cancel_unit(engine, bid, UnitType::Engineer);
                    }
                }
                BuildingType::Barracks => match btn_idx {
                    0 => try_cancel_unit(engine, bid, UnitType::ShockTrooper),
                    1 => try_cancel_unit(engine, bid, UnitType::Rifleman),
                    2 if engine.player_tech.energy_shielding => {
                        try_cancel_unit(engine, bid, UnitType::Vanguard);
                    }
                    3 if engine.player_tech.focused_laser => {
                        try_cancel_unit(engine, bid, UnitType::LaserMarksman);
                    }
                    4 if engine.player_tech.dark_matter_rockets => {
                        try_cancel_unit(engine, bid, UnitType::DarkRocketeer);
                    }
                    5 if engine.player_tech.celestial_flying_swords => {
                        try_cancel_unit(engine, bid, UnitType::TaoistSwordmaster);
                    }
                    6 if engine.player_tech.void_cataclysm => {
                        try_cancel_unit(engine, bid, UnitType::VoidWarlock);
                    }
                    7 if engine.player_tech.behemoth_mutation => {
                        try_cancel_unit(engine, bid, UnitType::BehemothTitan);
                    }
                    8 if engine.player_tech.acid_banelings => {
                        try_cancel_unit(engine, bid, UnitType::AcidBaneling);
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }
}

fn try_queue_unit(engine: &mut GameEngine, bid: u32, u_type: UnitType) {
    if engine.player_eco.supply >= engine.player_eco.max_supply {
        return; // Hết dân số
    }

    let (mut min_cost, mut en_cost) = engine.config.get_unit_cost(u_type);
    if engine.player_tech.hive_proliferation {
        min_cost *= 0.65;
        en_cost *= 0.65;
    }
    if engine.player_eco.minerals >= min_cost && engine.player_eco.energy >= en_cost {
        if let Some(b) = engine.buildings.iter_mut().find(|b| b.id == bid) {
            if b.training_queue.len() < 5 {
                engine.player_eco.minerals -= min_cost;
                engine.player_eco.energy -= en_cost;
                b.training_queue.push(u_type);
                engine.outgoing_events.push(EngineToReactEvent::PlaySound {
                    sound: "train_unit".to_string(),
                });
            }
        }
    }
}

fn try_cancel_unit(engine: &mut GameEngine, bid: u32, u_type: UnitType) {
    let mut refunded = false;
    let mut spawn_pos = (0.0, 0.0);
    let (mut min_cost, mut en_cost) = engine.config.get_unit_cost(u_type);
    if engine.player_tech.hive_proliferation {
        min_cost *= 0.65;
        en_cost *= 0.65;
    }

    if let Some(b) = engine
        .buildings
        .iter_mut()
        .find(|b| b.id == bid && b.team == Team::Player)
    {
        // Tìm vị trí cuối cùng của đơn vị loại này trong hàng đợi để hủy
        if let Some(idx) = b.training_queue.iter().rposition(|&t| t == u_type) {
            b.training_queue.remove(idx);
            // Nếu đơn vị bị hủy là đơn vị đang chạy ở đầu hàng đợi
            if idx == 0 {
                b.training_timer = 0.0;
            }
            refunded = true;
            spawn_pos = (b.x, b.y);
        }
    }

    if refunded {
        engine.player_eco.minerals += min_cost;
        engine.player_eco.energy += en_cost;
        let ft_id = engine.alloc_id();
        let refund_str = if en_cost > 0.0 {
            format!("+{:.0}💎 +{:.0}⚡", min_cost, en_cost)
        } else {
            format!("+{:.0}💎", min_cost)
        };
        engine.floating_texts.push(FloatingText {
            id: ft_id,
            text: refund_str,
            x: spawn_pos.0,
            y: spawn_pos.1 - 35.0,
            color: "#22c55e".to_string(),
            lifetime: 0.0,
            max_lifetime: 1.2,
        });
        engine.outgoing_events.push(EngineToReactEvent::PlaySound {
            sound: "click".to_string(),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_energy_conversion_normal() {
        // Có 1 nhà chuyển đổi, năng lượng 1200 (> 1000), dt = 1.0
        let (used_en, gained_minerals) = calculate_energy_conversion(1200.0, 1, 1.0);
        assert_eq!(used_en, 30.0);
        assert_eq!(gained_minerals, 25.0);
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
        // Tỷ lệ 25 / 30 = 5 / 6. 10 * 25 / 30 = 8.333333
        assert!((gained_minerals - (10.0 * 25.0 / 30.0)).abs() < 1e-5);
    }

    #[test]
    fn test_energy_conversion_multiple_converters() {
        // 2 nhà chuyển đổi, năng lượng 1200, dt = 1.0 -> Tiêu thụ 60 năng lượng, nhận 50 khoáng sản
        let (used_en, gained_minerals) = calculate_energy_conversion(1200.0, 2, 1.0);
        assert_eq!(used_en, 60.0);
        assert_eq!(gained_minerals, 50.0);
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

        let mut p_tech = TechTreeState::default();
        p_tech.overload_strike = true;
        p_tech.dark_matter_rockets = true;

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
        let ok2 =
            process_start_research(&mut p_tech, &mut p_eco, &config, "celestial_flying_swords");
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
        let completed_early =
            process_update_tech_research(Team::Player, &mut p_tech, &mut buildings, &mut units, &config, 15.0);
        assert!(completed_early.is_empty());
        assert!(!p_tech.focused_laser);
        assert!(!p_tech.celestial_flying_swords);

        // Tiến hành nghiên cứu thêm 10 giây nữa (tổng 25s, cả 2 đều xong)
        let completed_done =
            process_update_tech_research(Team::Player, &mut p_tech, &mut buildings, &mut units, &config, 10.0);
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
}
