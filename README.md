# Battlefield Blueprint — Tactical Stickman RTS

**Battlefield Blueprint** là tựa game Chiến thuật Thời gian thực (RTS) tốc độ cao trên nền tảng Web, mang phong cách đồ họa **Người Que trên Bản vẽ Kỹ thuật Số (Blueprint Tactical Neon)**.

Dự án kết hợp kiến trúc lai hiệu năng cao giữa **Rust + WebAssembly (`wgpu` WebGPU/WebGL2)** và **React 19 + TypeScript**, đi kèm hệ thống **AI Học Tăng Cường Sâu (Deep Reinforcement Learning)** được huấn luyện tăng tốc trực tiếp trên **GPU NVIDIA CUDA**.

---

## ✨ Điểm nhấn kiến trúc & Công nghệ

- **Kiến trúc Hybrid Rust Wasm + React 19**:
  - **Rust Engine (`rust-engine`)**: Đảm nhiệm toàn bộ vòng lặp mô phỏng RTS thời gian thực (ECS, Spatial Grid truy vấn không gian $O(1)$, tìm đường, khai thác, xây dựng, chiến đấu, đạn đạo, Sương mù chiến tranh Fog of War, 6 đội hình chiến thuật, Auto-Play Executor và mạng nơ-ron RL AI).
  - **Multi-Canvas Layering & GPU Pipeline (`wgpu`)**: Kết xuất chiến trường qua đường ống đồ họa **WebGPU** (tự động fallback sang **WebGL2**) kết hợp các lớp Canvas 2D tách biệt cho HUD (Minimap, Selection, Command Card có cơ chế tự động ẩn/hiện thông minh), mở khóa trần tốc độ khung hình **200+ FPS**.
  - **Đồ họa & Âm thanh Procedural 100%**: Toàn bộ nhân vật, công trình, hiệu ứng hạt/laser được vẽ bằng hình học vector toán học và toàn bộ âm thanh chiến trường được tổng hợp trực tiếp qua **Web Audio API Procedural Synth** — không phụ thuộc vào file ảnh hay file âm thanh bên ngoài.
- **Chiến trường Đa phe 8 Người chơi (`4800 x 4800`)**:
  - Hỗ trợ tối đa **8 vị trí xuất phát (8 Slots)** với bảng màu Neon nhận diện riêng biệt đồng bộ giữa chiến trường và Minimap.
  - 6 kịch bản thiết lập nhanh (**1v1**, **2v2**, **3v3**, **4v4**, **2v2v2**, **8-FFA Hỗn chiến**) hoặc tùy chỉnh phe phái/AI tự do cho từng ô người chơi ngay tại Main Menu.

---

## ⚔️ Cơ chế Chiến đấu & Kinh tế Chiến trường

### 1. Hệ thống Kinh tế 3 Tài nguyên & Chuyển hóa Vĩ mô
- 💎 **Khoáng sản (Minerals)**: Khai thác từ các cụm mỏ quặng bằng `ENGINEER // WORKER`. Dùng để xây dựng căn cứ và huấn luyện quân đội.
- ⚡ **Năng lượng (Energy)**: Sản sinh liên tục mỗi giây từ `POWER REACTOR`. Dùng để nghiên cứu công nghệ, sản xuất binh chủng cấp cao và nạp đạn cho tháp canh.
- 👥 **Dân số (Supply)**: Giới hạn quy mô quân đoàn (khởi đầu `4/10`). Mở rộng bằng cách xây dựng `SUPPLY DEPOT` (`+8 Supply`) hoặc `HQ COMMAND CENTER` (`+10 Supply`).
- 🔄 **Cơ chế Trạm Chuyển Hóa (`ENERGY CONVERTER`)**: Khi năng lượng dự trữ vượt ngưỡng **1000⚡**, mỗi trạm tự động chuyển hóa năng lượng dư thừa thành Khoáng sản theo tỷ lệ **100⚡ ➔ 50💎** mỗi giây.

