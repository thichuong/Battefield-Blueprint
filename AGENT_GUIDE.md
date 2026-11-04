# HƯỚNG DẪN AGENT HOẠT ĐỘNG (AGENT OPERATIONAL GUIDE)
## Battlefield Blueprint — Tactical Stickman RTS

> **Dành cho:** AI Coding Agents tham gia phát triển, bảo trì và mở rộng dự án.  
> **Mục tiêu:** Cung cấp quy trình từng bước (Standard Operating Procedures - SOP), cây quyết định và các tiêu chuẩn kiểm thử giúp agent thực thi chuẩn xác ngay trong lượt đầu tiên (Zero-shot Accuracy).

---

## 🧭 PHẦN 1: CÂY QUYẾT ĐỊNH & VÒNG ĐỜI XỬ LÝ NHIỆM VỤ

Mỗi khi nhận yêu cầu từ người dùng, Agent cần phân loại nhiệm vụ theo ma trận sau:

```
                          [YÊU CẦU CỦA NGƯỜI DÙNG]
                                     │
         ┌───────────────────────────┼───────────────────────────┐
         ▼                           ▼                           ▼
[NHÓM I: SIMULATION & ENGINE]  [NHÓM II: GRAPHICS/AUDIO]  [NHÓM III: UI & MODALS]
- Thêm/Sửa Unit, Building      - Vẽ hiệu ứng đạn/laser    - Tạo modal mới
- Cân bằng Combat, Kinh tế     - Web Audio Synth FX       - Thêm phím tắt
- Cây Công Nghệ                - Shaders WebGPU/Canvas    - Bảng điều khiển HUD
- Thẻ lệnh Auto-Play           - Tối ưu Render loop       - Menu, Settings
         │                           │                           │
         ▼                           ▼                           ▼
 Sửa Rust Engine               Sửa Rust Renderer           Sửa React 19
(`rust-engine/src/`)          hoặc `soundEngine.ts`       (`src/components/`)
         │                           │                           │
         ▼                           │                           │
 Chạy Unit Test                      │                           │
`cargo test -p rust-engine`          │                           │
         │                           │                           │
         ▼                           │                           │
 Rebuild WebAssembly                 │                           │
`./build-wasm.sh`                    │                           │
         │                           │                           │
         └───────────────────────────┼───────────────────────────┘
                                     │
                                     ▼
                    [ĐỒNG BỘ TS & KIỂM TRA LINT]
                    `src/bridge/wasmBridge.ts` (nếu có đổi API)
                    `npm run lint` (`tsc -b --noEmit`)
                                     │
                                     ▼
                        [XÁC MINH TOÀN DIỆN]
                        `compile_applet` / `npm run build`
```

---

## 📋 PHẦN 2: CÁC QUY TRÌNH VẬN HÀNH TIÊU CHUẨN (7 SOPS)

---

### 🔹 SOP 1: Thêm Mới Hoặc Chỉnh Sửa Binh Chủng (Unit) & Công Trình (Building)

**Bước 1: Khai báo Cấu hình trong Rust**
- Mở `rust-engine/src/config/types.rs`:
  - Thêm tên định danh vào enum `UnitType` hoặc `BuildingType`.
- Mở `rust-engine/src/config/defaults.rs`:
  - Khai báo chỉ số mặc định: Máu (`hp`), Tốc độ (`speed`), Tầm bắn (`attack_range`), Sát thương (`attack_damage`), Kiểu sát thương (`damage_type`), Bán kính AoE (`aoe_radius`), 4 chỉ số kháng (`resistances`), Chi phí (`cost_minerals`, `cost_energy`), Thời gian sản xuất (`training_time`).

**Bước 2: Triển khai Vẽ Vector Procedural trong Rust Renderer**
- Mở `rust-engine/src/renderer/units.rs` (đối với lính) hoặc `rust-engine/src/renderer/buildings.rs` (đối với nhà):
  - Định nghĩa hàm vẽ vector: vẽ đầu tròn, vẽ thân/chi chuyển động theo chu kỳ bước chạy (sine function của `move_timer`), và vũ khí đặc trưng.
  - Sử dụng màu Neon nhận diện theo phe hoặc theo thuộc tính binh chủng.

**Bước 3: Cập nhật Cầu nối WASM & TypeScript**
- Mở `src/bridge/wasmBridge.ts`:
  - Bổ sung tên đơn vị mới vào kiểu `UnitStatsConfig` hoặc danh sách khởi tạo.
- Cập nhật các Component React liên quan:
  - `src/components/TechTreeModal.tsx`: Thêm thẻ hiển thị đơn vị trong nhánh công nghệ mở khóa.
  - Command Card: Thêm nút huấn luyện/xây dựng tương ứng.

**Bước 4: Kiểm thử và Biên dịch**
```bash
cargo test -p rust-engine
./build-wasm.sh
npm run lint
```

---

### 🔹 SOP 2: Cân Bằng Chỉ Số Chiến Đấu & Kinh Tế (Combat & Economy)

