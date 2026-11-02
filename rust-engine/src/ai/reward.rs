use super::state::count_unlocked_techs;
use crate::components::*;
use crate::rl::{BotStrategicAction, RlBrain, STATE_DIM};
use crate::GameEngine;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AiState {
    pub rally_point: (f32, f32),
    pub expansion_attempted: bool,
    pub last_attack_target: Option<(f32, f32)>,

    // RL Brain & Policy State
    pub rl_brain: RlBrain,
    pub current_action: BotStrategicAction,
    #[serde(default)]
    pub current_formation: FormationType,
    pub decision_timer: f32,
    pub last_state: Vec<f32>,
    pub cumulative_reward: f32,
    pub training_enabled: bool,
}

#[derive(Clone, Debug, Default)]
pub struct RewardSnapshot {
    pub my_ccs: usize,
    pub my_turrets: usize,
    pub my_reactors: usize,
    pub my_converters: usize,
    pub my_barracks: usize,
    pub my_depots: usize,
    pub my_total_buildings: usize,
    pub my_combat: usize,
    pub my_banelings: usize,
    pub my_unlocked_techs: usize,
    pub opp_ccs: usize,
    pub opp_turrets: usize,
    pub opp_buildings: usize,
    pub opp_combat: usize,
    pub energy_rate: f32,
    pub energy: f32,
    pub opp_aoe_threat: f32,
    pub has_frontline: bool,
    pub has_artillery: bool,
    pub has_magic: bool,
}

impl RewardSnapshot {
    pub fn capture(engine: &GameEngine, team: Team) -> Self {
        let mut snap = Self::default();

        let my_eco = engine.team_eco(team);
        snap.energy_rate = my_eco.energy_per_sec;
        snap.energy = my_eco.energy;
        snap.my_unlocked_techs = count_unlocked_techs(engine.team_tech(team));

        for b in &engine.buildings {
            let is_hostile = engine.are_hostile(team, b.team);
            if b.team == team {
                snap.my_total_buildings += 1;
                match b.building_type {
                    BuildingType::CommandCenter => snap.my_ccs += 1,
                    BuildingType::TurretTower => snap.my_turrets += 1,
                    BuildingType::PowerReactor => snap.my_reactors += 1,
                    BuildingType::EnergyConverter => snap.my_converters += 1,
                    BuildingType::Barracks => snap.my_barracks += 1,
                    BuildingType::SupplyDepot => snap.my_depots += 1,
                }
            } else if is_hostile {
                snap.opp_buildings += 1;
                match b.building_type {
                    BuildingType::CommandCenter => snap.opp_ccs += 1,
                    BuildingType::TurretTower => snap.opp_turrets += 1,
                    _ => {}
                }
            }
        }

        let mut opp_aoe_count = 0;
        for u in &engine.units {
            let is_hostile = engine.are_hostile(team, u.team);
            if u.team == team {
                if u.unit_type != UnitType::Engineer {
                    snap.my_combat += 1;
                    match u.unit_type {
                        UnitType::BehemothTitan | UnitType::Vanguard | UnitType::ShockTrooper => {
                            snap.has_frontline = true;
                        }
                        UnitType::DarkRocketeer | UnitType::LaserMarksman | UnitType::Rifleman => {
                            snap.has_artillery = true;
                        }
                        UnitType::VoidWarlock | UnitType::TaoistSwordmaster => {
                            snap.has_magic = true;
                        }
                        UnitType::AcidBaneling => {
                            snap.my_banelings += 1;
                        }
                        _ => {}
                    }
                }
            } else if is_hostile && u.unit_type != UnitType::Engineer {
                snap.opp_combat += 1;
                if matches!(
                    u.unit_type,
                    UnitType::DarkRocketeer
                        | UnitType::VoidWarlock
                        | UnitType::AcidBaneling
                        | UnitType::BehemothTitan
                ) {
                    opp_aoe_count += 1;
                }
            }
        }

        snap.opp_aoe_threat = (opp_aoe_count as f32 / 8.0).clamp(0.0, 2.0);
        snap
    }
}