### 2. Ma trận 4 Hệ Sát thương & Chỉ số Kháng (Combat Matrix)
Mọi đơn vị và công trình đều sở hữu 4 chỉ số kháng riêng biệt (`Kháng Vật lý`, `Kháng Năng lượng`, `Kháng Nổ`, `Kháng Ma thuật`) hiển thị trực quan theo thời gian thực trên HUD:

| Hệ sát thương | Màu hiển thị | Đặc tính chiến thuật |
| :--- | :--- | :--- |
| ⚔️ **Vật lý (Physical)** | Trắng / Vàng kim | Sát thương cơ bản từ súng trường, pháo phòng thủ HQ và cự thú. Giảm trừ trực tiếp bởi Giáp Vật Lý. |
| ⚡ **Năng lượng (Energy)** | Xanh Cyan Neon | Chùm tia Plasma và Laser hội tụ tốc độ cao, khắc chế mạnh các mục tiêu giáp vật lý dày. |
| 💥 **Nổ AoE (Explosive)** | Cam Đỏ rực | Tên lửa hắc ám và bọ cảm tử gây sát thương lan diện rộng (bán kính `35px - 75px`). |
| ✨ **Ma thuật (Arcane)** | Tím Huyền ảo | Phi kiếm tiên đạo và pháp trận hư không. **Bỏ qua 50% Kháng Vật Lý** của mục tiêu. |

---

## 👥 Hệ thống 10 Binh chủng Người Que (Units)

| Binh chủng | Nhánh | Chi phí & Thời gian | Máu / Tốc độ | Sát thương & Tầm bắn | Đặc tính độc quyền |
| :--- | :--- | :--- | :--- | :--- | :--- |
| 🔧 **Engineer // Worker** | Cơ bản | `50💎` • `5s` | `90 HP` / `120` | `5 Physical` (`30px` / `1.0s`) | Khai thác mỏ khoáng sản và thi công xây dựng công trình. |
| ⚔️ **Striker // Shock Trooper** | Cơ bản | `75💎` • `7s` | `160 HP` / `140` | `22 Energy` (`40px` / `0.8s`) | Kiếm sĩ Plasma cơ động cao, áp sát chém năng lượng tầm gần. |
| 🎯 **Marksman // Rifleman** | Cơ bản | `100💎 20⚡` • `8s` | `120 HP` / `110` | `14 Physical` (`160px` / `1.1s`) | Xạ thủ tầm xa chủ lực đầu trận, tăng tầm bắn khi nâng cấp Đạn Từ Tính. |
| 🛡️ **Vanguard // Shielder** | Khoa học | `150💎 50⚡` • `12s` | `320 HP` / `90` | `18 Physical` (`45px` / `1.2s`) | Kháng đa hệ cao; tự động tái tạo lá chắn **100 Shield HP** mỗi 5 giây. |
| ⚡ **Siege // Laser Marksman** | Khoa học | `175💎 100⚡` • `15s` | `110 HP` / `95` | `1+ Energy` (`220px` / `0.2s`) | Khóa mục tiêu và chiếu tia Laser liên tục, tích cộng dồn tăng sát thương theo thời gian. |
| 🚀 **Dark Tech // Rocketeer** | Khoa học | `160💎 80⚡` • `14s` | `140 HP` / `95` | `50 Explosive` (`200px` / `1.5s`) | Phóng tên lửa nổ lan AoE bán kính `50px`, hủy diệt cụm lính co cụm. |
| 🗡️ **Celestial // Swordmaster** | Ma thuật | `180💎 90⚡` • `15s` | `170 HP` / `130` | `40 Arcane` (`140px` / `0.1s`) | Tích trữ tối đa **7 phi kiếm** (hồi 1 kiếm/s), phóng kiếm liên thanh xuyên 50% giáp vật lý. |
| 💀 **Void // Shadow Warlock** | Ma thuật | `200💎 120⚡` • `16s` | `135 HP` / `100` | `10% Max HP/s` (`175px` / `2.0s`) | Triệu hồi pháp trận hư không bán kính `75px` tồn tại 5s, đốt **10% máu tối đa/giây**. |
| 🦣 **Behemoth // Titan** | Quái vật | `250💎 100⚡` • `20s` | `650 HP` / `60` | `40 Physical` (`55px` / `1.5s`) | Cự thú siêu chống chịu (`50% Kháng Vật Lý`), dậm chấn động AoE bán kính `35px`. |
| 🦗 **Swarm // Acid Baneling** | Quái vật | `60💎 30⚡` • `6s` | `100 HP` / `160` | `90 Explosive` (`25px` / `0.5s`) | Bọ tốc độ cao lao vào tự nổ axit AoE `70px`, **gây x2 sát thương (180 DMG) lên công trình**. |

