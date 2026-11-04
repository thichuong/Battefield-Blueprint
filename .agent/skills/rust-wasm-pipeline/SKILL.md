---
name: rust-wasm-pipeline
description: "Quy trình can thiệp và phát triển Rust Engine cho RTS game Battlefield Blueprint: ECS, Spatial Grid O(1), Fog of War, Unit/Building/Combat simulation, kiểm thử cargo test, biên dịch WebAssembly và đồng bộ wasmBridge.ts."
---

# SKILL: Rust Wasm Pipeline (Battlefield Blueprint)

Kỹ năng này hướng dẫn Agent cách can thiệp, mở rộng và bảo trì lõi mô phỏng Rust Engine và đường ống WebAssembly trong dự án Battlefield Blueprint.

---

## 1. Bản Đồ Cấu Trúc Rust Engine (`rust-engine/src/`)

```text
rust-engine/src/
├── ai/                     # Trích xuất trạng thái 64 chiều (state.rs), Reward shaping, Reflexes
├── autoplay/               # Bộ thực thi thẻ lệnh Auto-Play, Tuyến tháp canh, 7 Archetypes
├── bin/train_bot.rs        # Script huấn luyện Headless đa luồng CPU & GPU Self-Play
├── bridge.rs               # Giao thức lệnh (Command) & Sự kiện (Event) JSON giao tiếp với JS
├── components.rs           # Cấu trúc dữ liệu ECS: Unit, Building, Projectile, Resource, Team
├── config/                 # Hằng số (constants.rs), chỉ số mặc định (defaults.rs), kiểu dữ liệu (types.rs)
├── cuda/rl_kernel.cu       # Kernel CUDA C++ tăng tốc huấn luyện MLP trên GPU
├── cuda_accel.rs           # Rust FFI wrapper gọi hàm CUDA GPU
├── engine.rs               # Bộ điều phối GameEngine, quản lý vòng lặp frame & sim_step
├── fog.rs                  # Ma trận sương mù Fog of War 96x96
├── formation.rs            # Tính toán tọa độ 6 đội hình tác chiến (Line, Wedge, Box, Scatter, Circle, Column)
├── lib.rs                  # Điểm vào wasm-bindgen (init_engine, engine_step, resize_canvas, input handlers)
├── map.rs                  # Quản lý kích thước chiến trường 4800x4800, vị trí mỏ quặng, 8 slots xuất phát
├── renderer/               # Tầng đồ họa WebGPU (wgpu) & Canvas 2D fallback, HUD minimap/command card
├── resources.rs            # Quản lý 3 loại tài nguyên: Minerals, Energy, Supply
├── rl.rs                   # Kiến trúc mạng nơ-ron Deep Dual-Head MLP (170,005 tham số)
├── simulation/             # Logic mô phỏng: combat, economy, buildings, units, commands, hud_input
└── spatial_grid.rs         # Lưới không gian 33x33 (CELL_SIZE = 150.0) tối ưu truy vấn O(1)
```

---

## 2. Kỷ Luật Lập Trình Rust

1. **Hiệu năng Zero-Allocation**:
   - Trong `simulation/`: Không gọi `format!()`, `Vec::new()`, `to_string()` hoặc `clone()` mảng lớn trong các vòng lặp duyệt từng entity mỗi tick.
   - Luôn sử dụng các buffer được cấp phát trước hoặc truyền tham chiếu `&mut Vec<T>` với lời gọi `.clear()` trước khi tái sử dụng.
2. **Truy vấn lân cận qua SpatialGrid**:
   ```rust
   // ĐÚNG: Truy vấn qua lưới không gian O(1)
   spatial_grid.query_radius(pos.x, pos.y, search_radius, &mut nearby_ids);

   // SAI: Duyệt lặp O(N^2) toàn bộ mảng lính
   for other in units.iter() { ... }
   ```
3. **An toàn Bộ nhớ & Không Panic**:
   - Hạn chế tối đa `unwrap()` trong code mô phỏng runtime; sử dụng `if let` hoặc `match` để xử lý an toàn.
   - Game loop không được phép panic làm crash WebAssembly instance.

---

## 3. Quy Trình Biên Dịch & Xác Minh WASM

### Bước 1: Kiểm thử Unit Test Rust
```bash
cargo test -p rust-engine
```
Nếu có test riêng biệt cho simulation hoặc autoplay:
```bash
cargo test -p rust-engine -- simulation
cargo test -p rust-engine -- autoplay
```

### Bước 2: Biên dịch sang WASM
Chạy script tự động phát hiện cargo và sinh file binding:
```bash
./build-wasm.sh
```
Script này thực thi:
1. `cargo build -p rust-engine --target wasm32-unknown-unknown --release`
2. `wasm-bindgen --target web --out-dir src/wasm-pkg target/wasm32-unknown-unknown/release/rust_engine.wasm`

### Bước 3: Đồng bộ hóa TypeScript Bridge
Nếu thêm kiểu lệnh mới trong `rust-engine/src/bridge.rs`:
- Cập nhật định nghĩa TypeScript trong `src/bridge/wasmBridge.ts`:
```typescript
// Thêm command type tương ứng
export type EngineCommand =
  | { type: 'TrainUnit'; unit_type: string }
  | { type: 'BuildStructure'; building_type: string; x: number; y: number }
  | { type: 'YourNewCommand'; payload: any };
```
- Kiểm tra tính tương thích bằng lệnh: `npm run lint`.