**Vị trí file tập trung:**
- Hằng số vĩ mô: `rust-engine/src/config/constants.rs` (ví dụ: `ENERGY_CONVERSION_THRESHOLD`, `BUILD_SPEED_WORKER`, `CELL_SIZE`).
- Chỉ số chi tiết từng unit/nhà: `rust-engine/src/config/defaults.rs`.
- Công thức tính sát thương thực nhận: `rust-engine/src/simulation/combat.rs`.
- Cơ chế thu thập mỏ và chuyển hóa năng lượng: `rust-engine/src/simulation/economy.rs`.

**Quy tắc cân bằng bắt buộc:**
1. **Kiểm tra Ma trận 4 Hệ Sát thương**:
   - Khi tăng sát thương của một đơn vị hệ `Arcane`, phải nhớ rằng đơn vị đó đã có nội tại bỏ qua 50% Kháng Vật Lý.
   - Sát thương `Explosive` luôn đi kèm bán kính `aoe_radius > 0` và có thể gây sát thương lên nhiều mục tiêu cùng lúc.
2. **Kinh tế 3 Tài nguyên**:
   - `Minerals` điều tiết số lượng quân cơ bản.
   - `Energy` là nút thắt công nghệ cao và phòng thủ tháp. Tỷ lệ chuyển hóa chuẩn là $100⚡ \to 50💎$. Không được phá vỡ ngưỡng kích hoạt $1000⚡$.
3. **Sau khi chỉnh sửa:**
   - Biên dịch lại WASM qua `./build-wasm.sh` để các chỉ số mới có hiệu lực ngay trong game.

---

### 🔹 SOP 3: Mở Rộng Hoặc Tinh Chỉnh Cây Công Nghệ (Tech Tree)

**Bước 1: Khai báo Công nghệ trong Rust**
- File `rust-engine/src/config/defaults.rs` (hoặc `tech.rs`):
  - Khai báo ID công nghệ, tên hiển thị, nhánh (`Science`, `Magic`, `Monster`, `Logistics`), chi phí khoáng sản/năng lượng, thời gian nghiên cứu và danh sách công nghệ điều kiện tiên quyết (prerequisites).

**Bước 2: Áp dụng Buff / Hiệu ứng trong Rust Simulation**
- Mở `rust-engine/src/simulation/economy.rs` hoặc `combat.rs`:
  - Kiểm tra cờ công nghệ khi tính toán chỉ số (ví dụ: tăng tầm bắn, tăng tốc độ di chuyển, tăng hồi máu, hoặc mở khóa nút sản xuất đơn vị mới).

**Bước 3: Đồng bộ Giao diện React TechTreeModal**
- Mở `src/components/TechTreeModal.tsx`:
  - Thêm thẻ hiển thị công nghệ, icon, mô tả chi tiết và cơ chế gắn sao `[★]` để ưu tiên cho hệ thống Auto-Play.

**Bước 4: Biên dịch lại WASM**:
```bash
./build-wasm.sh
```

---

### 🔹 SOP 4: Bổ Sung Điều Kiện & Thẻ Lệnh Auto-Play (Loop Cards Engine)

**Bước 1: Khai báo Kiểu Điều kiện / Hành động trong Rust**
- Mở `rust-engine/src/autoplay/types.rs`:
  - Thêm biến thể mới vào enum `AutoCondition` (ví dụ: `EnemyNearbyCountGreaterThan { count: usize }`).
  - Thêm biến thể mới vào enum `AutoAction` (ví dụ: `RetreatToBase`).

**Bước 2: Triển khai Logic Đánh giá và Thực thi**
- Mở `rust-engine/src/autoplay/executor.rs`:
  - Bổ sung hàm kiểm tra điều kiện trong `check_condition()`.
  - Bổ sung hành vi thực thi trong `execute_card_action()`.

**Bước 3: Cập nhật Giao diện AutoPlayModal**
- Mở `src/components/AutoPlayModal.tsx`:
  - Thêm form nhập thông số điều kiện/hành động mới để người chơi có thể tự do lập trình thẻ lệnh từ giao diện web.

**Bước 4: Kiểm thử Unit Test Autoplay & Rebuild WASM**:
```bash
cargo test -p rust-engine -- autoplay
./build-wasm.sh
```

---

### 🔹 SOP 5: Huấn Luyện & Cập Nhật Deep RL Brain (`train_bot.sh`)

Mô hình trí tuệ nhân tạo của trò chơi là một mạng nơ-ron hợp nhất duy nhất (`Deep Dual-Head MLP`) lưu tại `assets/rl_model.json`.

**Quy trình thực hiện:**
1. **Kiểm tra Kiến trúc Mạng Nơ-ron**:
   - Nếu thay đổi vector đặc trưng 64 chiều (`STATE_DIM = 64`): Phải cập nhật đồng thời trong:
     - `rust-engine/src/ai/state.rs` (hàm trích xuất trạng thái).
     - `rust-engine/src/rl.rs` (khởi tạo và forward pass).
     - `rust-engine/src/cuda/rl_kernel.cu` (nhân CUDA C++).
     - `rust-engine/src/bin/train_bot.rs` (vòng lặp huấn luyện).
