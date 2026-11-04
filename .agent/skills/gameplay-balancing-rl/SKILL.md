---
name: gameplay-balancing-rl
description: "Quy trình cân bằng hệ thống kinh tế 3 tài nguyên, ma trận 4 hệ sát thương, mở rộng cây công nghệ 4 nhánh, lập trình thẻ lệnh Auto-Play và huấn luyện Deep RL Dual-Head MLP trên GPU CUDA / CPU đa luồng."
---

# SKILL: Gameplay Balancing & Deep RL (Battlefield Blueprint)

Kỹ năng này hướng dẫn Agent cách hiệu chỉnh các thông số cân bằng trò chơi (chiến đấu, kinh tế, công nghệ), mở rộng hệ thống lập trình Auto-Play và huấn luyện mạng trí tuệ nhân tạo Deep RL.

---

## 1. Ma Trận Cân Bằng Chiến Đấu & Kinh Tế

### Ma trận 4 Hệ Sát thương
| Hệ Sát Thương | Mã Màu Neon | Đặc tính chiến thuật |
| :--- | :--- | :--- |
| **Physical** | `#FFFFFF` / `#FACC15` | Sát thương cơ bản; bị trừ thẳng bởi Kháng Vật Lý ($DMG \times (1 - Resist_{phys})$). |
| **Energy** | `#06B6D4` | Khắc chế giáp vật lý; tốc độ đạn đạo tức thời (Beam/Laser). |
| **Explosive** | `#F97316` | Gây sát thương nổ lan diện rộng ($AoE \ge 35px$), hủy diệt cụm lính co cụm. |
| **Arcane** | `#A855F7` | **Bỏ qua 50% Kháng Vật Lý** của mục tiêu ($Resist_{effective} = Resist_{phys} \times 0.5$). |

### Hệ Thống 3 Tài Nguyên & Chuyển Hóa
- **Minerals (💎)**: Sản xuất lính cơ bản và xây móng công trình. Engineer khai thác $10$ quặng/chuyến ($15$ quặng/chuyến sau khi nghiên cứu `Optical Drills`).
- **Energy (⚡)**: Sản sinh $+8⚡/\text{s}$ từ `Power Reactor`. Sử dụng cho lính công nghệ cao, tháp pháo và nghiên cứu.
- **Supply (👥)**: Khởi đầu $4/10$. `Supply Depot` cấp $+8$, `HQ` cấp $+10$. Trần tối đa $200$.
- **Energy Converter**: Khi Energy $> 1000⚡$, tự động chuyển hóa $100⚡ \to 50💎/\text{s}$.

---

## 2. Hệ Thống Auto-Play (Loop Cards Engine)

Auto-Play cho phép người chơi hoặc bot vận hành căn cứ hoàn toàn tự động theo chuỗi thẻ lệnh `Điều Kiện (If) -> Hành Động (Then)`.

### Các file liên quan:
- Cấu trúc dữ liệu: `rust-engine/src/autoplay/types.rs`
- Bộ thực thi: `rust-engine/src/autoplay/executor.rs`
- 7 Archetype mẫu: `rust-engine/src/autoplay/archetypes.rs`
- Giao diện người dùng: `src/components/AutoPlayModal.tsx`

### Thêm một Điều kiện AutoCondition mới:
1. Thêm variant vào `AutoCondition` trong `rust-engine/src/autoplay/types.rs`.
2. Viết logic kiểm tra trong `rust-engine/src/autoplay/executor.rs` (`check_condition`).
3. Thêm trường giao diện chọn điều kiện trong `src/components/AutoPlayModal.tsx`.

---

## 3. Huấn Luyện AI Deep RL (`train_bot.sh`)

### Kiến trúc Não bộ Duy nhất (`assets/rl_model.json`)
- **64 Chiều Đầu Vào (`STATE_DIM = 64`)**:
  - Kinh tế (Minerals, Energy, Supply ratio, Tỷ lệ chuyển hóa năng lượng)
  - Số lượng từng loại công trình và từng loại đơn vị
  - Chỉ số chênh lệch sức mạnh quân sự với đối phương
  - Tiến độ cây công nghệ 4 nhánh
  - Kịch bản trận đấu (1v1, 2v2, 3v3, 4v4, 2v2v2, 8-FFA)
- **Tầng Ẩn 768 Neurons**:
  - Hidden Layer 1: 512 neurons (LeakyReLU $\alpha = 0.01$)
  - Hidden Layer 2: 256 neurons (LeakyReLU $\alpha = 0.01$)
- **Đầu Ra Kép (Dual Head)**:
  - Head Action: 15 Q-values cho 15 hành động vĩ mô/vi mô.
  - Head Formation: 6 Q-values cho 6 đội hình tác chiến.
- **Tổng số tham số**: $170,005$ tham số.

### Cú pháp Lệnh Huấn luyện:
```bash
# Huấn luyện thông thường (128 ván, dt=0.25s, max_time=1800s, tiếp tục từ model cũ)
./train_bot.sh

# Huấn luyện từ đầu (Reset weights về ngẫu nhiên He/Xavier)
./train_bot.sh 128 0.25 1800 null false
```

### Quy tắc sau khi Huấn luyện:
Luôn chạy `./build-wasm.sh` để nhúng trọng số mới từ `assets/rl_model.json` vào binary WebAssembly.
