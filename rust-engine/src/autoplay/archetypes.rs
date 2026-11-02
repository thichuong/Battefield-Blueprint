use super::executor::ALL_TECH_IDS;
use super::types::{AutoAction, AutoCard, AutoCondition, AutoPlayConfig};
use crate::components::{BuildingType, FormationType, Team, UnitType};
use crate::GameEngine;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutoTrainingArchetype {
    ScienceArtillery = 0,
    MagicCelestialVoid = 1,
    MonsterSwarmBehemoth = 2,
    BioBlitzRush = 3,
    MacroFortress = 4,
    CombinedTitanArtilleryMage = 5,
    OmniTechGrandmaster = 6,
}

impl AutoTrainingArchetype {
    pub fn name(&self) -> &'static str {
        match self {
            Self::ScienceArtillery => "Science Laser & Rocket Siege",
            Self::MagicCelestialVoid => "Magic Sword & Void Cataclysm",
            Self::MonsterSwarmBehemoth => "Monster Swarm & Behemoth Titan",
            Self::BioBlitzRush => "Bio Infantry Blitz Rush",
            Self::MacroFortress => "Macro Eco & Multi-row Fortress",
            Self::CombinedTitanArtilleryMage => "Combined Titan, Artillery & Void Mage",
            Self::OmniTechGrandmaster => "Omni-Tech Grandmaster (All 19 Techs)",
        }
    }
}

pub fn create_training_autoplay_config(
    team: Team,
    archetype: AutoTrainingArchetype,
) -> AutoPlayConfig {
    let mut config = AutoPlayConfig::default_for_team(team);
    config.enabled = true;

    match archetype {
        AutoTrainingArchetype::ScienceArtillery => {
            config.priority_techs = vec![
                "focused_laser".to_string(),
                "energy_shielding".to_string(),
                "dark_matter_rockets".to_string(),
                "titan_alloy_plating".to_string(),
                "overcharged_reactors".to_string(),
                "overload_strike".to_string(),
                // Phối hợp thêm nhánh Quái vật & Ma thuật
                "primeval_scales".to_string(),
                "behemoth_mutation".to_string(),
                "celestial_flying_swords".to_string(),
            ];
            config.preferred_formation = FormationType::Box;
            config.turret_defense.rows = 2;
        }
        AutoTrainingArchetype::MagicCelestialVoid => {
            config.priority_techs = vec![
                "celestial_flying_swords".to_string(),
                "golden_bell_shield".to_string(),
                "corrosive_curse".to_string(),
                "void_cataclysm".to_string(),
                // Phối hợp thêm Hỏa lực tầm xa & Chống chịu
                "dark_matter_rockets".to_string(),
                "primeval_scales".to_string(),
                "behemoth_mutation".to_string(),
            ];
            config.preferred_formation = FormationType::Line;
        }
        AutoTrainingArchetype::MonsterSwarmBehemoth => {
            config.priority_techs = vec![
                "optical_drills".to_string(),
                "acid_banelings".to_string(),
                "hive_proliferation".to_string(),
                "primeval_scales".to_string(),
                "behemoth_mutation".to_string(),
                // Phối hợp thêm Phù thủy & Pháo hắc khoa kỹ
                "void_cataclysm".to_string(),
                "dark_matter_rockets".to_string(),
            ];
            config.preferred_formation = FormationType::Circle;
            if let Some(prod_loop) = config.loops.iter_mut().find(|l| l.id == "loop_production") {
                if let Some(w_card) = prod_loop
                    .cards
                    .iter_mut()
                    .find(|c| c.id == "card_prod_worker")
                {
                    w_card.conditions = vec![AutoCondition::UnitCountLessThan {
                        unit_type: "Engineer".to_string(),
                        count: 10,
                    }];
                }
            }
        }
        AutoTrainingArchetype::BioBlitzRush => {
            config.priority_techs = vec![
                "magnetic_rounds".to_string(),
                "exo_boosters".to_string(),
                "superheated_blades".to_string(),
                // Chuyển giao sang binh chủng cao cấp
                "dark_matter_rockets".to_string(),
                "behemoth_mutation".to_string(),
            ];
            config.preferred_formation = FormationType::Wedge;
            if let Some(mil_loop) = config.loops.iter_mut().find(|l| l.id == "loop_military") {
                if let Some(atk_card) = mil_loop
                    .cards
                    .iter_mut()
                    .find(|c| c.id == "card_mil_attack")
                {
                    atk_card.conditions = vec![AutoCondition::UnitCountGreaterThan {
                        unit_type: "Combat".to_string(),
                        count: 12,
                    }];
                }
            }
        }
        AutoTrainingArchetype::MacroFortress => {
            config.priority_techs = vec![
                "optical_drills".to_string(),
                "overcharged_reactors".to_string(),
                "fortified_blueprints".to_string(),
                "dark_matter_rockets".to_string(),
                "titan_alloy_plating".to_string(),
                "behemoth_mutation".to_string(),
                "void_cataclysm".to_string(),
            ];
            config.preferred_formation = FormationType::Line;
            config.turret_defense.rows = 2;
            config.turret_defense.spacing = 75.0;
        }
        AutoTrainingArchetype::CombinedTitanArtilleryMage => {
            config.priority_techs = vec![
                // 1. Kinh tế & phòng ngự cơ sở
                "optical_drills".to_string(),
                // 2. Chống chịu Khổng lồ
                "primeval_scales".to_string(),
                "behemoth_mutation".to_string(),
                // 3. Pháo tầm xa Rocket & Laser
                "dark_matter_rockets".to_string(),
                "focused_laser".to_string(),
                "energy_shielding".to_string(),
                // 4. Phù thủy Hư không & Kiếm thần
                "corrosive_curse".to_string(),
                "void_cataclysm".to_string(),
                "celestial_flying_swords".to_string(),
                "golden_bell_shield".to_string(),
            ];
            config.preferred_formation = FormationType::Box;
            config.turret_defense.rows = 2;
        }
        AutoTrainingArchetype::OmniTechGrandmaster => {
            config.priority_techs = ALL_TECH_IDS.iter().map(|s| s.to_string()).collect();
            config.preferred_formation = FormationType::Line;
            config.turret_defense.rows = 2;
        }
    }

    config
}

