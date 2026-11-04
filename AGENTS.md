# AGENTS.md — Agent System Specification
## Project: Battlefield Blueprint (Tactical Stickman RTS)

> **Mục đích tài liệu:** Quy định toàn diện cho tất cả AI Coding Agents (Claude, Cursor, Copilot, Windsurf, Roo Code, AI Studio) khi đọc, phân tích, đề xuất hoặc chỉnh sửa mã nguồn trong kho lưu trữ này.

---

## ⚡ Quick Reference Matrix for Agents

| Thành phần | Công nghệ chủ đạo | Thư mục mã nguồn | Trách nhiệm chính |
| :--- | :--- | :--- | :--- |
| **Simulation Core** | Rust 2021 + ECS | `rust-engine/src/simulation/` | Mô phỏng 60-120+ FPS, di chuyển, vật lý, combat, kinh tế |
| **WASM Bridge** | wasm-bindgen | `rust-engine/src/bridge.rs`, `src/bridge/` | Cầu nối JSON/Event 2 chiều giữa Rust & TypeScript |
| **GPU Rendering** | WebGPU + WebGL2 (`wgpu`) | `rust-engine/src/renderer/` | Vẽ vector thủ tục (Procedural) nhân vật, đạn đạo, sương mù |
| **Audio Synthesizer** | Web Audio API | `src/audio/soundEngine.ts` | 100% âm thanh thủ tục (Laser, nổ, chém, còi báo động) |
| **Auto-Play & AI** | Deep Dual-Head MLP + CUDA | `rust-engine/src/ai/`, `rust-engine/src/rl.rs` | 170k tham số, 7 Archetypes, Loop Cards Engine |
| **HUD & Modals** | React 19 + Tailwind CSS | `src/components/`, `src/App.tsx` | MainMenu, Cây công nghệ, Bảng tự động hóa, Cài đặt |

---

## 🧭 Bản đồ Tài liệu Agent

Khi thực hiện nhiệm vụ trên dự án, Agent PHẢI tham khảo các tài liệu tương ứng:

1. **[AGENT_RULES.md](./AGENT_RULES.md)**: Hệ thống luật bất biến (Architectural Invariants, Performance Laws, Asset Rules, Balance Matrix, Anti-patterns).
2. **[AGENT_GUIDE.md](./AGENT_GUIDE.md)**: Quy trình vận hành chuẩn (SOP 1 đến SOP 7, quy trình kiểm thử và xác minh).
3. **Thư mục Agent Skills (`.agent/skills/`)**:
   - `rust-wasm-pipeline`: Kỹ thuật can thiệp Rust Engine, kiểm thử và biên dịch WASM.
   - `gameplay-balancing-rl`: Kỹ thuật cân bằng chỉ số RTS, huấn luyện Deep RL bằng GPU/CPU.
   - `procedural-render-audio`: Kỹ thuật vẽ vector hình học và tổng hợp âm thanh bằng Web Audio API.
   - `react-blueprint-ui`: Kỹ thuật phát triển giao diện React 19 phong cách Blueprint Neon.

---

## 🚨 5 Quy Tắc Bất Di Bất Dịch (Non-Negotiable Rules)

1. **Không tạo asset file tĩnh (No Static Media Files)**: Tuyệt đối không thêm file `.png`, `.jpg`, `.svg`, `.mp3`, `.wav`. Toàn bộ đồ họa phải là hàm vẽ toán học (Canvas2D / WGSL Shader) và toàn bộ âm thanh phải dùng `soundEngine.ts` (Web Audio API Procedural Synth).
2. **Mô phỏng thuộc về Rust, Giao diện thuộc về React**: Không bao giờ tính toán sát thương, máu, đường đi hay kinh tế trong React. React chỉ nhận sự kiện và gửi lệnh qua `wasmBridge`.
3. **Hiệu năng Zero-Allocation trong Tick Loop**: Không cấp phát bộ nhớ động (`Vec::new`, string formatting) trong hàm `engine_step()` hoặc các vòng lặp duyệt đơn vị mỗi frame.
4. **Đồng bộ hóa WASM bắt buộc**: Mọi thay đổi trong `rust-engine/` đều phải được kiểm tra qua `cargo test` và biên dịch lại bằng `./build-wasm.sh` trước khi hoàn tất turn.
5. **Giữ vững mô hình AI duy nhất (`assets/rl_model.json`)**: Kiến trúc mạng Neural là Dual-Head MLP cố định (64 input -> 512 H1 -> 256 H2 -> 15 hành động + 6 đội hình). Không thay đổi số chiều input/output mà không cập nhật đồng thời `rl.rs`, `state.rs`, `train_bot.rs` và `rl_kernel.cu`.
