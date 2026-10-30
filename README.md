# Battlefield Blueprint (Tactical Stickman RTS)

Game chiến thuật thời gian thực (RTS) trên nền tảng Web với phong cách đồ họa **Người Que trên Bản vẽ Chiến thuật (Blueprint Tactical Neon)**.

Dự án kết hợp kiến trúc hiệu năng cao giữa **Rust WebAssembly (Wasm)** và **React + TypeScript**:
- **Rust Wasm Engine (`rust-engine`)**: Phụ trách toàn bộ vòng lặp mô phỏng game theo thời gian thực (ECS, hệ thống di chuyển, khai thác, xây dựng, chiến đấu, đạn/laser, Sương mù chiến tranh Fog of War, AI Skirmish) và vẽ toàn bộ đồ họa nhân vật, công trình cùng HUD trong trận (Minimap, Thanh tài nguyên, Command card, Chọn quân, Máu) bằng Canvas API.
- **React (`src`)**: Phụ trách các giao diện UI ngoài vòng lặp (Màn hình chính Main Menu, Modal Cây Công Nghệ 3 nhánh, Cài đặt Game, Bảng Thắng/Thua và Bộ tổng hợp âm thanh Web Audio procedural synth).

---

## 🚀 Tính năng nổi bật

- **Kiến trúc Hybrid Wasm + React**:
  - Giao tiếp 2 chiều không độ trễ giữa Rust và React qua `wasm-bindgen` và Event Queue.
  - Rust Wasm vẽ trực tiếp bằng đồ họa vector toán học (procedural geometry/strokes), không cần tải ảnh ngoài, kích thước Wasm siêu nhẹ (~228KB).
- **Hệ thống Đa dạng Loại Sát Thương & Chỉ số Kháng (Combat Matrix)**:
  - 💥 **4 Loại Sát Thương**:
    - **Vật lý (Physical)**: Sát thương đơn mục tiêu tốc độ cao, số nhảy màu trắng bạc.
    - **Năng lượng (Energy)**: Laser / plasma tốc độ cực nhanh, số nhảy xanh Cyan neon.
    - **Nổ (Explosive - AoE)**: Rocket / bọ cảm tử nổ diện rộng bán kính 30-65px, số nhảy cam đỏ rực rỡ.
    - **Ma thuật (Arcane)**: Bỏ qua giáp vật lý thông thường, số nhảy tím ma ảo.
  - 🛡️ **Bảng Chỉ Số Kháng Đa Diện (Resistances)**:
    - Mỗi đơn vị và công trình sở hữu 4 chỉ số kháng riêng: `Kháng Vật lý`, `Kháng Năng lượng`, `Kháng Nổ`, `Kháng Ma thuật`.
    - Công thức giảm trừ sát thương thực tế theo thời gian thực và hiển thị cụ thể trên HUD khi click chọn mục tiêu.
- **Hệ thống Tiến Hóa 3 Nhánh Đại Chủng Loài (Evolution System)**:
  - 🔬 **Khoa học (Science)**:
    - *Công nghệ tương lai*: Tia laser hội tụ quang năng, Khiên lượng tử phản xạ (+40% Kháng Năng lượng).
    - *Hắc khoa kỹ*: Lính bệ phóng Rocket nổ AoE (Dark Rocketeer), Tháp pháo bắn đạn nổ, Khung xương Titan cường lực (+35% Kháng Vật lý).
  - 🔮 **Ma thuật (Magic)**:
    - *Tiên pháp*: Tu tiên giả ngự phi kiếm chân khí Arcane (Taoist Swordmaster), Kim cang hộ thể linh quang (+40% Kháng Phép & hồi 3 HP/s).
    - *Pháp thuật bóng tối*: Hắc ám chú ấn (bào mòn kháng đối phương khi trúng đòn), Vu sư hư không nổ bóng tối AoE (Void Warlock).
  - 🧬 **Quái vật (Monsters)**:
    - *Khổng lồ*: Vảy cổ đại (+50% Kháng Vật lý & +40% Kháng Nổ), Cự thú Titan 650 HP (Behemoth Titan) dậm chấn động diện rộng.
    - *Bọ bầy đàn*: Bọ nổ cảm tử axit AoE cực lớn (Acid Baneling), Sinh sôi siêu tốc (-35% chi phí & thời gian huấn luyện toàn quân, +25% tốc chạy).
- **Hệ thống Đơn vị Người Que (10 Binh chủng)**:
  - 🔧 **Kỹ sư / Thợ mỏ (Engineer)**: Cầm cờ-lê/máy hàn, khai thác khoáng sản và thi công công trình.
  - ⚔️ **Kiếm sĩ Xung kích (Shock Trooper)**: Cầm kiếm Plasma Blade, tốc độ chạy nhanh, cận chiến mạnh.
  - 🎯 **Xạ thủ Súng trường (Rifleman)**: Tấn công tầm xa, thích hợp lập đội hình hỏa lực.
  - 🛡️ **Vệ binh Khiên chắn (Vanguard)**: Khiên năng lượng hấp thụ sát thương lớn cho tuyến sau.
  - ⚡ **Pháo thủ Laser Tầm xa (Laser Marksman)**: Chiếu tia laser xuyên thấu với tầm bắn và sát thương cực cao.
  - 🚀 **Lính Pháo Rocket Hắc Ám (Dark Rocketeer)**: Bệ phóng tên lửa nổ AoE tầm xa.
  - 🗡️ **Tu Tiên Giả Ngự Kiếm (Taoist Swordmaster)**: Phi kiếm chân khí sát thương Ma thuật Arcane.
  - 💀 **Vu Sư Hư Không (Void Warlock)**: Trượng bóng tối bắn cầu ma thuật bộc phá AoE.
  - 🦣 **Chiến Thú Khổng Lồ (Behemoth Titan)**: Stickman 650 HP to lớn, dậm chấn động diện rộng.
  - 🦗 **Bọ Nổ Cảm Tử (Acid Baneling)**: Bò 4 chân tốc độ cao, lao vào tự kích nổ axit AoE cực mạnh.