2. **Khởi chạy Huấn luyện Headless**:
   ```bash
   # Tiếp tục huấn luyện tích lũy từ trọng số hiện tại (128 ván):
   ./train_bot.sh 128 0.25 1800 null true

   # Hoặc huấn luyện lại từ đầu nếu cấu trúc nơ-ron thay đổi:
   ./train_bot.sh 128 0.25 1800 null false
   ```
3. **Đồng bộ Mô hình vào Game**:
   - Sau khi script chạy xong, trọng số mới được ghi vào `assets/rl_model.json` và `rust-engine/assets/rl_model.json`.
   - Chạy `./build-wasm.sh` để nhúng file json này trực tiếp vào binary WASM thông qua macro `include_str!`.

---

### 🔹 SOP 6: Thêm Hiệu Ứng Đồ Họa Vector & Web Audio Synth Presets

**Hiệu ứng Đồ Họa Procedural (Vector FX):**
- Thêm hiệu ứng hạt, tia laser, sóng chấn động dậm đất vào `rust-engine/src/renderer/effects.rs`.
- Đối với đường ống WebGPU: Cập nhật shader tương ứng trong `rust-engine/src/renderer/gpu/shaders.wgsl`.
- Nhớ: Toàn bộ hiệu ứng được sinh ra theo công thức toán học (quỹ đạo parabol, suy giảm hàm mũ theo thời gian sống `lifetime`), không dùng texture ảnh.

**Tổng hợp Âm Thanh Thủ Tục (Procedural Web Audio API):**
- Mở `src/audio/soundEngine.ts`.
- Tạo hàm phát âm thanh mới sử dụng Web Audio API (ví dụ: `playShockwaveSound()`, `playLaserContinuous()`):
  - Khởi tạo AudioContext.
  - Cấu hình Oscillator (tần số, loại sóng: sine, triangle, sawtooth).
  - Cấu hình Envelope qua `gainNode.gain.exponentialRampToValueAtTime`.
- Trong Rust Engine (`rust-engine/src/simulation/`): Phát sự kiện `Event::PlaySound { sound: "laser".into() }` khi xảy ra tương tác.

---

### 🔹 SOP 7: Thêm Tính Năng Giao Diện React 19, Modal Hoặc Phím Tắt

**Bước 1: Tạo Component Mới**
- Đặt file trong `src/components/` (ví dụ: `CustomFeatureModal.tsx`).
- Áp dụng các class Tailwind mang đậm phong cách Neon Blueprint:
  ```tsx
  <div className="bg-slate-950/90 border border-cyan-500/40 backdrop-blur-md rounded-lg p-6 shadow-[0_0_20px_rgba(6,182,212,0.2)]">
  ```
- Sử dụng icon từ thư viện `lucide-react`.

**Bước 2: Xử Lý Phím Tắt & Tự Động Tạm Dừng**
- Mở `src/App.tsx`:
  - Đăng ký phím tắt trong sự kiện `window.addEventListener('keydown', ...)`.
  - Đảm bảo phím tắt không kích hoạt khi người chơi đang gõ trong ô `<input>` hoặc `<textarea>`.
  - Đồng bộ trạng thái tạm dừng: Khi modal mở, nếu setting `autoPauseOnModal` bật, gọi `wasmBridge.sendCommand({ type: 'Pause' })`. Khi đóng modal, gửi `{ type: 'Resume' }`.

---

## 🔍 PHẦN 3: BẢNG KIỂM TRA ĐỘ HOÀN THIỆN (VERIFICATION CHECKLIST)

Trước khi bàn giao kết quả hoặc hoàn tất lượt làm việc, Agent PHẢI tự kiểm tra lại theo danh sách sau:

- [ ] **Rust Tests**: Chạy `cargo test -p rust-engine` không có bất kỳ test nào thất bại.
- [ ] **WASM Rebuild**: Đã chạy `./build-wasm.sh` thành công nếu có chỉnh sửa mã Rust.
- [ ] **TypeScript Types**: Các schema trong `src/bridge/wasmBridge.ts` hoàn toàn khớp với Rust `bridge.rs`.
- [ ] **Lint Check**: Chạy `npm run lint` (`tsc -b --noEmit`) đạt kết quả 0 lỗi (Zero Errors).
- [ ] **Production Build**: Chạy `npm run build` tạo bundle thành công không gặp cảnh báo nghiêm trọng.
- [ ] **Không Có File Asset Tĩnh Mới**: Kiểm tra `git status` đảm bảo không có file png/mp3 nào được thêm mới.
- [ ] **Bảo Toàn Hiệu Năng**: Không đưa các thao tác cấp phát bộ nhớ động vào vòng lặp render/tick.
- [ ] **Phím Tắt Toàn Cục**: Tất cả phím tắt mặc định (`Space`, `A`, `F`, `T`, `B`, `H`, `ESC`, `F3`/`~`) tiếp tục hoạt động mượt mà.