pub fn compute_ai_step_reward(
    prev: &RewardSnapshot,
    cur: &RewardSnapshot,
    action: BotStrategicAction,
    formation: FormationType,
    game_over: bool,
    game_time: f32,
    max_game_time: f32,
) -> f32 {
    let mut reward = 0.0_f32;

    // 1. MỞ RỘNG CĂN CỨ // COMMAND CENTER (+85 điểm / nhà chính mới)
    if cur.my_ccs > prev.my_ccs {
        reward += (cur.my_ccs - prev.my_ccs) as f32 * 85.0;
    }
    // Phạt nặng khi mất nhà chính
    if prev.my_ccs > cur.my_ccs {
        reward -= (prev.my_ccs - cur.my_ccs) as f32 * 180.0;
    }
    // Thưởng duy trì quy mô đa căn cứ
    if cur.my_ccs >= 2 {
        reward += (cur.my_ccs - 1) as f32 * 1.0;
    }

    // 2. PHÁT TRIỂN HỆ THỐNG PHÒNG THỦ TURRET TOWER (+24 điểm / tháp mới)
    if cur.my_turrets > prev.my_turrets {
        reward += (cur.my_turrets - prev.my_turrets) as f32 * 24.0;
    }
    // Thưởng duy trì lưới tháp pháo phòng ngự
    if cur.my_turrets > 0 {
        reward += (cur.my_turrets as f32 * 0.20).min(3.0);
    }
    // Thưởng phá tháp pháo đối phương
    if prev.opp_turrets > cur.opp_turrets {
        reward += (prev.opp_turrets - cur.opp_turrets) as f32 * 35.0;
    }

    // 3. XÂY DỰNG CÁC CÔNG TRÌNH HẠ TẦNG KHÁC
    if cur.my_converters > prev.my_converters {
        reward += (cur.my_converters - prev.my_converters) as f32 * 25.0;
    }
    if cur.my_barracks > prev.my_barracks {
        reward += (cur.my_barracks - prev.my_barracks) as f32 * 18.0;
    }
    if cur.my_reactors > prev.my_reactors {
        reward += (cur.my_reactors - prev.my_reactors) as f32 * 15.0;
    }
    if cur.my_depots > prev.my_depots {
        reward += (cur.my_depots - prev.my_depots) as f32 * 10.0;
    }
    // Thưởng tổng số công trình tăng thêm
    if cur.my_total_buildings > prev.my_total_buildings {
        reward += (cur.my_total_buildings - prev.my_total_buildings) as f32 * 8.0;
    }
    // Thưởng duy trì quy mô công trình tổng thể (Base Scale)
    reward += (cur.my_total_buildings as f32 * 0.15).min(4.5);
    if cur.my_total_buildings > cur.opp_buildings {
        reward += 0.4;
    }

    // 4. CHUỖI SẢN XUẤT VÀ CHUYỂN HÓA NĂNG LƯỢNG
    if cur.energy_rate > 10.0 {
        reward += (cur.energy_rate / 60.0).clamp(0.0, 1.5) * 0.4;
    }
    if cur.energy > crate::config::ENERGY_CONVERSION_THRESHOLD && cur.my_converters > 0 {
        reward += 0.65;
        if cur.my_reactors >= 4 {
            reward += 0.35;
        }
    }

    // 5. THƯỞNG HÀNH ĐỘNG CHIẾN LƯỢC MỞ RỘNG VÀ XÂY DỰNG
    match action {
        BotStrategicAction::ExpandEco => {
            if cur.my_ccs < 4 {
                reward += 0.8;
            }
        }
        BotStrategicAction::BuildInfrastructure => {
            if cur.my_reactors < 16 || cur.my_converters < 6 {
                reward += 0.6;
            }
        }
        BotStrategicAction::BuildDefenseGrid if cur.my_turrets < 16 => {
            reward += 0.6;
        }
        _ => {}
    }

    // 6. GIẢM SỬ DỤNG VÀ PHẠT LẠM DỤNG ACID BANELING
    // AcidBaneling nổ xong tự mất, spam quá nhiều gây kiệt quệ tài nguyên
    if cur.my_banelings > 2 {
        reward -= (cur.my_banelings - 2) as f32 * 1.5;
    }
    if action == BotStrategicAction::MilitarySwarmRush && cur.my_ccs < 2 {
        reward -= 1.5;
    }

    // 7. KHẮC CHẾ AOE VÀ TƯƠNG THÍCH ĐỘI HÌNH
    if prev.opp_aoe_threat > 0.15 {
        if formation == FormationType::Scatter {
            reward += 1.2;
        } else if formation == FormationType::Line || formation == FormationType::Box {
            reward -= 0.6;
        }
    }
    match action {
        BotStrategicAction::AllOutAssault | BotStrategicAction::SiegeArtilleryPush => {
            if formation == FormationType::Wedge || formation == FormationType::Line {
                reward += 0.8;
            }
        }
        BotStrategicAction::HarassEconomy | BotStrategicAction::FocusWeakestEnemy => {
            if formation == FormationType::Column || formation == FormationType::Wedge {
                reward += 0.9;
            }
        }
        BotStrategicAction::DefendAndCounter | BotStrategicAction::CoordinateAllySupport
            if (formation == FormationType::Circle || formation == FormationType::Box) =>
        {
            reward += 0.7;
        }
        _ => {}
    }

    // 8. TỔN THẤT VÀ TIÊU DIỆT LỰC LƯỢNG
    if prev.opp_buildings > cur.opp_buildings {
        reward += (prev.opp_buildings - cur.opp_buildings) as f32 * 25.0;
    }
    if prev.opp_combat > cur.opp_combat {
        reward += (prev.opp_combat - cur.opp_combat) as f32 * 3.5;
    }
    if prev.my_combat > cur.my_combat {
        reward -= (prev.my_combat - cur.my_combat) as f32 * 1.5;
    }
    if prev.opp_ccs > cur.opp_ccs {
        reward += (prev.opp_ccs - cur.opp_ccs) as f32 * 150.0;
    }

    // 9. CÔNG NGHỆ VÀ BINH CHỦNG PHỐI HỢP
    if cur.my_unlocked_techs > prev.my_unlocked_techs {
        reward += (cur.my_unlocked_techs - prev.my_unlocked_techs) as f32 * 16.0;
        if cur.my_unlocked_techs == 19 {
            reward += 60.0;
        }
    }
    if cur.has_frontline && cur.has_artillery && cur.has_magic {
        reward += 0.35;
    }

    // Phạt thời gian trôi nhẹ
    reward -= 0.01;

    // 10. KẾT QUẢ TRẬN ĐẤU
    if game_over {
        let any_hostile_alive = cur.opp_ccs > 0;
        let any_ally_alive = cur.my_ccs > 0;
        if !any_hostile_alive && any_ally_alive {
            reward += 500.0 + (max_game_time - game_time) * 0.2;
        } else if !any_ally_alive {
            reward -= 300.0;
        }
    } else if game_time >= max_game_time {
        reward -= 60.0;
    }

    reward
}

impl Default for AiState {
    fn default() -> Self {
        Self {
            rally_point: (1850.0, 580.0),
            expansion_attempted: false,
            last_attack_target: None,
            rl_brain: RlBrain::default(),
            current_action: BotStrategicAction::ExpandEco,
            current_formation: FormationType::Line,
            decision_timer: 0.0,
            last_state: vec![0.0; STATE_DIM],
            cumulative_reward: 0.0,
            training_enabled: false,
        }
    }
}
