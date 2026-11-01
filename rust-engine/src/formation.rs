use crate::components::{FormationType, UnitType};
use crate::map::{MAP_HEIGHT, MAP_WIDTH};

/// Tính tọa độ (world_x, world_y, offset_x, offset_y) cho danh sách đơn vị theo FormationType đã chọn.
/// Kết quả trả về danh sách tuple: (unit_id, target_x, target_y, offset_x, offset_y).
/// - start_pos: Tâm hiện tại của nhóm quân
/// - target_pos: Vị trí người chơi hoặc AI ra lệnh tới
pub fn compute_formation_positions(
    units: &[(u32, UnitType)],
    start_pos: (f32, f32),
    target_pos: (f32, f32),
    formation: FormationType,
) -> Vec<(u32, f32, f32, f32, f32)> {
    let count = units.len();
    if count == 0 {
        return Vec::new();
    }

    // 1. Phân loại và sắp xếp đơn vị theo vai trò hình thành đội hình:
    // Tier 0: Tank / Cận chiến / Cảm tử (Vanguard, Titan, ShockTrooper, AcidBaneling) -> Tuyến đầu
    // Tier 1: Cận chiến tầm trung / Xạ thủ cơ bản (Rifleman, Swordmaster) -> Trung tuyến
    // Tier 2: Pháo binh / Pháp sư / Thợ mỏ (Laser, Rocketeer, Warlock, Engineer) -> Hậu tuyến / Lõi bảo vệ
    let mut sorted_units: Vec<(usize, u32, UnitType)> = units
        .iter()
        .enumerate()
        .map(|(orig_idx, (uid, ut))| (orig_idx, *uid, *ut))
        .collect();

    sorted_units.sort_by_key(|(_, _, ut)| ut.formation_role_tier());

    // 2. Tính toán Vector hướng tiến quân (Forward & Right)
    let dx = target_pos.0 - start_pos.0;
    let dy = target_pos.1 - start_pos.1;
    let dist = (dx * dx + dy * dy).sqrt();

    let (fwd_x, fwd_y) = if dist > 0.1 {
        (dx / dist, dy / dist)
    } else {
        (1.0, 0.0) // Mặc định hướng sang phải nếu nhấp tại chỗ
    };

    // Vector vuông góc sang phải
    let (right_x, right_y) = (-fwd_y, fwd_x);

    // 3. Tính toán các điểm offset trong hệ tọa độ cục bộ (local_right, local_forward)
    // local_forward > 0 là tiến về trước, < 0 là lùi về sau.
    let mut local_slots: Vec<(f32, f32)> = Vec::with_capacity(count);

    match formation {
        FormationType::Line => {
            // Hàng Ngang (Phalanx): 2-3 hàng vuông góc với hướng di chuyển
            // Chiều rộng mỗi hàng tối đa 6-8 quân, khoảng cách ngang 32px, khoảng cách hàng 32px
            let cols = count.min(7).max(1);
            let spacing_x = 32.0;
            let spacing_y = 32.0;

            for i in 0..count {
                let row = i / cols;
                let col = i % cols;
                let row_count = ((count - row * cols).min(cols)) as f32;
                let col_offset = ((col as f32) - (row_count - 1.0) * 0.5) * spacing_x;
                let row_offset = -(row as f32) * spacing_y; // Hàng đầu row=0 ở phía trước
                local_slots.push((col_offset, row_offset));
            }
        }
        FormationType::Wedge => {
            // Mũi Tên V (Spearhead):
            // Quân đầu tiên ở đỉnh (0, 0).
            // Các quân tiếp theo chia đều sang 2 cánh lùi dần về sau.
            let spacing_x = 28.0;
            let spacing_y = 26.0;

            local_slots.push((0.0, 0.0)); // Đỉnh nhọn

            for i in 1..count {
                let rank = ((i + 1) / 2) as f32;
                let sign = if i % 2 == 1 { 1.0 } else { -1.0 };
                let col_offset = sign * rank * spacing_x;
                let row_offset = -rank * spacing_y;
                local_slots.push((col_offset, row_offset));
            }
        }
        FormationType::Box => {
            // Ô Vuông Hộ Vệ (Protect Square):
            // Xếp các đơn vị thành khung vuông bên ngoài, bảo vệ lõi ở giữa
            // Tính số vị trí trên mỗi cạnh của hình vuông
            let side = ((count as f32).sqrt().ceil() as usize).max(3);
            let spacing = 34.0;
            let half_span = ((side - 1) as f32) * spacing * 0.5;

            // Thu thập các vị trí trên viền khung vuông trước, sau đó tới phần ruột
            let mut border_positions = Vec::new();
            let mut inner_positions = Vec::new();

            for r in 0..side {
                for c in 0..side {
                    let rx = ((c as f32) * spacing) - half_span;
                    let ry = -(((r as f32) * spacing) - half_span); // r=0 ở trên cùng (tiến về trước)
                    if r == 0 || r == side - 1 || c == 0 || c == side - 1 {
                        border_positions.push((rx, ry));
                    } else {
                        inner_positions.push((rx, ry));
                    }
                }
            }

            // Xếp theo thứ tự: Viền trước (cho Tank/Shock), ruột trong (cho Pháo/Laser)
            for pos in border_positions.into_iter().chain(inner_positions.into_iter()) {
                if local_slots.len() < count {
                    local_slots.push(pos);
                } else {
                    break;
                }
            }

            // Nếu vẫn còn thiếu (do làm tròn):
            while local_slots.len() < count {
                let extra_idx = local_slots.len() - side * side;
                local_slots.push((((extra_idx as f32) * 25.0) - half_span, -half_span - 30.0));
            }
        }
        FormationType::Scatter => {
            // Phân Tán (Anti-AoE):
            // Lưới so le tản rộng với khoảng cách > 52px (lớn hơn bán kính đạn nổ 30-45px)
            let cols = count.min(6).max(1);
            let spacing_x = 54.0;
            let spacing_y = 52.0;

            for i in 0..count {
                let row = i / cols;
                let col = i % cols;
                let row_count = ((count - row * cols).min(cols)) as f32;
                let stagger = if row % 2 == 1 { spacing_x * 0.5 } else { 0.0 };
                let col_offset = ((col as f32) - (row_count - 1.0) * 0.5) * spacing_x + stagger;
                let row_offset = -(row as f32) * spacing_y;
                local_slots.push((col_offset, row_offset));
            }
        }
        FormationType::Circle => {
            // Vòng Cung Vây Bọc (Crescent Arc):
            // Hình bán nguyệt/vòng cung mở về phía trước ôm lấy mục tiêu
            let base_radius = (35.0 + (count as f32) * 5.5).min(220.0);
            let span_angle = std::f32::consts::PI * 0.85; // Cung 153 độ
            let step = if count > 1 { span_angle / ((count - 1) as f32) } else { 0.0 };

            for i in 0..count {
                let angle = -span_angle * 0.5 + (i as f32) * step;
                // angle = 0 là chính giữa, dương sang phải, âm sang trái
                let col_offset = base_radius * angle.sin();
                let row_offset = -base_radius * (1.0 - angle.cos()) * 0.6;
                local_slots.push((col_offset, row_offset));
            }
        }
        FormationType::Column => {
            // Cột Dọc Đột Kích (Blitz Column):
            // 2 hàng dọc hẹp luồn lách chokepoint
            let spacing_x = 24.0;
            let spacing_y = 30.0;

            for i in 0..count {
                let row = i / 2;
                let col = i % 2;
                let col_offset = if col == 0 { -spacing_x * 0.5 } else { spacing_x * 0.5 };
                let row_offset = -(row as f32) * spacing_y;
                local_slots.push((col_offset, row_offset));
            }
        }
    }

    // 4. Biến đổi tọa độ cục bộ sang Tọa độ Thế giới (World Coordinates)
    let min_bound = 60.0;
    let max_bound_x = MAP_WIDTH - 60.0;
    let max_bound_y = MAP_HEIGHT - 60.0;

    let mut result = Vec::with_capacity(count);

    for (i, &(_, uid, _)) in sorted_units.iter().enumerate() {
        let (loc_r, loc_f) = local_slots[i];
        let off_x = loc_r * right_x + loc_f * fwd_x;
        let off_y = loc_r * right_y + loc_f * fwd_y;

        let world_x = (target_pos.0 + off_x).clamp(min_bound, max_bound_x);
        let world_y = (target_pos.1 + off_y).clamp(min_bound, max_bound_y);

        result.push((uid, world_x, world_y, off_x, off_y));
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::UnitType;

    #[test]
    fn test_compute_formation_positions_count_and_bounds() {
        let units = vec![
            (1, UnitType::Rifleman),
            (2, UnitType::Vanguard),
            (3, UnitType::LaserMarksman),
            (4, UnitType::ShockTrooper),
        ];

        for &form in &FormationType::ALL {
            let res = compute_formation_positions(
                &units,
                (500.0, 500.0),
                (1000.0, 500.0), // Tiến sang phải
                form,
            );
            assert_eq!(res.len(), 4, "Phải đủ 4 vị trí cho 4 quân");
            for (_, wx, wy, _, _) in &res {
                assert!(*wx >= 60.0 && *wx <= MAP_WIDTH - 60.0);
                assert!(*wy >= 60.0 && *wy <= MAP_HEIGHT - 60.0);
            }
        }
    }

    #[test]
    fn test_formation_role_sorting() {
        let units = vec![
            (1, UnitType::LaserMarksman), // Tier 2
            (2, UnitType::Vanguard),      // Tier 0
            (3, UnitType::Rifleman),      // Tier 1
        ];

        let res = compute_formation_positions(
            &units,
            (100.0, 100.0),
            (500.0, 100.0), // Đi sang phải (fwd=(1, 0))
            FormationType::Line,
        );

        // Vanguard (id=2) phải được xếp ở hàng đầu tiên (row_offset = 0)
        let vanguard_pos = res.iter().find(|(uid, _, _, _, _)| *uid == 2).unwrap();
        let laser_pos = res.iter().find(|(uid, _, _, _, _)| *uid == 1).unwrap();

        // Kiểm tra tọa độ X của Vanguard phải >= Laser khi di chuyển sang phải
        assert!(vanguard_pos.1 >= laser_pos.1 - 1.0);
    }
}