/// Tùy biến giữa trận (Mid-Match Adaptation) cho Auto:
/// Dựa theo công nghệ đã nghiên cứu xong, binh chủng mới mở khóa, thời gian trận đấu,
/// và tình hình quân địch để tự động cập nhật hoặc thêm thẻ bài vào vòng lặp Auto.
pub fn adapt_training_autoplay_mid_match(engine: &mut GameEngine, team: Team) {
    let tech = engine.team_tech(team).clone();

    let enemy_combat_count = engine
        .units
        .iter()
        .filter(|u| engine.are_hostile(team, u.team) && u.unit_type != UnitType::Engineer)
        .count();

    let my_combat_count = engine
        .units
        .iter()
        .filter(|u| u.team == team && u.unit_type != UnitType::Engineer)
        .count();

    let enemy_aoe_threat = engine.units.iter().any(|u| {
        engine.are_hostile(team, u.team)
            && (u.unit_type == UnitType::DarkRocketeer
                || u.unit_type == UnitType::VoidWarlock
                || u.unit_type == UnitType::AcidBaneling
                || u.unit_type == UnitType::BehemothTitan)
    });

    let game_time = engine.game_time;
    let eco_minerals = engine.team_eco(team).minerals;
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

    let config = &mut engine.team_autoplay_mut(team).config;

    // 1. Tùy biến Vòng Lặp Sản Xuất (Loop Production)
    if let Some(prod_loop) = config.loops.iter_mut().find(|l| l.id == "loop_production") {
        // Nâng cấp thợ mỏ theo thời gian và kinh tế
        if let Some(w_card) = prod_loop
            .cards
            .iter_mut()
            .find(|c| c.id == "card_prod_worker")
        {
            let target_workers = if tech.optical_drills || game_time > 150.0 {
                16
            } else if game_time > 60.0 {
                10
            } else {
                7
            };
            w_card.conditions = vec![AutoCondition::UnitCountLessThan {
                unit_type: "Engineer".to_string(),
                count: target_workers,
            }];
        }

        // Tự động thêm thẻ mở rộng CommandCenter phụ nếu có nhiều khoáng sản và đủ phát triển
        let has_expand_card = prod_loop
            .cards
            .iter()
            .any(|c| c.id == "card_prod_expand_cc");
        if !has_expand_card && (game_time > 60.0 || eco_minerals >= 350.0) {
            prod_loop.cards.insert(
                1,
                AutoCard {
                    id: "card_prod_expand_cc".to_string(),
                    name: "Mở Rộng Căn Cứ Phụ (CommandCenter)".to_string(),
                    enabled: true,
                    conditions: vec![
                        AutoCondition::BuildingCountLessThan {
                            building_type: "CommandCenter".to_string(),
                            count: 4,
                        },
                        AutoCondition::MineralsGreaterThan { amount: 380.0 },
                    ],
                    action: AutoAction::BuildStructure {
                        building_type: "CommandCenter".to_string(),
                        count: 1,
                    },
                },
            );
        }

        // Tự động tăng gấp đôi giới hạn công trình (PowerReactor, TurretTower, Barracks) khi chạm tới trần
        for card in prod_loop.cards.iter_mut() {
            for cond in card.conditions.iter_mut() {
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

        // Tùy biến sản xuất binh chủng cao cấp ngay trong Vòng Lặp Sản Xuất (Loop Production)
        let advanced_units = [
            (
                "BehemothTitan",
                tech.behemoth_mutation,
                "card_prod_titan",
                "Mua BehemothTitan (Chiến Thú Khổng Lồ)",
                180.0,
                5,
            ),
            (
                "DarkRocketeer",
                tech.dark_matter_rockets,
                "card_prod_rocket",
                "Mua DarkRocketeer (Rocket Hắc Khoa Kỹ AoE)",
                120.0,
                7,
            ),
            (
                "LaserMarksman",
                tech.focused_laser,
                "card_prod_laser",
                "Mua LaserMarksman (Bắn Tỉa Laser)",
                130.0,
                6,
            ),
            (
                "VoidWarlock",
                tech.void_cataclysm,
                "card_prod_warlock",
                "Mua VoidWarlock (Phù Thủy Hư Không AoE)",
                140.0,
                5,
            ),
            (
                "TaoistSwordmaster",
                tech.celestial_flying_swords,
                "card_prod_sword",
                "Mua TaoistSwordmaster (Tu Tiên Ngự Kiếm)",
                130.0,
                6,
            ),
            (
                "Vanguard",
                tech.energy_shielding,
                "card_prod_vanguard",
                "Mua Vanguard (Lá Chắn Tiên Phong)",
                100.0,
                5,
            ),
            (
                "AcidBaneling",
                tech.acid_banelings,
                "card_prod_baneling",
                "Mua AcidBaneling (Bọ Nổ Axit Cảm Tử)",
                250.0,
                2,
            ),
        ];

        let mut unlocked_advanced_count = 0;

        for (u_name, is_unlocked, card_id, card_name, min_threshold, max_count) in advanced_units {
            if is_unlocked {
                unlocked_advanced_count += 1;
                let exists = prod_loop
                    .cards
                    .iter()
                    .any(|c| c.id == card_id || c.id == card_id.replace("card_prod_", "card_mil_"));
                if !exists {
                    // Chèn trước thẻ Delay (thường là thẻ cuối của vòng sản xuất)
                    let insert_pos = prod_loop
                        .cards
                        .iter()
                        .position(|c| c.id == "card_prod_delay")
                        .unwrap_or(prod_loop.cards.len());
                    prod_loop.cards.insert(
                        insert_pos,
                        AutoCard {
                            id: card_id.to_string(),
                            name: card_name.to_string(),
                            enabled: true,
                            conditions: vec![
                                AutoCondition::MineralsGreaterThan {
                                    amount: min_threshold,
                                },
                                AutoCondition::UnitCountLessThan {
                                    unit_type: u_name.to_string(),
                                    count: max_count,
                                },
                            ],
                            action: AutoAction::TrainUnit {
                                unit_type: u_name.to_string(),
                                count: if tech.hive_proliferation { 2 } else { 1 },
                            },
                        },
                    );
                } else if let Some(card) = prod_loop
                    .cards
                    .iter_mut()
                    .find(|c| c.id == card_id || c.id == card_id.replace("card_prod_", "card_mil_"))
                {
                    card.conditions = vec![
                        AutoCondition::MineralsGreaterThan {
                            amount: min_threshold,
                        },
                        AutoCondition::UnitCountLessThan {
                            unit_type: u_name.to_string(),
                            count: max_count,
                        },
                    ];
                }
            }
        }

        // Chuyển giao cơ cấu quân đội (Transition):
        // Khi đã có binh chủng cao cấp và bước sang mid-game, hạn chế sản xuất lính cơ bản
        let should_throttle_basics =
            unlocked_advanced_count >= 1 && (game_time > 110.0 || my_combat_count >= 12);
        if let Some(shock_card) = prod_loop
            .cards
            .iter_mut()
            .find(|c| c.id == "card_prod_shock" || c.id == "card_mil_shock")
        {
            if should_throttle_basics {
                shock_card.conditions = vec![
                    AutoCondition::UnitCountLessThan {
                        unit_type: "ShockTrooper".to_string(),
                        count: 6,
                    },
                    AutoCondition::MineralsGreaterThan { amount: 200.0 },
                ];
            }
        }
        if let Some(rifle_card) = prod_loop
            .cards
            .iter_mut()
            .find(|c| c.id == "card_prod_rifle" || c.id == "card_mil_rifle")
        {
            if should_throttle_basics {
                rifle_card.conditions = vec![
                    AutoCondition::UnitCountLessThan {
                        unit_type: "Rifleman".to_string(),
                        count: 6,
                    },
                    AutoCondition::MineralsGreaterThan { amount: 200.0 },
                ];
            }
        }
    }

    // 2. Tùy biến Vòng Lặp Quân Sự (Loop Military): chuyên trách Tổng Tấn Công & Hành Quân
    if let Some(mil_loop) = config.loops.iter_mut().find(|l| l.id == "loop_military") {
        // Tùy biến ngưỡng Tổng Tấn Công (AttackWave):
        // Nếu bị đối phương dồn ép hoặc đã có lượng quân áp đảo -> điều chỉnh ngưỡng tấn công
        if let Some(atk_card) = mil_loop
            .cards
            .iter_mut()
            .find(|c| c.id == "card_mil_attack")
        {
            let required_units = if enemy_combat_count >= 18 {
                // Địch quá đông, phải gom đủ lực lượng phản kích hoặc xả quân giữ nhà
                16
            } else if tech.dark_matter_rockets || tech.focused_laser || tech.behemoth_mutation {
                // Đã có binh chủng công thành hạng nặng -> tấn công áp đảo sớm hơn
                15
            } else {
                20
            };
            atk_card.conditions = vec![AutoCondition::UnitCountGreaterThan {
                unit_type: "Combat".to_string(),
                count: required_units,
            }];
        }
    }

    // 3. Tùy biến Phòng Tuyến Tháp Pháo (Turret Defense) & Đội hình khắc chế AoE
    if enemy_aoe_threat && config.preferred_formation != FormationType::Scatter {
        config.preferred_formation = FormationType::Scatter;
    }

    if enemy_combat_count >= 14 && config.turret_defense.rows < 2 {
        config.turret_defense.rows = 2;
    }
}