---

## 🏢 Hệ thống 6 Công trình Blueprint (Buildings)

- 🏛️ **HQ Command Center** (`400💎` • `15s` • `2000 HP`): Trung tâm chỉ huy đầu não. Đào tạo `Engineer`, điểm trả khoáng sản, cung cấp `+10 Supply` và tích hợp pháo phòng vệ (`100 DMG`, tầm bắn `250px`).
- 📦 **Supply Depot** (`100💎` • `8s` • `800 HP`): Trạm tiếp vận hậu cần, mở rộng giới hạn dân số thêm `+8 Supply`.
- 🔋 **Power Reactor** (`120💎` • `10s` • `700 HP`): Lò phản ứng lượng tử, tự động sản sinh `+8⚡ Năng lượng / giây`.
- ⛺ **Barracks** (`150💎` • `12s` • `1200 HP`): Doanh trại quân sự huấn luyện toàn bộ 9 binh chủng chiến đấu, hỗ trợ đặt điểm tập kết (`Rally Point`).
- 🗼 **Turret Tower** (`125💎 50⚡` • `10s` • `900 HP`): Tháp pháo phòng thủ kiên cố (`85% Kháng Vật Lý`), bắn đạn năng lượng `25 DMG` ở tầm `250px` (chuyển sang bắn đạn nổ AoE `40px` khi có công nghệ Rocket).
- ⚡ **Energy Converter** (`150💎 50⚡` • `12s` • `850 HP`): Trạm chuyển hóa vật chất, tự động quy đổi `100⚡ ➔ 50💎` mỗi giây khi tổng năng lượng vượt mốc `1000⚡`.

---

## 🔬 Cây Công Nghệ 4 Nhánh Nghiên Cứu Song Song (`[T]`)

Người chơi có thể **nghiên cứu đồng thời nhiều công nghệ** thuộc 4 nhánh độc lập và đánh dấu sao `[★]` để đưa vào hàng đợi ưu tiên cho Auto-Play:

1. **Nhánh Khoa học (Science)**:
   - *Tụ Quang Lượng Tử (Focused Laser)*: Mở khóa `Laser Marksman` & `+25%` sát thương Năng lượng toàn quân.
   - *Khiên Lượng Tử (Energy Shielding)*: Mở khóa `Vanguard` & `+40%` Kháng Năng lượng toàn quân.
   - *Bệ Phóng Rocket (Dark Matter Rockets)*: Mở khóa `Dark Rocketeer` & nâng cấp `Turret Tower` bắn đạn nổ AoE `40px`.
   - *Khung Titan (Titan Alloy Plating)*: Tăng `+35%` Kháng Vật lý toàn quân và `+30%` Máu toàn bộ công trình.
2. **Nhánh Ma thuật (Magic)**:
   - *Ngự Kiếm Thuật (Celestial Flying Swords)*: Mở khóa `Taoist Swordmaster`.
   - *Kim Cang Hộ Thể (Golden Bell Shield)*: Tăng `+40%` Kháng Ma thuật/Năng lượng và tự hồi `+3 HP/giây` cho toàn quân.
   - *Chú Ấn Ăn Mòn (Corrosive Curse)*: Đòn đánh của toàn quân bào mòn vĩnh viễn chỉ số Kháng của mục tiêu trúng đòn.
   - *Bộc Phá Hư Không (Void Cataclysm)*: Mở khóa `Void Warlock`.
