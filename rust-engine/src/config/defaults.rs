use super::constants::VOID_WARLOCK_DEFAULT_MAX_HP_DPS;
use super::types::{
    BuildingStatsConfig, EconomyConfig, GameConfig, RenderConfig, TechItemConfig, UnitStatsConfig,
};
use crate::components::{DamageType, Resistances};
use std::collections::HashMap;

impl Default for GameConfig {
    fn default() -> Self {
        let mut units = HashMap::new();
        let mut buildings = HashMap::new();
        let mut techs = HashMap::new();

        // 1. UNITS
        units.insert(
            "Engineer".to_string(),
            UnitStatsConfig {
                name: "ENGINEER // WORKER".to_string(),
                short_name: "WORKER".to_string(),
                description: "Huấn luyện thợ mỏ / kỹ sư xây dựng.".to_string(),
                hp: 90.0,
                speed: 120.0,
                attack_range: 30.0,
                attack_damage: 5.0,
                attack_cooldown: 1.0,
                damage_type: DamageType::Physical,
                aoe_radius: 0.0,
                resistances: Resistances {
                    physical: 0.05,
                    energy: 0.0,
                    explosive: 0.0,
                    arcane: 0.0,
                },
                cost_minerals: 50.0,
                cost_energy: 0.0,
                training_time: 5.0,
                percent_max_hp_dps: 0.0,
            },
        );

        units.insert(
            "ShockTrooper".to_string(),
            UnitStatsConfig {
                name: "STRIKER // SHOCK TROOPER".to_string(),
                short_name: "STRIKER".to_string(),
                description: "Kiếm sĩ Plasma, chạy nhanh, chém mạnh.".to_string(),
                hp: 160.0,
                speed: 140.0,
                attack_range: 40.0,
                attack_damage: 22.0,
                attack_cooldown: 0.8,
                damage_type: DamageType::Energy,
                aoe_radius: 0.0,
                resistances: Resistances {
                    physical: 0.15,
                    energy: 0.20,
                    explosive: 0.05,
                    arcane: 0.10,
                },
                cost_minerals: 75.0,
                cost_energy: 0.0,
                training_time: 7.0,
                percent_max_hp_dps: 0.0,
            },
        );

        units.insert(
            "Rifleman".to_string(),
            UnitStatsConfig {
                name: "MARKSMAN // RIFLEMAN".to_string(),
                short_name: "RIFLE".to_string(),
                description: "Xạ thủ súng trường tầm xa.".to_string(),
                hp: 120.0,
                speed: 110.0,
                attack_range: 160.0,
                attack_damage: 14.0,
                attack_cooldown: 1.1,
                damage_type: DamageType::Physical,
                aoe_radius: 0.0,
                resistances: Resistances {
                    physical: 0.10,
                    energy: 0.05,
                    explosive: 0.05,
                    arcane: 0.05,
                },
                cost_minerals: 100.0,
                cost_energy: 20.0,
                training_time: 8.0,
                percent_max_hp_dps: 0.0,
            },
        );

        units.insert(
            "Vanguard".to_string(),
            UnitStatsConfig {
                name: "VANGUARD // SHIELDER".to_string(),
                short_name: "SHIELD".to_string(),
                description: "Vệ binh khiên chắn năng lượng, mỗi 5s nhận khiên 100.".to_string(),
                hp: 320.0,
                speed: 90.0,
                attack_range: 45.0,
                attack_damage: 18.0,
                attack_cooldown: 1.2,
                damage_type: DamageType::Physical,
                aoe_radius: 0.0,
                resistances: Resistances {
                    physical: 0.40,
                    energy: 0.50,
                    explosive: 0.40,
                    arcane: 0.30,
                },
                cost_minerals: 150.0,
                cost_energy: 50.0,
                training_time: 12.0,
                percent_max_hp_dps: 0.0,
            },
        );

        units.insert(
            "LaserMarksman".to_string(),
            UnitStatsConfig {
                name: "SIEGE // LASER MARKSMAN".to_string(),
                short_name: "LASER".to_string(),
                description: "Tia Laser liên tục mỗi 0.2s, sát thương tăng dần theo thời gian."
                    .to_string(),
                hp: 110.0,
                speed: 95.0,
                attack_range: 220.0,
                attack_damage: 1.0,
                attack_cooldown: 0.2,
                damage_type: DamageType::Energy,
                aoe_radius: 0.0,
                resistances: Resistances {
                    physical: 0.05,
                    energy: 0.30,
                    explosive: 0.0,
                    arcane: 0.15,
                },
                cost_minerals: 175.0,
                cost_energy: 100.0,
                training_time: 15.0,
                percent_max_hp_dps: 0.0,
            },
        );

        units.insert(
            "DarkRocketeer".to_string(),
            UnitStatsConfig {
                name: "DARK TECH // ROCKETEER".to_string(),
                short_name: "ROCKET".to_string(),
                description: "Bệ phóng Rocket nổ AoE tầm xa.".to_string(),
                hp: 140.0,
                speed: 95.0,
                attack_range: 200.0,
                attack_damage: 50.0,
                attack_cooldown: 1.5,
                damage_type: DamageType::Explosive,
                aoe_radius: 50.0,
                resistances: Resistances {
                    physical: 0.20,
                    energy: 0.10,
                    explosive: 0.35,
                    arcane: 0.05,
                },
                cost_minerals: 160.0,
                cost_energy: 80.0,
                training_time: 14.0,
                percent_max_hp_dps: 0.0,
            },
        );

        units.insert(
            "TaoistSwordmaster".to_string(),
            UnitStatsConfig {
                name: "CELESTIAL // SWORDMASTER".to_string(),
                short_name: "SWORD".to_string(),
                description:
                    "Ngự phi kiếm chân khí Arcane, tích 1 kiếm/s (tối đa 7), tốc đánh 0.1s."
                        .to_string(),
                hp: 170.0,
                speed: 130.0,
                attack_range: 140.0,
                attack_damage: 40.0,
                attack_cooldown: 0.1,
                damage_type: DamageType::Arcane,
                aoe_radius: 10.0,
                resistances: Resistances {
                    physical: 0.15,
                    energy: 0.35,
                    explosive: 0.15,
                    arcane: 0.45,
                },
                cost_minerals: 180.0,
                cost_energy: 90.0,
                training_time: 15.0,
                percent_max_hp_dps: 0.0,
            },
        );

        units.insert(
            "VoidWarlock".to_string(),
            UnitStatsConfig {
                name: "VOID // SHADOW WARLOCK".to_string(),
                short_name: "WARLOCK".to_string(),
                description:
                    "Mỗi 2s triệu hồi vòng ma thuật tồn tại 5s, đốt 10% HP tối đa mỗi giây."
                        .to_string(),
                hp: 135.0,
                speed: 100.0,
                attack_range: 175.0,
                attack_damage: 0.0,
                attack_cooldown: 2.0,
                damage_type: DamageType::Arcane,
                aoe_radius: 75.0,
                resistances: Resistances {
                    physical: 0.10,
                    energy: 0.25,
                    explosive: 0.20,
                    arcane: 0.50,
                },
                cost_minerals: 200.0,
                cost_energy: 120.0,
                training_time: 16.0,
                percent_max_hp_dps: VOID_WARLOCK_DEFAULT_MAX_HP_DPS,
            },
        );

        units.insert(
            "BehemothTitan".to_string(),
            UnitStatsConfig {
                name: "BEHEMOTH // TITAN".to_string(),
                short_name: "TITAN".to_string(),
                description: "Cự thú HP khủng, dậm chấn động diện rộng.".to_string(),
                hp: 650.0,
                speed: 60.0,
                attack_range: 55.0,
                attack_damage: 40.0,
                attack_cooldown: 1.5,
                damage_type: DamageType::Physical,
                aoe_radius: 35.0,
                resistances: Resistances {
                    physical: 0.50,
                    energy: 0.20,
                    explosive: 0.20,
                    arcane: 0.10,
                },
                cost_minerals: 250.0,
                cost_energy: 100.0,
                training_time: 20.0,
                percent_max_hp_dps: 0.0,
            },
        );

        units.insert(
            "AcidBaneling".to_string(),
            UnitStatsConfig {
                name: "SWARM // ACID BANELING".to_string(),
                short_name: "BANELING".to_string(),
                description: "Bọ cảm tử tự nổ axit AoE cực lớn, x2 sát thương lên công trình."
                    .to_string(),
                hp: 100.0,
                speed: 160.0,
                attack_range: 25.0,
                attack_damage: 90.0,
                attack_cooldown: 0.5,
                damage_type: DamageType::Explosive,
                aoe_radius: 70.0,
                resistances: Resistances {
                    physical: 0.05,
                    energy: 0.05,
                    explosive: 0.20,
                    arcane: 0.20,
                },
                cost_minerals: 60.0,
                cost_energy: 30.0,
                training_time: 6.0,
                percent_max_hp_dps: 0.0,
            },
        );

        // 2. BUILDINGS
        buildings.insert(
            "CommandCenter".to_string(),
            BuildingStatsConfig {
                name: "HQ COMMAND CENTER".to_string(),
                short_name: "HQ CC".to_string(),
                description: "Trung tâm chỉ huy mở rộng.".to_string(),
                hp: 2000.0,
                cost_minerals: 400.0,
                cost_energy: 0.0,
                width: 90.0,
                height: 90.0,
                build_time: 15.0,
                resistances: Resistances {
                    physical: 0.80,
                    energy: 0.60,
                    explosive: 0.25,
                    arcane: 0.20,
                },
                supply_bonus: 10,
                energy_per_sec: 0.0,
                turret_range: 250.0,
                turret_damage: 100.0,
                turret_cooldown: 1.0,
                turret_damage_type: DamageType::Physical,
                turret_projectile_speed: 900.0,
            },
        );

        buildings.insert(
            "SupplyDepot".to_string(),
            BuildingStatsConfig {
                name: "SUPPLY DEPOT".to_string(),
                short_name: "SUPPLY".to_string(),
                description: "Tăng Giới hạn dân số (Supply).".to_string(),
                hp: 800.0,
                cost_minerals: 100.0,
                cost_energy: 0.0,
                width: 60.0,
                height: 60.0,
                build_time: 8.0,
                resistances: Resistances {
                    physical: 0.20,
                    energy: 0.15,
                    explosive: 0.10,
                    arcane: 0.05,
                },
                supply_bonus: 8,
                energy_per_sec: 0.0,
                turret_range: 0.0,
                turret_damage: 0.0,
                turret_cooldown: 0.0,
                turret_damage_type: DamageType::Physical,
                turret_projectile_speed: 900.0,
            },
        );

        buildings.insert(
            "PowerReactor".to_string(),
            BuildingStatsConfig {
                name: "POWER REACTOR".to_string(),
                short_name: "REACTOR".to_string(),
                description: "Tăng tốc độ sản sinh Năng lượng.".to_string(),
                hp: 700.0,
                cost_minerals: 120.0,
                cost_energy: 0.0,
                width: 60.0,
                height: 60.0,
                build_time: 10.0,
                resistances: Resistances {
                    physical: 0.15,
                    energy: 0.25,
                    explosive: 0.05,
                    arcane: 0.15,
                },
                supply_bonus: 0,
                energy_per_sec: 8.0,
                turret_range: 0.0,
                turret_damage: 0.0,
                turret_cooldown: 0.0,
                turret_damage_type: DamageType::Physical,
                turret_projectile_speed: 900.0,
            },
        );

        buildings.insert(
            "Barracks".to_string(),
            BuildingStatsConfig {
                name: "BARRACKS".to_string(),
                short_name: "BARRACKS".to_string(),
                description: "Trại huấn luyện lính chiến đấu.".to_string(),
                hp: 1200.0,
                cost_minerals: 150.0,
                cost_energy: 0.0,
                width: 80.0,
                height: 70.0,
                build_time: 12.0,
                resistances: Resistances {
                    physical: 0.35,
                    energy: 0.20,
                    explosive: 0.25,
                    arcane: 0.15,
                },
                supply_bonus: 0,
                energy_per_sec: 0.0,
                turret_range: 0.0,
                turret_damage: 0.0,
                turret_cooldown: 0.0,
                turret_damage_type: DamageType::Physical,
                turret_projectile_speed: 900.0,
            },
        );

        buildings.insert(
            "TurretTower".to_string(),
            BuildingStatsConfig {
                name: "TURRET TOWER".to_string(),
                short_name: "TURRET".to_string(),
                description: "Tháp phòng thủ tự động bắn tỉa.".to_string(),
                hp: 900.0,
                cost_minerals: 125.0,
                cost_energy: 50.0,
                width: 50.0,
                height: 50.0,
                build_time: 10.0,
                resistances: Resistances {
                    physical: 0.85,
                    energy: 0.60,
                    explosive: 0.35,
                    arcane: 0.30,
                },
                supply_bonus: 0,
                energy_per_sec: 0.0,
                turret_range: 250.0,
                turret_damage: 25.0,
                turret_cooldown: 1.0,
                turret_damage_type: DamageType::Energy,
                turret_projectile_speed: 900.0,
            },
        );

        buildings.insert(
            "EnergyConverter".to_string(),
            BuildingStatsConfig {
                name: "ENERGY CONVERTER".to_string(),
                short_name: "CONVERT".to_string(),
                description: "Tự động chuyển Năng lượng dư thừa thành Vàng.".to_string(),
                hp: 850.0,
                cost_minerals: 150.0,
                cost_energy: 50.0,
                width: 65.0,
                height: 65.0,
                build_time: 12.0,
                resistances: Resistances {
                    physical: 0.25,
                    energy: 0.35,
                    explosive: 0.10,
                    arcane: 0.25,
                },
                supply_bonus: 0,
                energy_per_sec: 0.0,
                turret_range: 0.0,
                turret_damage: 0.0,
                turret_cooldown: 0.0,
                turret_damage_type: DamageType::Physical,
                turret_projectile_speed: 900.0,
            },
        );

        // 3. TECHS
        techs.insert(
            "focused_laser".to_string(),
            TechItemConfig {
                name: "Focused Laser".to_string(),
                branch: "science".to_string(),
                cost_minerals: 250.0,
                cost_energy: 200.0,
                research_time: 20.0,
            },
        );
        techs.insert(
            "energy_shielding".to_string(),
            TechItemConfig {
                name: "Energy Shielding".to_string(),
                branch: "science".to_string(),
                cost_minerals: 180.0,
                cost_energy: 150.0,
                research_time: 15.0,
            },
        );
        techs.insert(
            "dark_matter_rockets".to_string(),
            TechItemConfig {
                name: "Dark Matter Rockets".to_string(),
                branch: "science".to_string(),
                cost_minerals: 220.0,
                cost_energy: 120.0,
                research_time: 18.0,
            },
        );
        techs.insert(
            "titan_alloy_plating".to_string(),
            TechItemConfig {
                name: "Titan Alloy Plating".to_string(),
                branch: "science".to_string(),
                cost_minerals: 300.0,
                cost_energy: 180.0,
                research_time: 25.0,
            },
        );
        techs.insert(
            "celestial_flying_swords".to_string(),
            TechItemConfig {
                name: "Celestial Flying Swords".to_string(),
                branch: "magic".to_string(),
                cost_minerals: 260.0,
                cost_energy: 160.0,
                research_time: 22.0,
            },
        );
        techs.insert(
            "golden_bell_shield".to_string(),
            TechItemConfig {
                name: "Golden Bell Shield".to_string(),
                branch: "magic".to_string(),
                cost_minerals: 200.0,
                cost_energy: 140.0,
                research_time: 16.0,
            },
        );
        techs.insert(
            "corrosive_curse".to_string(),
            TechItemConfig {
                name: "Corrosive Curse".to_string(),
                branch: "magic".to_string(),
                cost_minerals: 180.0,
                cost_energy: 160.0,
                research_time: 16.0,
            },
        );
        techs.insert(
            "void_cataclysm".to_string(),
            TechItemConfig {
                name: "Void Cataclysm".to_string(),
                branch: "magic".to_string(),
                cost_minerals: 320.0,
                cost_energy: 240.0,
                research_time: 28.0,
            },
        );
        techs.insert(
            "primeval_scales".to_string(),
            TechItemConfig {
                name: "Primeval Scales".to_string(),
                branch: "monster".to_string(),
                cost_minerals: 220.0,
                cost_energy: 120.0,
                research_time: 18.0,
            },
        );
        techs.insert(
            "behemoth_mutation".to_string(),
            TechItemConfig {
                name: "Behemoth Mutation".to_string(),
                branch: "monster".to_string(),
                cost_minerals: 320.0,
                cost_energy: 180.0,
                research_time: 26.0,
            },
        );
        techs.insert(
            "acid_banelings".to_string(),
            TechItemConfig {
                name: "Acid Banelings".to_string(),
                branch: "monster".to_string(),
                cost_minerals: 160.0,
                cost_energy: 90.0,
                research_time: 14.0,
            },
        );
        techs.insert(
            "hive_proliferation".to_string(),
            TechItemConfig {
                name: "Hive Proliferation".to_string(),
                branch: "monster".to_string(),
                cost_minerals: 260.0,
                cost_energy: 160.0,
                research_time: 20.0,
            },
        );
        techs.insert(
            "optical_drills".to_string(),
            TechItemConfig {
                name: "Optical Drills".to_string(),
                branch: "logistics".to_string(),
                cost_minerals: 100.0,
                cost_energy: 50.0,
                research_time: 10.0,
            },
        );
        techs.insert(
            "overcharged_reactors".to_string(),
            TechItemConfig {
                name: "Overcharged Reactors".to_string(),
                branch: "logistics".to_string(),
                cost_minerals: 150.0,
                cost_energy: 100.0,
                research_time: 15.0,
            },
        );
        techs.insert(
            "fortified_blueprints".to_string(),
            TechItemConfig {
                name: "Fortified Blueprints".to_string(),
                branch: "logistics".to_string(),
                cost_minerals: 200.0,
                cost_energy: 150.0,
                research_time: 20.0,
            },
        );
        techs.insert(
            "magnetic_rounds".to_string(),
            TechItemConfig {
                name: "Magnetic Rounds".to_string(),
                branch: "logistics".to_string(),
                cost_minerals: 200.0,
                cost_energy: 150.0,
                research_time: 15.0,
            },
        );
        techs.insert(
            "exo_boosters".to_string(),
            TechItemConfig {
                name: "Exo Boosters".to_string(),
                branch: "logistics".to_string(),
                cost_minerals: 250.0,
                cost_energy: 200.0,
                research_time: 20.0,
            },
        );
        techs.insert(
            "superheated_blades".to_string(),
            TechItemConfig {
                name: "Superheated Blades".to_string(),
                branch: "logistics".to_string(),
                cost_minerals: 350.0,
                cost_energy: 300.0,
                research_time: 30.0,
            },
        );
        techs.insert(
            "overload_strike".to_string(),
            TechItemConfig {
                name: "Overload Strike".to_string(),
                branch: "logistics".to_string(),
                cost_minerals: 400.0,
                cost_energy: 350.0,
                research_time: 35.0,
            },
        );

        Self {
            units,
            buildings,
            economy: EconomyConfig::default(),
            techs,
            render: RenderConfig::default(),
        }
    }
}
