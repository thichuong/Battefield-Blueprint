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
- **Hệ thống Công trình Blueprint (6 Công trình)**:
  - 🏢 **Trung tâm Chỉ huy (Command Center)**: Thu nhận khoáng sản, xin thợ mỏ.
  - 📦 **Trạm Tiếp vận (Supply Depot)**: Tăng giới hạn dân số (+8 Supply).
  - 🔋 **Lò Phản ứng Năng lượng (Power Reactor)**: Tự động gia tăng Năng lượng (Energy/s).
  - ⛺ **Trại Huấn luyện (Barracks)**: Huấn luyện các binh chủng chiến đấu có điểm tập kết (Rally Point).
  - 🗼 **Tháp Pháo Phòng thủ (Turret Tower)**: Tự động bắn hạ kẻ địch xâm phạm trong tầm ngắm.
  - ⚡ **Trạm Chuyển Hóa Năng Lượng (Energy Converter)**: Tự động quy đổi năng lượng dư thừa thành khoáng sản vàng khi duy trì tỷ lệ điện/lò ổn định.
- **Hệ thống Tự Động Hóa Auto-Play & Tuyến Phòng Thủ Tháp Canh (Auto-Play System)**:
  - 📋 **Vòng lặp Thẻ bài (Loop Cards Engine)**: Thiết lập các vòng lặp độc lập (Sản xuất, Quân sự, v.v.) với điều kiện kích hoạt linh hoạt (giới hạn quân số, khoáng sản, năng lượng, trần supply, tỷ lệ lò phản ứng).
  - 🛡️ **Tuyến Phòng Thủ Tháp Canh (Turret Defense Line)**: Kéo thả và tính toán vector tự động bố trí tháp pháo 1 đến 3 hàng chắn giữa các trục xung yếu của căn cứ.
- **Hệ Thống Trí Tuệ Nhân Tạo & Huấn Luyện Tự Động (AI & RL Self-Play Trainer)**:
  - 🧠 **Não bộ RL Q-Learning (`RlBrain`)**: Mạng nơ-ron học sâu trích xuất 24 chiều vector trạng thái chiến trường (kinh tế, supply, số lượng quân, công nghệ, khoảng cách đe dọa) và lựa chọn 11 nhánh hành động chiến lược.
  - ⚔️ **Auto Chuyên Sâu & Tùy Biến Giữa Trận (Adaptive Deep Auto)**:
    - 5 Archetype chiến thuật xoay tua: *Pháo binh Laser & Rocket*, *Tiên kiếm & Vu sư Hư không*, *Bọ nổ & Quái thú Titan*, *Bộ binh Đột kích*, *Đại công sự Kinh tế*.
    - *Tùy biến giữa trận (Mid-Match Adaptation)*: Khi trận đấu diễn ra và công nghệ mới được nghiên cứu xong, Auto tự động phát hiện và bổ sung các thẻ mua binh chủng mới tương ứng, đồng thời chuyển dịch cơ cấu giảm lính cơ bản để dồn tài nguyên cho quân chủ lực.
  - 🤖 **Bot-vs-Bot Self-Play & Champion Pool**:
    - Ngay sau khi Bot thắng được Auto chuyên sâu, hệ thống tự động mở khóa chế độ **Bot tự đấu với nhau (Self-Play)**.
    - Lưu trữ bản sao não bộ vào **Champion Pool** qua các thế hệ đối thủ ngày càng thông minh.
    - **Thi đấu xen kẽ**: Các vòng đấu tự cọ xát Bot-vs-Bot được xen kẽ với các vòng Auto chuyên sâu để bot phát triển toàn diện cả tư duy đối kháng lẫn khả năng macro phòng thủ.
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

5. **Huấn luyện AI Bot (RL Trainer & Self-Play)**:
   Bạn có thể tự chạy trình giả lập huấn luyện siêu tốc (Headless Simulation ~30-70 ep/s) để huấn luyện bộ não AI và cập nhật trọng số mới:
   ```bash
   # Huấn luyện 60 trận với bước nhảy thời gian 0.15s/step
   ./train_bot.sh 60 0.15

   # Hoặc chạy trực tiếp qua cargo binary
   cargo run --release --manifest-path rust-engine/Cargo.toml --bin train_bot -- --episodes 100 --dt 0.15 --output assets/rl_model.json
   ```
   Sau khi huấn luyện xong, chạy `./build-wasm.sh` để biên dịch lại gói WASM và nhúng trọng số mới vào trò chơi.

