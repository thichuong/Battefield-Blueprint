# AGENT RULES // BATTLEFIELD BLUEPRINT
### Quy tắc & Kỷ luật Kỹ thuật Tuyệt đối Dành cho AI Agent

---

## MỤC LỤC
1. [Nguyên Tắc Kiến Trúc Cốt Lõi (Architecture Invariants)](#1-nguyên-tắc-kiến-trúc-cốt-lõi)
2. [Kỷ Luật Hiệu Năng & Quản Lý Bộ Nhớ (Zero-Allocation Rules)](#2-kỷ-luật-hiệu-năng--quản-lý-bộ-nhớ)
3. [Quy Chuẩn Tài Nguyên & Đồ Họa / Âm Thanh Thủ Tục (Procedural Assets)](#3-quy-chuẩn-tài-nguyên--đồ-họa--âm-thanh-thủ-tục)
4. [Quy Tắc Biên Dịch & Đồng Bộ WASM Bridge](#4-quy-tắc-biên-dịch--đồng-bộ-wasm-bridge)
5. [Ma Trận Cân Bằng Chiến Đấu & Kinh Tế (Combat & Economy Matrix)](#5-ma-trận-cân-bằng-chiến-đấu--kinh-tế)
6. [Hệ Thống Auto-Play & Deep Reinforcement Learning (RL Rules)](#6-hệ-thống-auto-play--deep-rl)
7. [Quy Chuẩn Giao Diện React 19 & Tactical Blueprint UI](#7-quy-chuẩn-giao-diện-react-19--tactical-blueprint-ui)
8. [Danh Sách Các Lỗi Phổ Biến & Điều Cấm Kỵ (Anti-Patterns)](#8-danh-sách-điều-cấm-kỵ)

---

## 1. NGUYÊN TẮC KIẾN TRÚC CỐT LÕI

### 1.1. Single Source of Truth
- Mọi logic trò chơi (vị trí tọa độ, di chuyển, máu, giáp, sát thương, tài nguyên, hàng đợi sản xuất, nghiên cứu công nghệ, đạn đạo, sương mù chiến tranh) **CHỈ ĐƯỢC PHÉP TỒN TẠI VÀ TÍNH TOÁN TRONG RUST ENGINE (`rust-engine/`)**.
- Phía React 19 và TypeScript (`src/`) tuyệt đối **KHÔNG** tự ý giả lập hoặc suy đoán trạng thái trận đấu. React đóng vai trò là tầng Hiển thị (Presentation Layer) và Bảng điều khiển (Control Layer).

### 1.2. Giao Thức Giao Tiếp 2 Chiều (Bridge Protocol)
- **Từ React sang Rust (Downlink)**: Sử dụng duy nhất hàm `wasmBridge.sendCommand(cmd)`. Lệnh gửi đi là một đối tượng JSON có thuộc tính `type` định danh hành động (ví dụ: `TrainUnit`, `BuildStructure`, `SetFormation`, `StartResearch`, `AttackMove`).
- **Từ Rust sang React (Uplink)**: Rust Engine đưa sự kiện vào hàng đợi nội bộ (`event_queue`). Mỗi frame, hàm `poll_events_from_engine()` sẽ trích xuất danh sách sự kiện dưới dạng mảng JSON (ví dụ: `PlaySound`, `UnitTrained`, `BuildingCompleted`, `GameOver`).
- **Không can thiệp trực tiếp vào con trỏ bộ nhớ (Direct Memory Pointer)** giữa JS và WASM trừ các API chuẩn đã được export qua `wasm_bindgen`.

---

## 2. KỶ LUẬT HIỆU NĂNG & QUẢN LÝ BỘ NHỚ

Trò chơi hướng tới tốc độ khung hình **120-200+ FPS** trên bản đồ rộng $4800 \times 4800$ với hàng trăm đơn vị tham chiến đồng thời.

### 2.1. Không Cấp Phát Vùng Nhớ Trong Vòng Lặp Tick (Zero Allocation in Simulation Loop)
- **CẤM**: Gọi `Vec::new()`, `format!()`, `to_string()`, `Box::new()` hoặc các cấu trúc dữ liệu tự cấp phát heap bên trong hàm `engine_step(dt)` hoặc trong các vòng lặp duyệt từng unit/projectile mỗi tick.
- **KHUYẾN KHÍCH**: Sử dụng mảng cố định (`[T; N]`), mảng đệm tái sử dụng (`reusable buffer`), cấu trúc bitmask, hoặc truyền tham chiếu `&mut Vec<T>` đã được `clear()` sẵn trước đó.

### 2.2. Truy Vấn Không Gian Tuyệt Đối Qua Spatial Grid $O(1)$
- Kích thước ô lưới không gian là $150 \times 150$ (`CELL_SIZE = 150.0`), tương ứng lưới $33 \times 33$ ô trên toàn bản đồ.
- Khi tìm kiếm mục tiêu tấn công, va chạm đơn vị, hoặc bán kính sát thương diện rộng AoE: **BẮT BUỘC** gọi qua `SpatialGrid::query_radius()` hoặc `SpatialGrid::query_box()`.
- **TUYỆT ĐỐI CẤM**: Lặp lồng $O(N^2)$ duyệt qua toàn bộ mảng `units` của các người chơi để tìm mục tiêu gần nhất.

### 2.3. Tránh Rác Bộ Nhớ (Garbage Collection Spikes) Ở Phía JavaScript
- Tránh tạo các closure mới hoặc Object literal mới bên trong hàm `requestAnimationFrame` của React.
- Toàn bộ thao tác chuột/phím tắt (`mousemove`, `keydown`) chuyển thẳng tọa độ và mã phím dạng primitive number/string sang WASM qua `wasmBridge.onMouseMove(x, y)` mà không tạo payload trung gian không cần thiết.

---

## 3. QUY CHUẨN TÀI NGUYÊN & ĐỒ HỌA / ÂM THANH THỦ TỤC

### 3.1. 100% Đồ Họa Vector Toán Học (Procedural Geometry)
- Bản sắc của Battlefield Blueprint là phong cách **Bản Vẽ Kỹ Thuật Neon (Blueprint Tactical Vector)**.
- Mọi đơn vị Người Que (Stickman) được kết xuất bằng các lệnh vẽ vector hình học: đầu tròn (arc/circle), thân và chi (stroke line với góc xoay chuyển động run-cycle theo hàm sin/cos của vận tốc), vũ khí (plasma blade, laser pointer, bệ phóng tên lửa).
- **CẤM**: Thêm bất kỳ file ảnh bitmap tĩnh nào (`.png`, `.jpg`, `.jpeg`, `.webp`, `.svg`, `.gif`) vào thư mục dự án để làm sprite.

### 3.2. 100% Tổng Hợp Âm Thanh Thủ Tục (Web Audio API Procedural Synth)
- Mọi âm thanh trong game (tiếng bắn Laser, tiếng đạn nổ AoE, tiếng chém Plasma, tiếng còi hú căn cứ bị tấn công, tiếng hoàn thành công trình) được tổng hợp bằng hàm toán học từ các linh kiện cơ bản của Web Audio API: `OscillatorNode` (sine, square, sawtooth, triangle), `BiquadFilterNode`, `GainNode` và `AudioBufferSourceNode` (White Noise buffer).
- **CẤM**: Thêm bất kỳ file âm thanh nào (`.mp3`, `.wav`, `.ogg`, `.flac`) vào kho lưu trữ.
- Khi thêm hiệu ứng âm thanh mới: Bổ sung phương thức synth trong `src/audio/soundEngine.ts` và kích hoạt qua event `PlaySound` từ Rust Engine.

---

## 4. QUY TẮC BIÊN DỊCH & ĐỒNG BỘ WASM BRIDGE

### 4.1. Quy Trình 3 Bước Khi Sửa Mã Rust
Khi thực hiện bất kỳ thay đổi nào trong `rust-engine/src/`:
1. **Kiểm tra Unit Test Rust**:
   ```bash
   cargo test -p rust-engine
   ```
2. **Biên dịch sang WebAssembly**:
   ```bash
   ./build-wasm.sh
   # hoặc: npm run build:wasm
   ```
3. **Đồng bộ hóa Type Definitions trong TypeScript**:
   - Nếu thay đổi tham số trong `rust-engine/src/config/`: Cập nhật interface tương ứng trong `src/bridge/wasmBridge.ts`.
   - Nếu thêm loại lệnh mới trong `bridge.rs`: Cập nhật kiểu `EngineCommand` trong TypeScript.
   - Nếu thêm sự kiện mới: Cập nhật hàm xử lý `pollEvents()` trong `src/bridge/wasmBridge.ts`.

### 4.2. Bảo Vệ File Gốc Của WASM
- Thư mục `src/wasm-pkg/` là sản phẩm đầu ra của `wasm-bindgen`. **KHÔNG** chỉnh sửa thủ công các file trong `src/wasm-pkg/` vì sẽ bị ghi đè hoàn toàn khi chạy `./build-wasm.sh`.

---

## 5. MA TRẬN CÂN BẰNG CHIẾN ĐẤU & KINH TẾ

Mọi điều chỉnh balance hoặc thêm đơn vị mới phải tuân thủ nghiêm ngặt ma trận 4 hệ sát thương và 3 tài nguyên:

### 5.1. Bảng Khắc Chế 4 Hệ Sát Thương
| Hệ Sát Thương | Màu Neon | Đặc Điểm Cốt Lõi | Mục Tiêu Khắc Chế |
| :--- | :--- | :--- | :--- |
| **Physical (Vật lý)** | Trắng `#FFFFFF` / Vàng `#FACC15` | Sát thương thuần, bị trừ thẳng theo % Kháng Vật Lý | Quân giáp nhẹ, mục tiêu tầm xa |
| **Energy (Năng lượng)** | Cyan Neon `#06B6D4` | Bỏ qua giáp vật lý nặng; tốc độ tia cực nhanh | Các đơn vị Tanker bọc thép dày |
| **Explosive (Nổ AoE)** | Cam Đỏ `#F97316` | Gây sát thương lan diện rộng bán kính $35px - 75px$ | Cụm bộ binh đông, co cụm đội hình |
| **Arcane (Ma thuật)** | Tím `#A855F7` | **Bỏ qua 50% Kháng Vật Lý** của mục tiêu | Đơn vị siêu giáp, công trình kiên cố |

### 5.2. Cấu Trúc Kinh Tế & Chuyển Đổi Vĩ Mô
- Mỏ Khoáng Sản: Chứa trữ lượng $5,000 - 15,000$ khoáng sản; `Engineer` khai thác $10$ quặng/lần ($15$ quặng/lần khi có nâng cấp `Optical Drills`).
- Năng Lượng (`Energy`): Sản sinh liên tục $+8⚡/\text{giây}$ từ mỗi `Power Reactor`.
- Dân Số (`Supply`): `HQ` cung cấp $+10$, `Supply Depot` cung cấp $+8$. Trần tối đa mặc định $200$.
- Trạm Chuyển Hóa (`Energy Converter`): Khi dự trữ $Energy \ge 1000⚡$, mỗi trạm tự động tiêu thụ $100⚡$ để hoàn lại $50💎$ khoáng sản mỗi giây.

---

## 6. HỆ THỐNG AUTO-PLAY & DEEP RL

### 6.1. Nguyên Tắc Của Hệ Thống Auto-Play
- Hệ thống Auto-Play sử dụng mô hình **Loop Cards Engine** (`rust-engine/src/autoplay/`).
- Mỗi thẻ lệnh (`AutoCard`) bao gồm:
  - Danh sách điều kiện `AutoCondition` (ví dụ: `SupplyNearCap`, `MineralsGreaterThan`, `UnitCountLessThan`).
  - Một hành động `AutoAction` (ví dụ: `TrainUnit`, `BuildStructure`, `ResearchTech`, `AttackWave`).
- 7 Archetype Auto mẫu cố định:
  1. *Science Laser & Rocket Siege*
  2. *Magic Sword & Void Cataclysm*
  3. *Monster Swarm & Behemoth Titan*
  4. *Bio Infantry Blitz Rush*
  5. *Macro Eco & Multi-row Fortress*
  6. *Combined Titan, Artillery & Void Mage*
  7. *Omni-Tech Grandmaster*

### 6.2. Kiến Trúc Mạng Deep RL Duy Nhất (`assets/rl_model.json`)
- **Kiến trúc**: Deep Dual-Head MLP.
- **Vector Đầu Vào**: 64 chiều chuẩn hóa trong khoảng $[-1.0, 1.0]$.
- **Tầng Ẩn**: 2 tầng ẩn đầy đủ (Hidden Layer 1: 512 units, Hidden Layer 2: 256 units, kích hoạt LeakyReLU $\alpha = 0.01$).
- **Đầu Ra Kép (Dual Head)**:
  - Head 1: 15 Q-values cho 15 hành động chiến lược.
  - Head 2: 6 Q-values cho 6 đội hình tác chiến (`Line`, `Wedge`, `Box`, `Scatter`, `Circle`, `Column`).
- **Tổng Tham Số**: Đúng $170,005$ tham số.
- Khi huấn luyện bằng `train_bot.sh`, file `assets/rl_model.json` và `rust-engine/assets/rl_model.json` luôn được đồng bộ cùng lúc.

---

## 7. QUY CHUẨN GIAO DIỆN REACT 19 & TACTICAL BLUEPRINT UI

### 7.1. Bảng Màu & Thẩm Mỹ (Color Palette & Aesthetics)
- Nền chính: `#030712` (Black / Slate-950) kết hợp lưới tọa độ blueprint mờ.
- Viền & Khung: Border mỏng bán trong suốt với hiệu ứng phát sáng nhẹ (`border-cyan-500/30`, `hover:border-cyan-400`, `shadow-[0_0_15px_rgba(6,182,212,0.15)]`).
- Font chữ: Monospace (`font-mono`) cho toàn bộ nhãn số liệu, thông số DPS, tài nguyên, phím tắt.
- Chữ hoa kỹ thuật: Toàn bộ tiêu đề nút bấm và bảng chức năng viết hoa kỹ thuật số (`ATTACK-MOVE`, `FORMATION: WEDGE`, `AUTO-PLAY`).

### 7.2. Xử Lý Modal & Tự Động Tạm Dừng (Modal Auto-Pause)
- Trò chơi hỗ trợ cơ chế tự động dừng mô phỏng khi mở bảng cài đặt, cây công nghệ, hướng dẫn hoặc bảng lập trình Auto-Play.
- Khi mở modal: Đọc cờ `autoPauseOnModal` từ `settings`. Nếu `true`, gửi lệnh `{ type: 'Pause' }` sang WASM.
- Khi đóng modal: Gửi lệnh `{ type: 'Resume' }` sang WASM.

---

## 8. DANH SÁCH ĐIỀU CẤM KỴ (ANTI-PATTERNS)

❌ **1. KHÔNG thêm file ảnh hoặc âm thanh bên ngoài** vào dự án. Mọi hình ảnh phải vẽ vector, mọi âm thanh phải tạo từ Web Audio API.  
❌ **2. KHÔNG tính toán logic gameplay bên trong React component**. React chỉ gửi command và hiển thị event từ Wasm.  
❌ **3. KHÔNG viết vòng lặp $O(N^2)$** duyệt đơn vị trên bản đồ; luôn sử dụng `SpatialGrid`.  
❌ **4. KHÔNG cấp phát bộ nhớ động trong vòng lặp frame** của Rust Engine (`format!`, `Vec::new`).  
❌ **5. KHÔNG sửa trực tiếp các file trong `src/wasm-pkg/`**; luôn sửa mã Rust rồi chạy `./build-wasm.sh`.  
❌ **6. KHÔNG làm gãy cấu trúc vector 64 chiều** của mạng Neural RL nếu không cập nhật đồng bộ các file huấn luyện CUDA và Rust.  
❌ **7. KHÔNG xóa bỏ hoặc làm sai lệch các phím tắt bàn phím** đã quy ước trong trò chơi.