3. **Nhánh Quái vật (Monster)**:
   - *Vảy Cổ Đại (Primeval Scales)*: Tăng `+35%` Kháng Vật lý và `+20%` Máu tối đa toàn quân.
   - *Biến Dị Khổng Lồ (Behemoth Mutation)*: Mở khóa `Behemoth Titan`.
   - *Bọ Độc Axit (Acid Banelings)*: Mở khóa `Acid Baneling`.
   - *Bầy Đàn Phân Hạch (Hive Proliferation)*: Giảm `-35%` chi phí và thời gian huấn luyện toàn bộ binh chủng, đồng thời tăng tốc chạy.
4. **Nhánh Kinh tế & Phụ trợ (Logistics)**:
   - *Khoan Quang Học (Optical Drills)*: Tăng tốc độ đào và sức chứa khoáng sản của `Engineer` (`10 ➔ 15`).
   - *Lò Phản Ứng Siêu Nạp (Overcharged Reactors)*: Tăng `+50%` sản lượng điện của `Power Reactor`.
   - *Bản Vẽ Gia Cố (Fortified Blueprints)*: Tăng `+30%` Máu công trình và tốc độ xây dựng.
   - *Đạn Từ Tính (Magnetic Rounds)*: Tăng tầm bắn cho `Rifleman` và `Turret Tower`.
   - *Động Cơ Tăng Tốc (Exo Boosters)*: Tăng `+20%` tốc độ di chuyển toàn quân.
   - *Kích Nạp Quá Tải (Overload Strike)*: Tăng `+66%` tốc độ bắn cho `Turret Tower` và `HQ Command Center`.

---

## 🤖 Hệ thống Lập trình Auto-Play (`[B]`) & AI Học Tăng Cường Sâu (Deep RL)

### 1. Tự động hóa Auto-Play & Tuyến Phòng Thủ Tháp Canh
- **Vòng lặp Sản xuất & Vòng lặp Quân sự (Loop Cards Engine)**: Thiết lập chuỗi thẻ lệnh theo cấu trúc `ĐIỀU KIỆN (If) ➔ HÀNH ĐỘNG (Then)` (ví dụ: tự động xin thợ mỏ, tự xây trạm tiếp vận khi sát trần dân số, tự nghiên cứu công nghệ gắn sao `[★]`, tự động phát lệnh tổng tấn công khi gom đủ quy mô quân đoàn).
- **Tuyến Phòng Thủ Tháp Canh (Turret Defense Line)**: Kéo chọn đoạn thẳng phòng ngự từ điểm A đến điểm B trực tiếp trên bản đồ; hệ thống tự động tính toán vector và rải từ 1 đến 3 hàng `Turret Tower` bảo vệ yết hầu căn cứ.

### 2. Kiến trúc Não bộ Deep Dual-Head MLP & Huấn luyện GPU CUDA (`train_bot`)
- **Mô hình Hợp nhất Duy nhất (`assets/rl_model.json`)**:
  - **64 Chiều Đầu Vào (`STATE_DIM = 64`)**: Mã hóa toàn diện trạng thái vĩ mô/vi mô, lưu lượng kinh tế, tỷ lệ chuyển hóa năng lượng, tiến độ công nghệ, tương quan lực lượng và kịch bản trận đấu (`1v1`, `2v2`, `3v3`, `4v4`, `2v2v2`, `8-FFA`).
  - **2 Tầng Ẩn Sâu (768 Neurons)**: `Layer 1 (512 neurons)` + `Layer 2 (256 neurons)` với hàm kích hoạt LeakyReLU ($\alpha = 0.01$), tổng cộng **170,005 tham số (weights & biases)**.
  - **Đầu ra Kép (Dual-Head)**: Dự đoán đồng thời Q-values cho **15 Hành động chiến lược** và **6 Đội hình tác chiến**.
