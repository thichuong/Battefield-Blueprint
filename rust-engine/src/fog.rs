use crate::config::{MAP_HEIGHT, MAP_WIDTH};
pub use crate::config::{CELL_H, CELL_W, FOG_COLS, FOG_ROWS};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Visibility {
    Hidden = 0,   // Chưa từng đi qua (Đen/Tối mờ Blueprint)
    Explored = 1, // Đã từng nhìn thấy (Thấy đất/nhà tĩnh, không thấy lính địch)
    Visible = 2,  // Đang trong tầm mắt trực tiếp
}

pub struct FogOfWar {
    pub grid: [Visibility; FOG_COLS * FOG_ROWS],
}

impl Default for FogOfWar {
    fn default() -> Self {
        Self::new()
    }
}

impl FogOfWar {
    pub fn new() -> Self {
        Self {
            grid: [Visibility::Hidden; FOG_COLS * FOG_ROWS],
        }
    }

    pub fn reset_visibility_to_explored(&mut self) {
        for cell in self.grid.iter_mut() {
            if *cell == Visibility::Visible {
                *cell = Visibility::Explored;
            }
        }
    }

    pub fn reveal_circle(&mut self, world_x: f32, world_y: f32, radius: f32) {
        let min_c = ((world_x - radius).max(0.0) / CELL_W) as usize;
        let max_c = (((world_x + radius).min(MAP_WIDTH - 1.0)) / CELL_W) as usize;
        let min_r = ((world_y - radius).max(0.0) / CELL_H) as usize;
        let max_r = (((world_y + radius).min(MAP_HEIGHT - 1.0)) / CELL_H) as usize;

        let r2 = radius * radius;

        for r in min_r..=max_r.min(FOG_ROWS - 1) {
            let cy = (r as f32 + 0.5) * CELL_H;
            for c in min_c..=max_c.min(FOG_COLS - 1) {
                let cx = (c as f32 + 0.5) * CELL_W;
                let dx = cx - world_x;
                let dy = cy - world_y;
                if dx * dx + dy * dy <= r2 {
                    self.grid[r * FOG_COLS + c] = Visibility::Visible;
                }
            }
        }
    }

    pub fn is_visible(&self, world_x: f32, world_y: f32) -> bool {
        if !(0.0..MAP_WIDTH).contains(&world_x) || !(0.0..MAP_HEIGHT).contains(&world_y) {
            return false;
        }
        let c = (world_x / CELL_W) as usize;
        let r = (world_y / CELL_H) as usize;
        let c = c.min(FOG_COLS - 1);
        let r = r.min(FOG_ROWS - 1);
        self.grid[r * FOG_COLS + c] == Visibility::Visible
    }
}