- **Hệ thống Công trình Blueprint (5 Công trình)**:
  - 🏢 **Trung tâm Chỉ huy (Command Center)**: Thu nhận khoáng sản, xin thợ mỏ.
  - 📦 **Trạm Tiếp vận (Supply Depot)**: Tăng giới hạn dân số (+8 Supply).
  - 🔋 **Lò Phản ứng Năng lượng (Power Reactor)**: Tự động gia tăng Năng lượng (Energy/s).
  - ⛺ **Trại Huấn luyện (Barracks)**: Huấn luyện các binh chủng chiến đấu có điểm tập kết (Rally Point).
  - 🗼 **Tháp Pháo Phòng thủ (Turret Tower)**: Tự động bắn hạ kẻ địch xâm phạm trong tầm ngắm.
- **Cây Công Nghệ 3 Nhánh (Tech Tree Modal)**:
  - *Kinh tế & Công trình*: Mũi khoan quang học, Lò phản ứng quá tải, Bản vẽ gia cố kiến trúc.
  - *Bộ binh & Vũ trang*: Khiên năng lượng, Đạn gia tốc từ trường, Bộ phản lực chân Exo-Boost.
  - *Công nghệ cao & Laser*: Tia laser hội tụ, Lưỡi kiếm plasma quá nhiệt, Tụ điện quá tải hủy diệt.
- **Chiến trường & AI Đối thủ**:
  - Bản đồ chiến thuật `2400x2400` với các vách tường chokepoint, mỏ tài nguyên trung tâm.
  - Sương mù chiến tranh (Fog of War) 3 trạng thái: Ẩn (Hidden), Đã khám phá (Explored), Trong tầm nhìn (Visible).
  - AI Skirmish tự động phát triển căn cứ, nâng cấp kinh tế và tổ chức các đợt càn quét người chơi.
- **Web Audio Procedural Synth**:
  - Toàn bộ âm thanh (tiếng bíp lệnh, chém kiếm, đạn súng, laser, nghiên cứu, thắng/thua) được tổng hợp bằng code Web Audio API, không cần file mp3/wav ngoài.

---

## 🎮 Hướng dẫn Điều khiển & Phím tắt

| Thao tác | Phím tắt / Chuột | Tác dụng |
| :--- | :--- | :--- |
| **Di chuyển Camera** | `W`, `A`, `S`, `D` hoặc `Mũi tên` | Di chuyển góc nhìn chiến trường |
| **Phóng to / Thu nhỏ** | `Cuộn chuột (Wheel)` | Zoom in / Zoom out từ 0.5x đến 2.0x |
| **Quét chọn quân** | `Kéo chuột trái` | Quét hộp chọn nhiều đơn vị lính |
| **Chọn đơn vị / Nhà** | `Click chuột trái` | Chọn một lính hoặc một công trình |
| **Lệnh hành động** | `Click chuột phải` | Di chuyển đội hình / Khai thác mỏ / Tấn công mục tiêu / Đặt cờ tập kết |
| **Cây Công Nghệ** | Phím `T` hoặc Nút trên HUD | Mở bảng sơ đồ Cây Công Nghệ 3 nhánh |
| **Cài đặt & Tạm dừng** | Phím `ESC` hoặc Nút trên HUD | Mở bảng Cài đặt (Âm lượng, Tốc độ camera, Auto-pause) |
| **Tạm dừng nhanh** | Phím `Space` | Tạm dừng / Tiếp tục trận đấu |

---

## 🛠️ Cài đặt & Khởi chạy dự án

### Yêu cầu môi trường
- [Rust](https://www.rust-lang.org/) (hỗ trợ target `wasm32-unknown-unknown`)
- `wasm-bindgen-cli` (`cargo install wasm-bindgen-cli`)
- [Node.js](https://nodejs.org/) (phiên bản 18+ trở lên) & `npm`

### Các bước cài đặt

1. **Cài đặt dependencies Node.js**:
   ```bash
   npm install
   ```

2. **Biên dịch Rust sang WebAssembly**:
   ```bash
   ./build-wasm.sh
   ```
   *(Hoặc chạy lệnh `npm run build:wasm`)*

3. **Chạy máy chủ phát triển (Dev Server)**:
   ```bash
   npm run dev
   ```
   Mở trình duyệt tại: `http://localhost:3000`

4. **Đóng gói dự án cho Production**:
   ```bash
   npm run build
   ```

