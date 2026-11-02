use crate::bridge::EngineToReactEvent;
use crate::components::*;
use crate::map::{MAP_HEIGHT, MAP_WIDTH};
use crate::simulation::commands::set_player_formation_and_reform;
use crate::simulation::economy::send_sync_event;
use crate::GameEngine;

// Bấm nút UI trên HUD Bevy (Góc trên, Minimap, Bảng nút lệnh góc dưới phải)
pub fn check_bevy_hud_click(engine: &mut GameEngine, sx: f32, sy: f32) -> bool {
    let vw = engine.camera.viewport_w;
    let vh = engine.camera.viewport_h;

    // -1. Nút đo FPS ở thanh trên cùng (click để bật/tắt bảng chi tiết hiệu năng)
    if (vw - 380.0..=vw - 302.0).contains(&sx) && (10.0..=42.0).contains(&sy) {
        engine.show_debug_overlay = !engine.show_debug_overlay;
        return true;
    }

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

pub fn handle_command_card_click(engine: &mut GameEngine, rel_x: f32, rel_y: f32) {
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
    if !engine.selected_unit_ids.is_empty() && btn_idx < 6 {
        let target_form = FormationType::from_index(btn_idx);
        set_player_formation_and_reform(engine, target_form);
        engine.outgoing_events.push(EngineToReactEvent::PlaySound {
            sound: "click".to_string(),
        });
        send_sync_event(engine);
        return;
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

pub fn handle_command_card_right_click(engine: &mut GameEngine, rel_x: f32, rel_y: f32) {
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

pub fn try_queue_unit(engine: &mut GameEngine, bid: u32, u_type: UnitType) {
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

pub fn try_cancel_unit(engine: &mut GameEngine, bid: u32, u_type: UnitType) {
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
