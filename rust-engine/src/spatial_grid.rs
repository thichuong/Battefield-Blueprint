use crate::components::Team;
pub use crate::config::{CELL_SIZE, COLS, ROWS};
use glam::Vec2;

#[derive(Clone, Copy, Debug)]
pub struct GridEntity {
    pub id: u32,
    pub team: Team,
    pub pos: Vec2,
    pub radius: f32,
    pub is_building: bool,
    pub is_threat: bool,
}

pub struct SpatialGrid {
    cells: Vec<Vec<GridEntity>>,
}

impl Default for SpatialGrid {
    fn default() -> Self {
        Self::new()
    }
}

impl SpatialGrid {
    pub fn new() -> Self {
        let total_cells = COLS * ROWS;
        let mut cells = Vec::with_capacity(total_cells);
        for _ in 0..total_cells {
            cells.push(Vec::with_capacity(8));
        }
        Self { cells }
    }

    pub fn clear(&mut self) {
        for cell in self.cells.iter_mut() {
            cell.clear();
        }
    }

    /// Tái tạo lưới từ danh sách đơn vị và công trình
    pub fn rebuild(
        &mut self,
        units: &[crate::components::Unit],
        buildings: &[crate::components::Building],
    ) {
        self.clear();
        for u in units {
            self.insert(GridEntity {
                id: u.id,
                team: u.team,
                pos: Vec2::new(u.x, u.y),
                radius: 12.0,
                is_building: false,
                is_threat: u.unit_type != crate::components::UnitType::Engineer,
            });
        }
        for b in buildings {
            let is_threat = b.building_type == crate::components::BuildingType::TurretTower
                || b.building_type == crate::components::BuildingType::CommandCenter;
            self.insert(GridEntity {
                id: b.id,
                team: b.team,
                pos: Vec2::new(b.x, b.y),
                radius: b.width.max(b.height) * 0.5,
                is_building: true,
                is_threat,
            });
        }
    }

    #[inline]
    fn cell_coord(&self, x: f32, y: f32) -> (usize, usize) {
        let c = ((x / CELL_SIZE).max(0.0) as usize).min(COLS - 1);
        let r = ((y / CELL_SIZE).max(0.0) as usize).min(ROWS - 1);
        (c, r)
    }

    #[inline]
    fn cell_index(&self, c: usize, r: usize) -> usize {
        r * COLS + c
    }

    pub fn insert(&mut self, entity: GridEntity) {
        let (c, r) = self.cell_coord(entity.pos.x, entity.pos.y);
        let idx = self.cell_index(c, r);
        self.cells[idx].push(entity);
    }

    /// Tìm mục tiêu kẻ địch gần nhất trong bán kính max_range (Ưu tiên đe dọa vũ trang: lính, tháp pháo, CC)
    pub fn find_nearest_enemy(
        &self,
        pos: Vec2,
        max_range: f32,
        my_team: Team,
    ) -> Option<(u32, bool, f32)> {
        let min_c = (((pos.x - max_range) / CELL_SIZE).max(0.0) as usize).min(COLS - 1);
        let max_c = (((pos.x + max_range) / CELL_SIZE).max(0.0) as usize).min(COLS - 1);
        let min_r = (((pos.y - max_range) / CELL_SIZE).max(0.0) as usize).min(ROWS - 1);
        let max_r = (((pos.y + max_range) / CELL_SIZE).max(0.0) as usize).min(ROWS - 1);

        let mut best_threat: Option<(u32, bool, f32)> = None;
        let mut best_threat_dist_sq = max_range * max_range;
        let mut best_other: Option<(u32, bool, f32)> = None;
        let mut best_other_dist_sq = max_range * max_range;

        for r in min_r..=max_r {
            for c in min_c..=max_c {
                let idx = self.cell_index(c, r);
                for entity in &self.cells[idx] {
                    if entity.team != my_team {
                        let d_sq = pos.distance_squared(entity.pos);
                        let total_range = max_range + entity.radius;
                        if d_sq <= total_range * total_range {
                            if entity.is_threat {
                                if d_sq < best_threat_dist_sq {
                                    best_threat_dist_sq = d_sq;
                                    best_threat = Some((entity.id, entity.is_building, d_sq.sqrt()));
                                }
                            } else {
                                if d_sq < best_other_dist_sq {
                                    best_other_dist_sq = d_sq;
                                    best_other = Some((entity.id, entity.is_building, d_sq.sqrt()));
                                }
                            }
                        }
                    }
                }
            }
        }

        best_threat.or(best_other)
    }

    /// Tìm tất cả thực thể trong bán kính radius
    pub fn query_radius(&self, pos: Vec2, radius: f32) -> Vec<GridEntity> {
        let min_c = (((pos.x - radius) / CELL_SIZE).max(0.0) as usize).min(COLS - 1);
        let max_c = (((pos.x + radius) / CELL_SIZE).max(0.0) as usize).min(COLS - 1);
        let min_r = (((pos.y - radius) / CELL_SIZE).max(0.0) as usize).min(ROWS - 1);
        let max_r = (((pos.y + radius) / CELL_SIZE).max(0.0) as usize).min(ROWS - 1);

        let mut results = Vec::new();
        let _r_sq = radius * radius;

        for r in min_r..=max_r {
            for c in min_c..=max_c {
                let idx = self.cell_index(c, r);
                for entity in &self.cells[idx] {
                    let d_sq = pos.distance_squared(entity.pos);
                    let reach = radius + entity.radius;
                    if d_sq <= reach * reach {
                        results.push(*entity);
                    }
                }
            }
        }

        results
    }
}
