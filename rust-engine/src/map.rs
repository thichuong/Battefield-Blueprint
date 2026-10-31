use crate::components::*;
pub use crate::config::{MAP_HEIGHT, MAP_WIDTH};

pub fn generate_map_elements() -> (Vec<MineralNode>, Vec<Obstacle>) {
    let mut minerals = Vec::new();
    let mut obstacles = Vec::new();

    // Mỏ khoáng sản căn cứ Player (Góc Tây-Nam: x ~ 300, y ~ 2000)
    minerals.push(MineralNode {
        id: 1,
        x: 260.0,
        y: 1950.0,
        amount: 2500.0,
    });
    minerals.push(MineralNode {
        id: 2,
        x: 340.0,
        y: 1950.0,
        amount: 2500.0,
    });
    minerals.push(MineralNode {
        id: 3,
        x: 420.0,
        y: 1970.0,
        amount: 2500.0,
    });

    // Mỏ khoáng sản căn cứ AI (Góc Đông-Bắc: x ~ 2100, y ~ 400)
    minerals.push(MineralNode {
        id: 10,
        x: 2050.0,
        y: 450.0,
        amount: 2500.0,
    });
    minerals.push(MineralNode {
        id: 11,
        x: 2130.0,
        y: 450.0,
        amount: 2500.0,
    });
    minerals.push(MineralNode {
        id: 12,
        x: 2210.0,
        y: 430.0,
        amount: 2500.0,
    });

    // Mỏ khoáng sản tranh chấp ở trung tâm (Center expansions)
    minerals.push(MineralNode {
        id: 20,
        x: 1100.0,
        y: 1200.0,
        amount: 4000.0,
    });
    minerals.push(MineralNode {
        id: 21,
        x: 1300.0,
        y: 1200.0,
        amount: 4000.0,
    });
    minerals.push(MineralNode {
        id: 22,
        x: 500.0,
        y: 1000.0,
        amount: 3000.0,
    });
    minerals.push(MineralNode {
        id: 23,
        x: 1900.0,
        y: 1400.0,
        amount: 3000.0,
    });

    // Địa hình vật cản (Blueprint Obstacles / Barrier Walls)
    obstacles.push(Obstacle {
        x: 700.0,
        y: 1500.0,
        width: 280.0,
        height: 40.0,
    });
    obstacles.push(Obstacle {
        x: 940.0,
        y: 1300.0,
        width: 40.0,
        height: 240.0,
    });

    // Tường trung tâm tạo hẻm chokepoint
    obstacles.push(Obstacle {
        x: 1180.0,
        y: 800.0,
        width: 40.0,
        height: 260.0,
    });
    obstacles.push(Obstacle {
        x: 1180.0,
        y: 1340.0,
        width: 40.0,
        height: 260.0,
    });

    // Vách đá gần căn cứ Enemy
    obstacles.push(Obstacle {
        x: 1420.0,
        y: 860.0,
        width: 280.0,
        height: 40.0,
    });
    obstacles.push(Obstacle {
        x: 1420.0,
        y: 860.0,
        width: 40.0,
        height: 240.0,
    });

    (minerals, obstacles)
}