- **Tăng tốc Phần cứng NVIDIA CUDA & Đa luồng CPU**:
  - Tích hợp nhân CUDA C++ (`rust-engine/src/cuda/rl_kernel.cu`) với **Persistent GPU VRAM Buffer**, thực thi Mini-Batch Experience Replay, Forward/Backpropagation và chuẩn hóa Gradient trực tiếp trên GPU (tự động fallback sang CPU nếu máy không có CUDA).
  - Giả lập chiến trường song song trên nhiều luồng CPU (`worker threads`) giúp tăng tốc độ thu thập kinh nghiệm gấp nhiều lần.
- **Giáo trình Huấn luyện Đa tầng (Curriculum & Self-Play)**:
  - **Vòng 1 (7 ván đầu mỗi kịch bản)**: Bot lần lượt đối đầu với đủ **7 Archetype Auto Chuyên sâu** (*Science Laser & Rocket Siege*, *Magic Sword & Void Cataclysm*, *Monster Swarm & Behemoth Titan*, *Bio Infantry Blitz Rush*, *Macro Eco & Multi-row Fortress*, *Combined Titan, Artillery & Void Mage*, *Omni-Tech Grandmaster*) có khả năng tự động chuyển dịch cơ cấu quân theo công nghệ giữa trận.
  - **Từ ván 8 trở đi**: Tự động ngắt Auto mẫu, chuyển hoàn toàn sang **Bot-vs-Bot Self-Play** kết hợp **Champion Pool** để các thế hệ AI tự đối kháng và tiến hóa chiến thuật.

---

## 🎮 Bảng Điều khiển & Phím tắt

| Thao tác / Phím tắt | Chức năng |
| :--- | :--- |
| **Kéo Chuột Trái** | Quét hộp chọn (Box Selection) nhiều đơn vị quân hoặc click chọn công trình. |
| **Chuột Phải** | Di chuyển / Tấn công mục tiêu / Giao thợ mỏ khai thác quặng / Đặt điểm tập kết (`Rally Point`). |
| **Chuột Phải (trên Command Card)** | Hủy đơn vị đang chờ trong hàng đợi sản xuất và hoàn lại 100% tài nguyên. |
| **Phím `A` + Chuột Trái** | **Attack-Move**: Hành quân chiến đấu, tự động tấn công mọi kẻ địch bắt gặp trên đường đi. |
| **Phím `F`** | Chuyển đổi luân phiên 6 Đội hình (`Line`, `Wedge`, `Box`, `Scatter`, `Circle`, `Column`). |
| **Phím `T`** | Mở / Đóng sơ đồ **Cây Công Nghệ (Tech Tree)**. |
| **Phím `B`** | Mở / Đóng bảng lập trình **Tự Động Hóa (Auto-Play)**. |
| **Phím `H`** | Mở / Đóng **Cẩm Nang Chiến Thuật (Game Guide)**. |
| **Phím `Space`** | Tạm dừng (`Pause`) hoặc Tiếp tục trận đấu theo thời gian thực. |
| **Phím `ESC`** | Mở bảng **Cài Đặt (Settings)** / Đóng hộp thoại đang mở / Hủy thao tác đặt móng nhà. |
| **Phím `F3` hoặc `~`** | Bật / Tắt bảng giám sát hiệu năng thời gian thực (`FPS`, `Sim ms`, `Render ms`). |
| **Con lăn Chuột** | Phóng to / Thu nhỏ góc nhìn camera chiến trường (`Zoom 0.5x – 2.0x`). |

*(Lưu ý: Bạn có thể bật/tắt chế độ **Tự động tạm dừng trận đấu khi mở bảng (Auto-Pause on Modal)** ngay trong phần Cài đặt `[ESC]`).*

---

## 🛠️ Hướng dẫn Cài đặt & Phát triển

### Yêu cầu môi trường
- **Node.js** `>= 18` & **npm**
- **Rust Toolchain** (`rustup`, `cargo`) với target `wasm32-unknown-unknown`:
  ```bash
  rustup target add wasm32-unknown-unknown
  cargo install wasm-bindgen-cli
  ```
- *(Tùy chọn)* **NVIDIA CUDA Toolkit (`nvcc`)** nếu muốn tăng tốc huấn luyện AI bằng GPU.

