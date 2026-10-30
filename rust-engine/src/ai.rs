use crate::components::*;
use crate::GameEngine;

pub fn update_ai(engine: &mut GameEngine) {
    let enemy_workers: Vec<u32> = engine
        .units
        .iter()
        .filter(|u| u.team == Team::Enemy && u.unit_type == UnitType::Engineer)
        .map(|u| u.id)
        .collect();

    let enemy_combat_units: Vec<u32> = engine
        .units
        .iter()
        .filter(|u| u.team == Team::Enemy && u.unit_type != UnitType::Engineer)
        .map(|u| u.id)
        .collect();

    // 1. Quản lý Thợ mỏ AI tự động khai thác khoáng sản gần nhất
    let enemy_minerals: Vec<u32> = engine
        .minerals
        .iter()
        .filter(|m| m.x > 1800.0 && m.y < 800.0)
        .map(|m| m.id)
        .collect();

    if let Some(&target_mineral) = enemy_minerals.first() {
        for wid in &enemy_workers {
            if let Some(w) = engine.units.iter_mut().find(|u| u.id == *wid) {
                if w.order == UnitOrder::Idle {
                    w.order = UnitOrder::Harvest {
                        target_mineral_id: target_mineral,
                        phase: HarvestPhase::MovingToNode,
                    };
                }
            }
        }
    }

    // 2. AI Xây dựng công trình khi đủ tiền
    let enemy_supply = engine.enemy_eco.supply;
    let enemy_max_supply = engine.enemy_eco.max_supply;
    let enemy_minerals_val = engine.enemy_eco.minerals;

    // Cần thêm Supply Depot khi gần chạm trần
    let (depot_min, _) = engine.config.get_building_cost(BuildingType::SupplyDepot);
    if enemy_supply + 2 >= enemy_max_supply && enemy_minerals_val >= depot_min {
        let count_depots = engine
            .buildings
            .iter()
            .filter(|b| b.team == Team::Enemy && b.building_type == BuildingType::SupplyDepot)
            .count();
        if count_depots < 4 {
            try_ai_build(
                engine,
                BuildingType::SupplyDepot,
                2050.0 - (count_depots as f32 * 70.0),
                280.0,
            );
        }
    }

    // Xây Barracks
    let (barracks_min, _) = engine.config.get_building_cost(BuildingType::Barracks);
    let barracks_count = engine
        .buildings
        .iter()
        .filter(|b| b.team == Team::Enemy && b.building_type == BuildingType::Barracks)
        .count();
    if barracks_count < 2 && engine.enemy_eco.minerals >= barracks_min {
        try_ai_build(
            engine,
            BuildingType::Barracks,
            1950.0,
            480.0 + (barracks_count as f32 * 80.0),
        );
    }

    // Xây Power Reactor
    let (reactor_min, _) = engine.config.get_building_cost(BuildingType::PowerReactor);
    let reactor_count = engine
        .buildings
        .iter()
        .filter(|b| b.team == Team::Enemy && b.building_type == BuildingType::PowerReactor)
        .count();
    if reactor_count < 2 && engine.enemy_eco.minerals >= reactor_min {
        try_ai_build(
            engine,
            BuildingType::PowerReactor,
            2220.0,
            520.0 + (reactor_count as f32 * 70.0),
        );
    }

    // 3. AI Huấn luyện lính
    // Nếu thiếu Worker (< 6) xin thêm tại Command Center
    let (worker_min, _) = engine.config.get_unit_cost(UnitType::Engineer);
    if enemy_workers.len() < 6
        && engine.enemy_eco.minerals >= worker_min
        && enemy_supply < enemy_max_supply
    {
        if let Some(cc) = engine.buildings.iter_mut().find(|b| {
            b.team == Team::Enemy
                && b.building_type == BuildingType::CommandCenter
                && b.build_progress >= 1.0
        }) {
            if cc.training_queue.len() < 2 {
                engine.enemy_eco.minerals -= worker_min;
                cc.training_queue.push(UnitType::Engineer);
            }
        }
    }

    // Xin lính chiến đấu tại Barracks
    for b in engine.buildings.iter_mut() {
        if b.team == Team::Enemy
            && b.building_type == BuildingType::Barracks
            && b.build_progress >= 1.0
            && b.training_queue.len() < 2
            && engine.enemy_eco.supply < engine.enemy_eco.max_supply
        {
            // Huấn luyện đơn vị chiến đấu đa dạng
            if engine.enemy_eco.minerals >= 200.0 && engine.enemy_eco.energy >= 100.0 && engine.game_time > 45.0 {
                let choice = (engine.units.len() + b.id as usize) % 5;
                let evo_unit = match choice {
                    0 => UnitType::DarkRocketeer,
                    1 => UnitType::TaoistSwordmaster,
                    2 => UnitType::VoidWarlock,
                    3 => UnitType::BehemothTitan,
                    _ => UnitType::AcidBaneling,
                };
                let (mc, ec) = engine.config.get_unit_cost(evo_unit);
                if engine.enemy_eco.minerals >= mc && engine.enemy_eco.energy >= ec {
                    engine.enemy_eco.minerals -= mc;
                    engine.enemy_eco.energy -= ec;
                    b.training_queue.push(evo_unit);
                }
            } else {
                let (rifle_m, rifle_e) = engine.config.get_unit_cost(UnitType::Rifleman);
                let (striker_m, striker_e) = engine.config.get_unit_cost(UnitType::ShockTrooper);

                if engine.enemy_eco.minerals >= rifle_m && engine.enemy_eco.energy >= rifle_e {
                    engine.enemy_eco.minerals -= rifle_m;
                    engine.enemy_eco.energy -= rifle_e;
                    b.training_queue.push(UnitType::Rifleman);
                } else if engine.enemy_eco.minerals >= striker_m && engine.enemy_eco.energy >= striker_e {
                    engine.enemy_eco.minerals -= striker_m;
                    engine.enemy_eco.energy -= striker_e;
                    b.training_queue.push(UnitType::ShockTrooper);
                }
            }
        }
    }

    // 4. AI Điều quân Tấn công (Attack Wave) khi gom đủ >= 6 lính chiến đấu
    if enemy_combat_units.len() >= 6 {
        // Tìm vị trí căn cứ hoặc lính Player để càn quét
        let target_pos = engine
            .buildings
            .iter()
            .find(|b| b.team == Team::Player && b.building_type == BuildingType::CommandCenter)
            .map(|b| (b.x, b.y))
            .unwrap_or((300.0, 2000.0));

        for (i, uid) in enemy_combat_units.iter().enumerate() {
            if let Some(u) = engine.units.iter_mut().find(|u| u.id == *uid) {
                if u.order == UnitOrder::Idle {
                    let offset_x = ((i % 4) as f32 - 1.5) * 25.0;
                    let offset_y = ((i / 4) as f32) * 25.0;
                    u.order = UnitOrder::AttackMove {
                        x: target_pos.0 + offset_x,
                        y: target_pos.1 + offset_y,
                    };
                }
            }
        }
    }
}

fn try_ai_build(
    engine: &mut GameEngine,
    b_type: BuildingType,
    wx: f32,
    wy: f32,
) {
    let (min_cost, en_cost) = engine.config.get_building_cost(b_type);
    if engine.enemy_eco.minerals >= min_cost && engine.enemy_eco.energy >= en_cost {
        engine.enemy_eco.minerals -= min_cost;
        engine.enemy_eco.energy -= en_cost;
        let bid = engine.alloc_id();
        let mut building = engine.config.create_building(bid, Team::Enemy, b_type, wx, wy, false);
        building.hp = 1.0;
        building.build_progress = 0.1;
        building.rally_x = wx - 50.0;
        building.rally_y = wy;
        engine.buildings.push(building);
    }
}
