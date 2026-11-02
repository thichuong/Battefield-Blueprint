use super::types::TurretDefenseLine;
use crate::components::{BuildingType, Team};
use crate::GameEngine;

/// Tính danh sách tất cả các điểm đặt tháp dọc theo hàng rào phòng thủ
pub fn compute_defense_tower_spots(defense_line: &TurretDefenseLine) -> Vec<(f32, f32)> {
    let mut spots = Vec::new();
    let (x1, y1) = defense_line.p1;
    let (x2, y2) = defense_line.p2;

    let dx = x2 - x1;
    let dy = y2 - y1;
    let len = (dx * dx + dy * dy).sqrt();

    if len < 10.0 {
        return vec![(x1, y1)];
    }

    let ux = dx / len;
    let uy = dy / len;

    // Vector pháp tuyến vuông góc hướng ra ngoài (về phía địch / Đông Bắc)
    // Nếu trục line chạy từ Tây Bắc sang Đông Nam, pháp tuyến (-uy, ux) sẽ hướng về phía Đông Bắc
    let nx = -uy;
    let ny = ux;

    let spacing = defense_line.spacing.max(45.0);
    let num_towers_per_row = ((len / spacing).floor() as usize) + 1;
    let row_spacing = defense_line.row_spacing.max(40.0);
    let rows = defense_line.rows.max(1) as usize;

    for r in 0..rows {
        let row_offset_x = (r as f32) * row_spacing * nx;
        let row_offset_y = (r as f32) * row_spacing * ny;

        for i in 0..num_towers_per_row {
            let t = if num_towers_per_row > 1 {
                (i as f32) * spacing
            } else {
                len * 0.5
            };
            if t <= len + 5.0 {
                let sx = x1 + t * ux + row_offset_x;
                let sy = y1 + t * uy + row_offset_y;
                spots.push((sx, sy));
            }
        }
    }

    spots
}

/// Tìm điểm còn trống trên hàng rào chưa có TurretTower và hợp lệ để đặt móng
pub fn find_defense_line_spot(
    engine: &GameEngine,
    defense_line: &TurretDefenseLine,
) -> Option<(f32, f32)> {
    find_defense_line_spot_for_team(engine, Team::Player, defense_line)
}

pub fn find_defense_line_spot_for_team(
    engine: &GameEngine,
    team: Team,
    defense_line: &TurretDefenseLine,
) -> Option<(f32, f32)> {
    let spots = compute_defense_tower_spots(defense_line);
    let (bw, bh) = engine
        .config
        .get_building(BuildingType::TurretTower)
        .map(|c| (c.width, c.height))
        .unwrap_or((60.0, 60.0));

    for (sx, sy) in spots {
        let already_has_turret = engine.buildings.iter().any(|b| {
            b.team == team
                && b.building_type == BuildingType::TurretTower
                && (b.x - sx).powi(2) + (b.y - sy).powi(2) < 45.0_f32.powi(2)
        });
        if already_has_turret {
            continue;
        }

        let collides_building = engine.buildings.iter().any(|b| {
            let clearance_x = (bw + b.width) * 0.5 + 15.0;
            let clearance_y = (bh + b.height) * 0.5 + 15.0;
            (sx - b.x).abs() < clearance_x && (sy - b.y).abs() < clearance_y
        });
        if collides_building {
            continue;
        }

        let collides_minerals = engine.minerals.iter().any(|m| {
            let dist = ((sx - m.x).powi(2) + (sy - m.y).powi(2)).sqrt();
            dist < 75.0
        });
        if collides_minerals {
            continue;
        }

        let collides_obstacles = engine.obstacles.iter().any(|obs| {
            let obs_cx = obs.x + obs.width * 0.5;
            let obs_cy = obs.y + obs.height * 0.5;
            let clearance_x = (bw + obs.width) * 0.5 + 12.0;
            let clearance_y = (bh + obs.height) * 0.5 + 12.0;
            (sx - obs_cx).abs() < clearance_x && (sy - obs_cy).abs() < clearance_y
        });
        if collides_obstacles {
            continue;
        }

        // Không thể đặt móng công trình đè lên đầu quân địch đang giao chiến
        let collides_enemies = engine
            .units
            .iter()
            .any(|u| u.team != team && ((sx - u.x).powi(2) + (sy - u.y).powi(2)).sqrt() < 80.0);
        if collides_enemies {
            continue;
        }

        if !(120.0..=(crate::map::MAP_WIDTH - 120.0)).contains(&sx)
            || !(120.0..=(crate::map::MAP_HEIGHT - 120.0)).contains(&sy)
        {
            continue;
        }

        return Some((sx, sy));
    }

    None
}