### 1. Cài đặt thư viện giao diện
```bash
npm install
```

### 2. Biên dịch Rust Engine sang WebAssembly
```bash
./build-wasm.sh
# Hoặc sử dụng npm script:
npm run build:wasm
```

### 3. Khởi chạy máy chủ phát triển (Dev Server)
```bash
npm run dev
```
Truy cập trò chơi trên trình duyệt tại địa chỉ được Vite hiển thị (mặc định `http://localhost:5173`).

### 4. Đóng gói bản Production
```bash
npm run build
npm run preview
```

---

## 🧠 Hướng dẫn Huấn luyện AI Bot (`train_bot.sh`)

Dự án cung cấp sẵn kịch bản huấn luyện đa luồng kết hợp GPU CUDA để cập nhật trọng số vào mô hình hợp nhất [`assets/rl_model.json`](file:///home/exblackhole/Desktop/Battefield%20Blueprint/assets/rl_model.json):

```bash
# Cู pháp: ./train_bot.sh [EPISODES] [DT] [MAX_TIME] [THREADS] [RESUME]

# 1. Chạy huấn luyện mặc định (128 ván, dt=0.25s, max_time=1800s, số luồng = CPU cores / 4, tiếp tục từ model cũ)
./train_bot.sh

# 2. Tuỳ chỉnh số ván và giữ số luồng mặc định (truyền "null" vào vị trí THREADS)
./train_bot.sh 256 0.25 1800 null

# 3. Huấn luyện lại từ đầu (khởi tạo trọng số He/Xavier mới hoàn toàn)
./train_bot.sh 128 0.25 1800 null false
```

Sau khi quá trình huấn luyện hoàn tất và ghi nhận trọng số mới vào `assets/rl_model.json`, hãy chạy lại `./build-wasm.sh` để nhúng bộ não AI mới nhất vào gói WebAssembly của trò chơi.

---

## 📂 Cấu trúc Thư mục Chính

```text
├── assets/
│   └── rl_model.json              # Trọng số mạng Neural 170,005 tham số hợp nhất
├── rust-engine/                   # Core RTS Engine & AI Trainer viết bằng Rust
│   ├── src/
│   │   ├── ai/                    # Trích xuất trạng thái 64 chiều, Reward Shaping, Macro & Reflexes
│   │   ├── autoplay/              # Bộ thực thi thẻ lệnh Auto-Play, Tuyến tháp canh & 7 Archetypes
│   │   ├── bin/train_bot.rs       # Trình huấn luyện Headless đa luồng & Self-Play
│   │   ├── config/                # Thông số cân bằng đơn vị, công trình, công nghệ, kinh tế
│   │   ├── cuda/rl_kernel.cu      # Nhân CUDA C++ tăng tốc huấn luyện MLP trên GPU
│   │   ├── renderer/              # Pipeline đồ họa WebGPU/WebGL2 (wgpu) & Multi-Canvas HUD
│   │   ├── simulation/            # Vòng lặp mô phỏng vật lý, kinh tế, chiến đấu, sương mù
│   │   ├── engine.rs              # Quản lý trạng thái GameEngine trung tâm
│   │   └── rl.rs                  # Kiến trúc mạng Deep Dual-Head MLP (RlBrain)
│   └── build.rs                   # Tự động phát hiện và biên dịch CUDA kernel (nếu có nvcc)
├── src/                           # Giao diện React 19 + TypeScript + Tailwind CSS
│   ├── audio/soundEngine.ts       # Bộ tổng hợp âm thanh Web Audio API Procedural Synth
│   ├── bridge/wasmBridge.ts       # Cầu nối giao tiếp 2 chiều giữa React và Rust Wasm
│   └── components/                # MainMenu, TechTreeModal, AutoPlayModal, GuideModal, SettingsModal...
├── build-wasm.sh                  # Script biên dịch Rust sang gói WebAssembly (src/wasm-pkg)
└── train_bot.sh                   # Script khởi chạy huấn luyện AI trên GPU/CPU
```
