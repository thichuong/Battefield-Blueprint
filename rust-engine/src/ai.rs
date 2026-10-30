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
    if enemy_supply + 2 >= enemy_max_supply && enemy_minerals_val >= 100.0 {
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
                100.0,
                0.0,
            );
        }
    }

    // Xây Barracks
    let barracks_count = engine
        .buildings
        .iter()
        .filter(|b| b.team == Team::Enemy && b.building_type == BuildingType::Barracks)
        .count();
    if barracks_count < 2 && engine.enemy_eco.minerals >= 150.0 {
        try_ai_build(
            engine,
            BuildingType::Barracks,
            1950.0,
            480.0 + (barracks_count as f32 * 80.0),
            150.0,
            0.0,
        );
    }

    // Xây Power Reactor
    let reactor_count = engine
        .buildings
        .iter()
        .filter(|b| b.team == Team::Enemy && b.building_type == BuildingType::PowerReactor)
        .count();
    if reactor_count < 2 && engine.enemy_eco.minerals >= 120.0 {
        try_ai_build(
            engine,
            BuildingType::PowerReactor,
            2220.0,
            520.0 + (reactor_count as f32 * 70.0),
            120.0,
            0.0,
        );
    }

    // 3. AI Huấn luyện lính
    // Nếu thiếu Worker (< 6) xin thêm tại Command Center
    if enemy_workers.len() < 6
        && engine.enemy_eco.minerals >= 50.0
        && enemy_supply < enemy_max_supply
    {
        if let Some(cc) = engine.buildings.iter_mut().find(|b| {
            b.team == Team::Enemy
                && b.building_type == BuildingType::CommandCenter
                && b.build_progress >= 1.0
        }) {
            if cc.training_queue.len() < 2 {
                engine.enemy_eco.minerals -= 50.0;
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
                let (mc, ec) = evo_unit.cost();
                if engine.enemy_eco.minerals >= mc && engine.enemy_eco.energy >= ec {
                    engine.enemy_eco.minerals -= mc;
                    engine.enemy_eco.energy -= ec;
                    b.training_queue.push(evo_unit);
                }
            } else if engine.enemy_eco.minerals >= 100.0 && engine.enemy_eco.energy >= 20.0 {
                engine.enemy_eco.minerals -= 100.0;
                engine.enemy_eco.energy -= 20.0;
                b.training_queue.push(UnitType::Rifleman);
            } else if engine.enemy_eco.minerals >= 75.0 {
                engine.enemy_eco.minerals -= 75.0;
                b.training_queue.push(UnitType::ShockTrooper);
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
    min_cost: f32,
    en_cost: f32,
) {
    if engine.enemy_eco.minerals >= min_cost && engine.enemy_eco.energy >= en_cost {
        engine.enemy_eco.minerals -= min_cost;
        engine.enemy_eco.energy -= en_cost;
        let bid = engine.alloc_id();
        let (w, h, hp) = match b_type {
            BuildingType::SupplyDepot => (60.0, 60.0, 800.0),
            BuildingType::PowerReactor => (60.0, 60.0, 700.0),
            BuildingType::Barracks => (80.0, 70.0, 1200.0),
            _ => (60.0, 60.0, 800.0),
        };
        engine.buildings.push(Building {
            id: bid,
            team: Team::Enemy,
            building_type: b_type,
            x: wx,
            y: wy,
            width: w,
            height: h,
            hp: 1.0,
            max_hp: hp,
            damage_type: DamageType::Physical,
            resistances: Resistances {
                physical: 0.15,
                energy: 0.10,
                explosive: 0.15,
                arcane: 0.0,
            },
            build_progress: 0.1,
            attack_cooldown: 0.0,
            current_cooldown: 0.0,
            rally_x: wx - 50.0,
            rally_y: wy,
            training_queue: Vec::new(),
            training_timer: 0.0,
        });
    }
}
