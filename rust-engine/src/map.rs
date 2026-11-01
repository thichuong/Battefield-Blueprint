use crate::components::*;
pub use crate::config::{MAP_HEIGHT, MAP_WIDTH};

/// Trả về 8 vị trí căn cứ của 8 người chơi phân bố đều quanh bản đồ 4800x4800
pub fn get_spawn_positions() -> [(f32, f32); 8] {
    [
        (550.0, 4250.0),  // Slot 0 (Tây-Nam - Player chính)
        (4250.0, 550.0),  // Slot 1 (Đông-Bắc)
        (550.0, 550.0),   // Slot 2 (Tây-Bắc)
        (4250.0, 4250.0), // Slot 3 (Đông-Nam)
        (2400.0, 4350.0), // Slot 4 (Nam - Giữa dưới)
        (2400.0, 450.0),  // Slot 5 (Bắc - Giữa trên)
        (450.0, 2400.0),  // Slot 6 (Tây - Giữa trái)
        (4350.0, 2400.0), // Slot 7 (Đông - Giữa phải)
    ]
}

/// Trả về điểm tập kết ban đầu của từng slot hướng về trung tâm bản đồ
pub fn get_default_rally_point(slot: usize) -> (f32, f32) {
    let spawns = get_spawn_positions();
    let (sx, sy) = spawns[slot % 8];
    let dx = 2400.0 - sx;
    let dy = 2400.0 - sy;
    let len = (dx * dx + dy * dy).sqrt().max(1.0);
    // Điểm tập kết cách căn cứ 160px hướng về tâm
    (sx + (dx / len) * 160.0, sy + (dy / len) * 160.0)
}

pub fn generate_map_elements() -> (Vec<MineralNode>, Vec<Obstacle>) {
    let mut minerals = Vec::new();
    let mut obstacles = Vec::new();

    let spawns = get_spawn_positions();

    // 1. Mỏ khoáng sản khởi đầu cho 8 căn cứ (3 mỏ mỗi căn cứ = 24 mỏ)
    for (slot, &(sx, sy)) in spawns.iter().enumerate() {
        let base_id = (slot as u32 + 1) * 10;
        // Vector từ tâm (2400, 2400) đến căn cứ
        let dx = sx - 2400.0;
        let dy = sy - 2400.0;
        let dist = (dx * dx + dy * dy).sqrt().max(1.0);
        let ux = dx / dist;
        let uy = dy / dist;
        // Pháp tuyến vuông góc
        let nx = -uy;
        let ny = ux;

        // Bố trí 3 mỏ khoáng sản phía sau hoặc chếch bên căn cứ
        minerals.push(MineralNode {
            id: base_id + 1,
            x: sx + ux * 100.0 - nx * 80.0,
            y: sy + uy * 100.0 - ny * 80.0,
            amount: 2500.0,
        });
        minerals.push(MineralNode {
            id: base_id + 2,
            x: sx + ux * 120.0,
            y: sy + uy * 120.0,
            amount: 2500.0,
        });
        minerals.push(MineralNode {
            id: base_id + 3,
            x: sx + ux * 100.0 + nx * 80.0,
            y: sy + uy * 100.0 + ny * 80.0,
            amount: 2500.0,
        });
    }

    // 2. 8 cụm mỏ mở rộng tầm trung (Natural Expansions) cách tâm ~1400px (mỗi cụm 2 mỏ = 16 mỏ)
    let expansion_angles = [
        std::f32::consts::PI * 0.25,
        std::f32::consts::PI * 0.75,
        std::f32::consts::PI * 1.25,
        std::f32::consts::PI * 1.75,
        0.0,
        std::f32::consts::PI * 0.5,
        std::f32::consts::PI,
        std::f32::consts::PI * 1.5,
    ];

    for (idx, &ang) in expansion_angles.iter().enumerate() {
        let exp_id = 100 + (idx as u32) * 5;
        let r = 1400.0;
        let cx = 2400.0 + ang.cos() * r;
        let cy = 2400.0 + ang.sin() * r;

        minerals.push(MineralNode {
            id: exp_id + 1,
            x: cx - 40.0 * ang.sin(),
            y: cy + 40.0 * ang.cos(),
            amount: 3000.0,
        });
        minerals.push(MineralNode {
            id: exp_id + 2,
            x: cx + 40.0 * ang.sin(),
            y: cy - 40.0 * ang.cos(),
            amount: 3000.0,
        });
    }

    // 3. Cụm mỏ khoáng sản siêu giàu ở TRUNG TÂM bản đồ (Center Dispute, 4 mỏ lớn)
    minerals.push(MineralNode {
        id: 200,
        x: 2280.0,
        y: 2280.0,
        amount: 4500.0,
    });
    minerals.push(MineralNode {
        id: 201,
        x: 2520.0,
        y: 2280.0,
        amount: 4500.0,
    });
    minerals.push(MineralNode {
        id: 202,
        x: 2280.0,
        y: 2520.0,
        amount: 4500.0,
    });
    minerals.push(MineralNode {
        id: 203,
        x: 2520.0,
        y: 2520.0,
        amount: 4500.0,
    });

    // 4. Địa hình chướng ngại vật (Blueprint Obstacles / Barrier Walls)
    // Tường bảo vệ quanh trung tâm tạo các chokepoint chiến thuật 8 hướng
    obstacles.push(Obstacle {
        x: 2150.0,
        y: 1950.0,
        width: 500.0,
        height: 40.0,
    });
    obstacles.push(Obstacle {
        x: 2150.0,
        y: 2810.0,
        width: 500.0,
        height: 40.0,
    });
    obstacles.push(Obstacle {
        x: 1950.0,
        y: 2150.0,
        width: 40.0,
        height: 500.0,
    });
    obstacles.push(Obstacle {
        x: 2810.0,
        y: 2150.0,
        width: 40.0,
        height: 500.0,
    });

    // Các vách đá phân cách giữa các làn đường tiếp cận (4 góc chéo)
    obstacles.push(Obstacle {
        x: 1400.0,
        y: 1400.0,
        width: 250.0,
        height: 40.0,
    });
    obstacles.push(Obstacle {
        x: 3150.0,
        y: 1400.0,
        width: 250.0,
        height: 40.0,
    });
    obstacles.push(Obstacle {
        x: 1400.0,
        y: 3360.0,
        width: 250.0,
        height: 40.0,
    });
    obstacles.push(Obstacle {
        x: 3150.0,
        y: 3360.0,
        width: 250.0,
        height: 40.0,
    });

    (minerals, obstacles)
}
