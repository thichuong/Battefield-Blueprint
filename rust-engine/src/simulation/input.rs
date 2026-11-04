use crate::bridge::EngineToReactEvent;
use crate::components::*;
use crate::config::{
    COMMAND_CARD_BTN_HEIGHT, COMMAND_CARD_BTN_WIDTH, COMMAND_CARD_GAP, COMMAND_CARD_HEIGHT,
    COMMAND_CARD_PADDING, COMMAND_CARD_WIDTH, HUD_MARGIN, MINIMAP_SIZE,
};
use crate::map::{MAP_HEIGHT, MAP_WIDTH};
use crate::simulation::commands::set_player_formation_and_reform;
use crate::simulation::economy::send_sync_event;
use crate::simulation::hud_input::{check_bevy_hud_click, handle_command_card_right_click};
use crate::GameEngine;

pub fn handle_mouse_down(engine: &mut GameEngine, button: i32, cx: f32, cy: f32) {
    let vh = engine.camera.viewport_h;

    let mm_min_x = HUD_MARGIN;
    let mm_max_x = HUD_MARGIN + MINIMAP_SIZE;
    let mm_min_y = vh - (HUD_MARGIN + MINIMAP_SIZE);
    let mm_max_y = vh - HUD_MARGIN;

    // Kiểm tra bấm vào Minimap để kéo thả camera
    if (mm_min_x..=mm_max_x).contains(&cx) && (mm_min_y..=mm_max_y).contains(&cy) {
        engine.is_minimap_dragging = true;
        let norm_x = ((cx - mm_min_x) / MINIMAP_SIZE).clamp(0.0, 1.0);
        let norm_y = ((cy - mm_min_y) / MINIMAP_SIZE).clamp(0.0, 1.0);
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
                        if let Some(u) = engine
                            .units
                            .iter()
                            .find(|u| u.id == *uid && u.team == Team::Player)
                        {
                            sum_x += u.x;
                            sum_y += u.y;
                            count += 1.0;
                        }
                    }
                    if count > 0.0 {
                        (sum_x / count, sum_y / count)
                    } else {
                        (wx, wy)
                    }
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
        if engine.build_mode.is_none() {
            engine.selection_box = Some((cx, cy, cx, cy));
        }

        // Kiểm tra xem có bấm vào các nút trên HUD của Bevy không
        if check_bevy_hud_click(engine, cx, cy) {
            engine.selection_box = None;
            engine.is_mouse_down = false;
        }
    } else if button == 2 {
        let vw = engine.camera.viewport_w;
        let vh = engine.camera.viewport_h;

        let cc_min_x = vw - (HUD_MARGIN + COMMAND_CARD_WIDTH);
        let cc_max_x = vw - HUD_MARGIN;
        let cc_min_y = vh - (HUD_MARGIN + COMMAND_CARD_HEIGHT);
        let cc_max_y = vh - HUD_MARGIN;

        // Bấm chuột phải trên Command Card để hủy bớt lính trong hàng đợi và hoàn tiền (nếu đang hiển thị)
        if engine.has_command_card_content()
            && cx >= cc_min_x
            && cx <= cc_max_x
            && cy >= cc_min_y
            && cy <= cc_max_y
        {
            handle_command_card_right_click(engine, cx - cc_min_x, cy - cc_min_y);
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
            engine.selection_box = None;
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

    let mm_min_x = HUD_MARGIN;
    let mm_min_y = vh - (HUD_MARGIN + MINIMAP_SIZE);

    if engine.is_minimap_dragging {
        let norm_x = ((cx - mm_min_x) / MINIMAP_SIZE).clamp(0.0, 1.0);
        let norm_y = ((cy - mm_min_y) / MINIMAP_SIZE).clamp(0.0, 1.0);
        engine.camera.x = norm_x * MAP_WIDTH;
        engine.camera.y = norm_y * MAP_HEIGHT;
        return;
    }

    if engine.is_mouse_down && engine.build_mode.is_none() {
        let (sx1, sy1) = engine.mouse_down_start;
        engine.selection_box = Some((sx1, sy1, cx, cy));
    }

    let cc_min_x = vw - (HUD_MARGIN + COMMAND_CARD_WIDTH);
    let cc_max_x = vw - HUD_MARGIN;
    let cc_min_y = vh - (HUD_MARGIN + COMMAND_CARD_HEIGHT);
    let cc_max_y = vh - HUD_MARGIN;

    // Hover trên Command Card (chỉ khi đang hiển thị)
    if engine.has_command_card_content()
        && cx >= cc_min_x
        && cx <= cc_max_x
        && cy >= cc_min_y
        && cy <= cc_max_y
    {
        let rel_x = cx - cc_min_x;
        let rel_y = cy - cc_min_y;
        let col_step = COMMAND_CARD_BTN_WIDTH + COMMAND_CARD_GAP;
        let row_step = COMMAND_CARD_BTN_HEIGHT + COMMAND_CARD_GAP;
        let col = ((rel_x - COMMAND_CARD_PADDING).max(0.0) / col_step).min(2.0) as usize;
        let row = ((rel_y - COMMAND_CARD_PADDING).max(0.0) / row_step).min(2.0) as usize;
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
                engine.sim_accumulator = 0.0;
                engine.last_frame_time = 0.0;
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
        "KeyC" => {
            // Xây Nhà Chính CommandCenter nếu đang chọn Engineer
            let has_worker = engine.selected_unit_ids.iter().any(|uid| {
                engine.units.iter().any(|u| {
                    u.id == *uid && u.unit_type == UnitType::Engineer && u.team == Team::Player
                })
            });
            if has_worker {
                engine.build_mode = Some(BuildingType::CommandCenter);
                engine.outgoing_events.push(EngineToReactEvent::PlaySound {
                    sound: "click".to_string(),
                });
            }
        }
        "KeyH" => {
            // Mở Bảng Hướng Dẫn Chơi Game (Guide)
            engine
                .outgoing_events
                .push(EngineToReactEvent::OpenGuideModal);
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
            engine.sim_accumulator = 0.0;
            engine.last_frame_time = 0.0;
        }
        _ => {}
    }
}

pub fn select_at_point(engine: &mut GameEngine, wx: f32, wy: f32) {
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

pub fn issue_right_click_order(engine: &mut GameEngine, wx: f32, wy: f32) {
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
        engine.are_hostile(Team::Player, u.team)
            && ((u.x - wx).powi(2) + (u.y - wy).powi(2)).sqrt() <= 26.0
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
                if let Some(u) = engine
                    .units
                    .iter()
                    .find(|u| u.id == *uid && u.team == Team::Player)
                {
                    sum_x += u.x;
                    sum_y += u.y;
                    count += 1.0;
                }
            }
            if count > 0.0 {
                (sum_x / count, sum_y / count)
            } else {
                (wx, wy)
            }
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

pub fn try_place_building(engine: &mut GameEngine, b_type: BuildingType, wx: f32, wy: f32) {
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