pub fn find_player_build_spot(engine: &GameEngine, b_type: BuildingType) -> Option<(f32, f32)> {
    find_team_build_spot(engine, Team::Player, b_type)
}

pub fn find_team_build_spot(
    engine: &GameEngine,
    team: Team,
    b_type: BuildingType,
) -> Option<(f32, f32)> {
    let (bw, bh) = engine
        .config
        .get_building(b_type)
        .map(|c| (c.width, c.height))
        .unwrap_or((70.0, 70.0));

    let defense_line = &engine.team_autoplay(team).config.turret_defense;

    if b_type == BuildingType::TurretTower && defense_line.enabled {
        if let Some(spot) = find_defense_line_spot_for_team(engine, team, defense_line) {
            return Some(spot);
        }

        let mut extended_line = defense_line.clone();
        if extended_line.rows < 3 {
            extended_line.rows = (extended_line.rows + 1).min(3);
            if let Some(spot) = find_defense_line_spot_for_team(engine, team, &extended_line) {
                return Some(spot);
            }
        }
    }

    if b_type == BuildingType::CommandCenter {
        let mut candidate_minerals: Vec<(f32, f32)> = Vec::new();
        for m in &engine.minerals {
            let has_cc_nearby = engine.buildings.iter().any(|b| {
                b.building_type == BuildingType::CommandCenter
                    && (b.x - m.x).powi(2) + (b.y - m.y).powi(2) < 380.0_f32.powi(2)
            });
            if !has_cc_nearby {
                candidate_minerals.push((m.x, m.y));
            }
        }

        let base_ref = engine.base_pos(team);
        candidate_minerals.sort_by(|a, b| {
            let da = (a.0 - base_ref.0).powi(2) + (a.1 - base_ref.1).powi(2);
            let db = (b.0 - base_ref.0).powi(2) + (b.1 - base_ref.1).powi(2);
            da.partial_cmp(&db).unwrap()
        });

        for (mx, my) in candidate_minerals {
            for ang_deg in [0, 45, 90, 135, 180, 225, 270, 315] {
                let rad = (ang_deg as f32).to_radians();
                let cand_x = mx + 115.0 * rad.cos();
                let cand_y = my + 115.0 * rad.sin();

                if !(120.0..=(crate::map::MAP_WIDTH - 120.0)).contains(&cand_x)
                    || !(120.0..=(crate::map::MAP_HEIGHT - 120.0)).contains(&cand_y)
                {
                    continue;
                }
                let collides_b = engine.buildings.iter().any(|b| {
                    (cand_x - b.x).abs() < (bw + b.width) * 0.5 + 20.0
                        && (cand_y - b.y).abs() < (bh + b.height) * 0.5 + 20.0
                });
                if collides_b {
                    continue;
                }
                let collides_m = engine
                    .minerals
                    .iter()
                    .any(|m| ((cand_x - m.x).powi(2) + (cand_y - m.y).powi(2)).sqrt() < 85.0);
                if collides_m {
                    continue;
                }
                let collides_obs = engine.obstacles.iter().any(|obs| {
                    let obs_cx = obs.x + obs.width * 0.5;
                    let obs_cy = obs.y + obs.height * 0.5;
                    (cand_x - obs_cx).abs() < (bw + obs.width) * 0.5 + 15.0
                        && (cand_y - obs_cy).abs() < (bh + obs.height) * 0.5 + 15.0
                });
                if collides_obs {
                    continue;
                }

                return Some((cand_x, cand_y));
            }
        }
    }

    let mut team_ccs: Vec<&crate::components::Building> = engine
        .buildings
        .iter()
        .filter(|b| b.team == team && b.building_type == BuildingType::CommandCenter)
        .collect();

    team_ccs.sort_by_key(|b| std::cmp::Reverse(b.id));

    let default_pos = engine.base_pos(team);
    let newest_cc_pos = team_ccs.first().map(|b| (b.x, b.y)).unwrap_or(default_pos);

    let mut anchors = Vec::new();
    anchors.push(newest_cc_pos);

    for b in &engine.buildings {
        if b.team == team && b.building_type != BuildingType::CommandCenter {
            let dist_sq = (b.x - newest_cc_pos.0).powi(2) + (b.y - newest_cc_pos.1).powi(2);
            if dist_sq < 600.0_f32.powi(2) {
                anchors.push((b.x, b.y));
            }
        }
    }

    for cc in team_ccs.iter().skip(1) {
        anchors.push((cc.x, cc.y));
    }

    for b in &engine.buildings {
        if b.team == team && !anchors.contains(&(b.x, b.y)) {
            anchors.push((b.x, b.y));
        }
    }

    for (a_idx, anchor) in anchors.iter().enumerate() {
        let dy: f32 = (crate::map::MAP_HEIGHT * 0.5) - anchor.1;
        let dx: f32 = (crate::map::MAP_WIDTH * 0.5) - anchor.0;
        let to_center_ang = dy.atan2(dx);
        let angle_bias = match b_type {
            BuildingType::TurretTower => Some(to_center_ang + 0.1),
            BuildingType::PowerReactor => Some(to_center_ang - 0.5),
            BuildingType::Barracks => Some(to_center_ang),
            _ => Some(to_center_ang),
        };

        let mut current_radius = 80.0;
        let max_radius = if a_idx == 0 { 1400.0 } else { 550.0 };
        let radius_step = 65.0;

        while current_radius <= max_radius {
            let num_angles =
                ((2.0 * std::f32::consts::PI * current_radius) / 70.0).max(8.0) as usize;
            for i in 0..num_angles {
                let angle = if let Some(base_ang) = angle_bias {
                    let offset = if i % 2 == 0 {
                        (i / 2) as f32
                    } else {
                        -(i.div_ceil(2) as f32)
                    };
                    base_ang + offset * (std::f32::consts::PI * 2.0 / num_angles as f32)
                } else {
                    (i as f32) * (std::f32::consts::PI * 2.0 / num_angles as f32)
                };

                let cand_x = anchor.0 + current_radius * angle.cos();
                let cand_y = anchor.1 + current_radius * angle.sin();

                if !(120.0..=(crate::map::MAP_WIDTH - 120.0)).contains(&cand_x)
                    || !(120.0..=(crate::map::MAP_HEIGHT - 120.0)).contains(&cand_y)
                {
                    continue;
                }

                let collides_building = engine.buildings.iter().any(|b| {
                    let clearance_x = (bw + b.width) * 0.5 + 16.0;
                    let clearance_y = (bh + b.height) * 0.5 + 16.0;
                    (cand_x - b.x).abs() < clearance_x && (cand_y - b.y).abs() < clearance_y
                });
                if collides_building {
                    continue;
                }

                let collides_minerals = engine.minerals.iter().any(|m| {
                    let dist = ((cand_x - m.x).powi(2) + (cand_y - m.y).powi(2)).sqrt();
                    dist < 75.0
                });
                if collides_minerals {
                    continue;
                }

                let collides_obstacles = engine.obstacles.iter().any(|obs| {
                    let obs_cx = obs.x + obs.width * 0.5;
                    let obs_cy = obs.y + obs.height * 0.5;
                    let clearance_x = (bw + obs.width) * 0.5 + 12.0;
                    let clearance_y = (bh + obs.height) * 0.5 + 12.0;
                    (cand_x - obs_cx).abs() < clearance_x && (cand_y - obs_cy).abs() < clearance_y
                });
                if collides_obstacles {
                    continue;
                }

                let collides_enemies = engine.units.iter().any(|u| {
                    u.team != team
                        && ((cand_x - u.x).powi(2) + (cand_y - u.y).powi(2)).sqrt() < 80.0
                });
                if collides_enemies {
                    continue;
                }

                return Some((cand_x, cand_y));
            }
            current_radius += radius_step;
        }
    }

    None
}
