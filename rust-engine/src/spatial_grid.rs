use crate::components::Team;
pub use crate::config::{CELL_SIZE, COLS, ROWS};
use glam::Vec2;
use std::collections::HashMap;

#[derive(Clone, Copy, Debug)]
pub struct GridEntity {
    pub id: u32,
    pub index: usize,
    pub team: Team,
    pub faction: u8,
    pub pos: Vec2,
    pub radius: f32,
    pub is_building: bool,
    pub is_threat: bool,
}

pub struct SpatialGrid {
    cells: Vec<Vec<GridEntity>>,
    pub unit_id_to_index: HashMap<u32, usize>,
    pub building_id_to_index: HashMap<u32, usize>,
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
        Self {
            cells,
            unit_id_to_index: HashMap::with_capacity(512),
            building_id_to_index: HashMap::with_capacity(128),
        }
    }

    pub fn clear(&mut self) {
        for cell in self.cells.iter_mut() {
            cell.clear();
        }
        self.unit_id_to_index.clear();
        self.building_id_to_index.clear();
    }

    /// Tái tạo lưới từ danh sách đơn vị và công trình kèm thông tin phe phái
    pub fn rebuild_with_factions(
        &mut self,
        units: &[crate::components::Unit],
        buildings: &[crate::components::Building],
        factions: &[u8; 8],
    ) {
        self.clear();
        for (idx, u) in units.iter().enumerate() {
            self.unit_id_to_index.insert(u.id, idx);
            let faction = factions[u.team.index() % 8];
            self.insert(GridEntity {
                id: u.id,
                index: idx,
                team: u.team,
                faction,
                pos: Vec2::new(u.x, u.y),
                radius: 12.0,
                is_building: false,
                is_threat: u.unit_type != crate::components::UnitType::Engineer,
            });
        }
        for (idx, b) in buildings.iter().enumerate() {
            self.building_id_to_index.insert(b.id, idx);
            let faction = factions[b.team.index() % 8];
            let is_threat = b.building_type == crate::components::BuildingType::TurretTower
                || b.building_type == crate::components::BuildingType::CommandCenter;
            self.insert(GridEntity {
                id: b.id,
                index: idx,
                team: b.team,
                faction,
                pos: Vec2::new(b.x, b.y),
                radius: b.width.max(b.height) * 0.5,
                is_building: true,
                is_threat,
            });
        }
    }

    /// Tái tạo lưới từ danh sách đơn vị và công trình (Tương thích ngược 2 phe Player & Enemy)
    pub fn rebuild(
        &mut self,
        units: &[crate::components::Unit],
        buildings: &[crate::components::Building],
    ) {
        let mut default_factions = [0u8; 8];
        for faction in default_factions.iter_mut().skip(1) {
            *faction = 1;
        }
        self.rebuild_with_factions(units, buildings, &default_factions);
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

    /// Tìm mục tiêu kẻ địch thuộc phe khác gần nhất trong bán kính max_range (Ưu tiên đe dọa vũ trang: lính, tháp pháo, CC)
    pub fn find_nearest_hostile(
        &self,
        pos: Vec2,
        max_range: f32,
        my_faction: u8,
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
                    if entity.faction != my_faction {
                        let d_sq = pos.distance_squared(entity.pos);
                        let total_range = max_range + entity.radius;
                        if d_sq <= total_range * total_range {
                            if entity.is_threat {
                                if d_sq < best_threat_dist_sq {
                                    best_threat_dist_sq = d_sq;
                                    best_threat =
                                        Some((entity.id, entity.is_building, d_sq.sqrt()));
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

    /// Tìm mục tiêu kẻ địch gần nhất trong bán kính max_range (Tương thích ngược kiểm tra team != my_team)
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
                                    best_threat =
                                        Some((entity.id, entity.is_building, d_sq.sqrt()));
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

    /// Duyệt qua tất cả thực thể thù địch trong bán kính radius (Zero Heap Allocation)
    #[inline]
    pub fn for_each_hostile_in_radius<F>(&self, pos: Vec2, radius: f32, my_faction: u8, mut f: F)
    where
        F: FnMut(&GridEntity, f32),
    {
        let min_c = (((pos.x - radius) / CELL_SIZE).max(0.0) as usize).min(COLS - 1);
        let max_c = (((pos.x + radius) / CELL_SIZE).max(0.0) as usize).min(COLS - 1);
        let min_r = (((pos.y - radius) / CELL_SIZE).max(0.0) as usize).min(ROWS - 1);
        let max_r = (((pos.y + radius) / CELL_SIZE).max(0.0) as usize).min(ROWS - 1);

        for r in min_r..=max_r {
            for c in min_c..=max_c {
                let idx = self.cell_index(c, r);
                for entity in &self.cells[idx] {
                    if entity.faction != my_faction {
                        let d_sq = pos.distance_squared(entity.pos);
                        let reach = radius + entity.radius;
                        if d_sq <= reach * reach {
                            f(entity, d_sq.sqrt());
                        }
                    }
                }
            }
        }
    }

    /// Kiểm tra nhanh xem có quân thù địch trong bán kính không (phục vụ placement, reflexes)
    #[inline]
    pub fn has_hostile_in_radius(&self, pos: Vec2, radius: f32, my_faction: u8) -> bool {
        let min_c = (((pos.x - radius) / CELL_SIZE).max(0.0) as usize).min(COLS - 1);
        let max_c = (((pos.x + radius) / CELL_SIZE).max(0.0) as usize).min(COLS - 1);
        let min_r = (((pos.y - radius) / CELL_SIZE).max(0.0) as usize).min(ROWS - 1);
        let max_r = (((pos.y + radius) / CELL_SIZE).max(0.0) as usize).min(ROWS - 1);

        for r in min_r..=max_r {
            for c in min_c..=max_c {
                let idx = self.cell_index(c, r);
                for entity in &self.cells[idx] {
                    if entity.faction != my_faction {
                        let d_sq = pos.distance_squared(entity.pos);
                        let reach = radius + entity.radius;
                        if d_sq <= reach * reach {
                            return true;
                        }
                    }
                }
            }
        }
        false
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
