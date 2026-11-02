pub mod constants;
mod defaults;
#[cfg(test)]
mod tests;
pub mod types;

pub use constants::*;
pub use types::*;

use crate::components::{Building, DamageType, Resistances, Team, Unit, UnitOrder};

impl GameConfig {
    pub fn unit_key(u_type: crate::components::UnitType) -> &'static str {
        match u_type {
            crate::components::UnitType::Engineer => "Engineer",
            crate::components::UnitType::ShockTrooper => "ShockTrooper",
            crate::components::UnitType::Rifleman => "Rifleman",
            crate::components::UnitType::Vanguard => "Vanguard",
            crate::components::UnitType::LaserMarksman => "LaserMarksman",
            crate::components::UnitType::DarkRocketeer => "DarkRocketeer",
            crate::components::UnitType::TaoistSwordmaster => "TaoistSwordmaster",
            crate::components::UnitType::VoidWarlock => "VoidWarlock",
            crate::components::UnitType::BehemothTitan => "BehemothTitan",
            crate::components::UnitType::AcidBaneling => "AcidBaneling",
        }
    }

    pub fn building_key(b_type: crate::components::BuildingType) -> &'static str {
        match b_type {
            crate::components::BuildingType::CommandCenter => "CommandCenter",
            crate::components::BuildingType::SupplyDepot => "SupplyDepot",
            crate::components::BuildingType::PowerReactor => "PowerReactor",
            crate::components::BuildingType::Barracks => "Barracks",
            crate::components::BuildingType::TurretTower => "TurretTower",
            crate::components::BuildingType::EnergyConverter => "EnergyConverter",
        }
    }

    #[inline]
    pub fn get_unit(&self, u_type: crate::components::UnitType) -> Option<&UnitStatsConfig> {
        self.units.get(Self::unit_key(u_type))
    }

    #[inline]
    pub fn get_building(
        &self,
        b_type: crate::components::BuildingType,
    ) -> Option<&BuildingStatsConfig> {
        self.buildings.get(Self::building_key(b_type))
    }

    #[inline]
    pub fn get_tech(&self, tech_id: &str) -> Option<&TechItemConfig> {
        self.techs.get(tech_id)
    }

    pub fn get_building_dimensions(&self, b_type: crate::components::BuildingType) -> (f32, f32) {
        if let Some(b) = self.get_building(b_type) {
            (b.width, b.height)
        } else {
            (60.0, 60.0)
        }
    }

    pub fn get_building_name(&self, b_type: crate::components::BuildingType) -> &str {
        if let Some(b) = self.get_building(b_type) {
            &b.name
        } else {
            "STRUCTURE"
        }
    }

    pub fn get_building_short_name(&self, b_type: crate::components::BuildingType) -> &str {
        if let Some(b) = self.get_building(b_type) {
            if !b.short_name.is_empty() {
                return &b.short_name;
            }
            &b.name
        } else {
            "BUILDING"
        }
    }

    pub fn get_building_description(&self, b_type: crate::components::BuildingType) -> &str {
        if let Some(b) = self.get_building(b_type) {
            &b.description
        } else {
            ""
        }
    }

    pub fn get_unit_name(&self, u_type: crate::components::UnitType) -> &str {
        if let Some(u) = self.get_unit(u_type) {
            &u.name
        } else {
            u_type.display_name()
        }
    }

    pub fn get_unit_short_name(&self, u_type: crate::components::UnitType) -> &str {
        if let Some(u) = self.get_unit(u_type) {
            if !u.short_name.is_empty() {
                return &u.short_name;
            }
            &u.name
        } else {
            u_type.button_label()
        }
    }

    pub fn get_unit_description(&self, u_type: crate::components::UnitType) -> &str {
        if let Some(u) = self.get_unit(u_type) {
            &u.description
        } else {
            ""
        }
    }

    pub fn get_unit_cost(&self, u_type: crate::components::UnitType) -> (f32, f32) {
        if let Some(u) = self.get_unit(u_type) {
            (u.cost_minerals, u.cost_energy)
        } else {
            u_type.cost()
        }
    }

    pub fn get_unit_training_time(&self, u_type: crate::components::UnitType) -> f32 {
        if let Some(u) = self.get_unit(u_type) {
            u.training_time
        } else {
            u_type.training_time()
        }
    }

    pub fn get_unit_cost_label(&self, u_type: crate::components::UnitType) -> String {
        let (min, en) = self.get_unit_cost(u_type);
        if en > 0.0 {
            format!("{:.0}M {:.0}E", min, en)
        } else {
            format!("{:.0}M", min)
        }
    }

    pub fn get_building_cost(&self, b_type: crate::components::BuildingType) -> (f32, f32) {
        if let Some(b) = self.get_building(b_type) {
            (b.cost_minerals, b.cost_energy)
        } else {
            (100.0, 0.0)
        }
    }

    pub fn get_building_cost_label(&self, b_type: crate::components::BuildingType) -> String {
        let (min, en) = self.get_building_cost(b_type);
        if en > 0.0 {
            format!("{:.0}M {:.0}E", min, en)
        } else {
            format!("{:.0}M", min)
        }
    }

    pub fn create_unit(
        &self,
        id: u32,
        team: Team,
        u_type: crate::components::UnitType,
        x: f32,
        y: f32,
    ) -> Unit {
        let default_cfg = UnitStatsConfig {
            name: "Unknown".to_string(),
            short_name: "UNK".to_string(),
            description: String::new(),
            hp: 100.0,
            speed: 100.0,
            attack_range: 40.0,
            attack_damage: 10.0,
            attack_cooldown: 1.0,
            damage_type: DamageType::Physical,
            aoe_radius: 0.0,
            resistances: Resistances::default(),
            cost_minerals: 50.0,
            cost_energy: 0.0,
            training_time: 5.0,
            percent_max_hp_dps: 0.0,
        };
        let cfg = self.get_unit(u_type).unwrap_or(&default_cfg);

        let (shield, max_shield) = if u_type == crate::components::UnitType::Vanguard {
            (100.0, 100.0)
        } else {
            (0.0, 0.0)
        };
        let flying_swords = if u_type == crate::components::UnitType::TaoistSwordmaster {
            7
        } else {
            0
        };

        Unit {
            id,
            team,
            unit_type: u_type,
            x,
            y,
            target_x: x,
            target_y: y,
            speed: cfg.speed,
            hp: cfg.hp,
            max_hp: cfg.hp,
            attack_range: cfg.attack_range,
            attack_damage: cfg.attack_damage,
            damage_type: cfg.damage_type,
            aoe_radius: cfg.aoe_radius,
            resistances: cfg.resistances,
            attack_cooldown: cfg.attack_cooldown,
            current_cooldown: 0.0,
            walk_cycle: 0.0,
            facing_right: team == Team::Player,
            order: UnitOrder::Idle,
            carried_minerals: 0.0,
            formation_offset: (0.0, 0.0),
            shield,
            max_shield,
            shield_timer: 0.0,
            laser_target_id: None,
            laser_stacks: 0,
            flying_swords,
            sword_regen_timer: 0.0,
            attack_anim_timer: 0.0,
            last_attack_pos: None,
            percent_max_hp_dps: cfg.percent_max_hp_dps,
        }
    }

    pub fn create_building(
        &self,
        id: u32,
        team: Team,
        b_type: crate::components::BuildingType,
        x: f32,
        y: f32,
        is_completed: bool,
    ) -> Building {
        let default_cfg = BuildingStatsConfig {
            name: "Unknown".to_string(),
            short_name: "UNK".to_string(),
            description: String::new(),
            hp: 1000.0,
            cost_minerals: 100.0,
            cost_energy: 0.0,
            width: 60.0,
            height: 60.0,
            build_time: 10.0,
            resistances: Resistances::default(),
            supply_bonus: 0,
            energy_per_sec: 0.0,
            turret_range: 0.0,
            turret_damage: 0.0,
            turret_cooldown: 0.0,
            turret_damage_type: DamageType::Physical,
            turret_projectile_speed: 0.0,
        };
        let cfg = self.get_building(b_type).unwrap_or(&default_cfg);

        Building {
            id,
            team,
            building_type: b_type,
            x,
            y,
            width: cfg.width,
            height: cfg.height,
            hp: cfg.hp,
            max_hp: cfg.hp,
            damage_type: cfg.turret_damage_type,
            resistances: cfg.resistances,
            build_progress: if is_completed { 1.0 } else { 0.0 },
            under_attack_timer: 0.0,
            attack_cooldown: 0.0,
            current_cooldown: 0.0,
            rally_x: x,
            rally_y: y + cfg.height * 0.5 + 20.0,
            training_queue: Vec::new(),
            training_timer: 0.0,
        }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|_| "{}".to_string())
    }

    pub fn from_json(json_str: &str) -> Result<Self, String> {
        serde_json::from_str::<GameConfig>(json_str).map_err(|e| e.to_string())
    }

    pub fn update_from_json(&mut self, json_str: &str) -> Result<(), String> {
        let new_cfg = Self::from_json(json_str)?;
        *self = new_cfg;
        Ok(())
    }
}
