---
name: react-blueprint-ui
description: "Quy trình thiết kế và phát triển giao diện React 19 + Tailwind CSS phong cách Blueprint Tactical Neon, multi-canvas HUD overlay, Minimap, Selection Panel, Command Card, Modals và cơ chế Auto-Pause."
---

# SKILL: React Blueprint UI (Battlefield Blueprint)

Kỹ năng này hướng dẫn Agent cách xây dựng, tạo kiểu và tích hợp các thành phần giao diện người dùng (UI) trong dự án Battlefield Blueprint bằng **React 19 + TypeScript + Tailwind CSS**.

---

## 1. Hệ Thống Kiến Trúc Giao Diện

Giao diện bao gồm 2 lớp chính:
1. **Lớp Multi-Canvas (Tầng Nền)**:
   - Các phần tử tương tác tốc độ cao (Minimap, Command Card, Selection Box, Thanh tài nguyên đỉnh màn hình) được kết xuất qua canvas chuyên dụng quản lý bởi Wasm Engine.
2. **Lớp React Overlay & Modals (Tầng Trên)**:
   - Toàn bộ menu và hộp thoại quản lý chiến thuật nằm trong `src/components/`:
     - `MainMenu.tsx`: Chọn kịch bản (1v1, 2v2, 3v3, 4v4, 2v2v2, 8-FFA), chọn phe, chỉnh AI.
     - `TechTreeModal.tsx`: Sơ đồ 4 nhánh nghiên cứu công nghệ song song kèm tính năng ưu tiên Auto `[★]`.
     - `AutoPlayModal.tsx`: Bộ soạn thảo chuỗi thẻ lệnh Loop Cards và cấu hình tuyến tháp canh.
     - `TurretDefenseModal.tsx`: Giao diện kéo thả tuyến phòng thủ tháp canh.
     - `GuideModal.tsx`: Cẩm nang chiến thuật và cẩm nang phím tắt.
     - `SettingsModal.tsx`: Điều chỉnh âm lượng, độ phân giải, cờ Auto-Pause on Modal.
     - `GameOverModal.tsx`: Bảng tổng kết chiến thắng/thất bại kèm chỉ số kinh tế.

---

## 2. Ngôn Ngữ Thiết Kế: Blueprint Tactical Neon

### Bảng màu chủ đạo:
- Nền vũ trụ / Blueprint tối: `bg-slate-950` (`#030712`), `bg-slate-900/90` (`#0f172a`).
- Đường viền kỹ thuật: `border-cyan-500/30`, `hover:border-cyan-400`.
- Hiệu ứng phát sáng Neon Glow: `shadow-[0_0_15px_rgba(6,182,212,0.15)]`.
- Màu nhấn hệ thống:
  - Cyan (Công nghệ Khoa học & Năng lượng): `text-cyan-400`, `border-cyan-500`.
  - Tím (Ma thuật Arcane): `text-purple-400`, `border-purple-500`.
  - Cam Đỏ (Vũ khí Nổ Explosive / Cảnh báo): `text-orange-400`, `border-rose-500`.
  - Vàng Kim (Vật lý & Khoáng sản): `text-amber-400`, `border-yellow-500`.
  - Lục Emerald (Kinh tế & Dân số): `text-emerald-400`, `border-emerald-500`.

### Kiểu chữ:
- Tiêu đề & Chỉ số: `font-mono tracking-wider uppercase`.
- Các nút bấm: Luôn viết hoa theo phong cách bản vẽ thiết kế (`CONFIRM`, `RESEARCH`, `ENGAGE`).

---

## 3. Cơ Chế Auto-Pause On Modal

Khi mở bất kỳ modal nào (`TechTree`, `AutoPlay`, `Guide`, `Settings`):
1. Đọc thiết lập `autoPauseOnModal` từ `Settings`:
   ```typescript
   if (settings.autoPauseOnModal) {
     wasmBridge.sendCommand({ type: 'Pause' });
   }
   ```
2. Khi đóng modal:
   ```typescript
   if (settings.autoPauseOnModal) {
     wasmBridge.sendCommand({ type: 'Resume' });
   }
   ```

---

## 4. Xử Lý Phím Tắt Bàn Phím (Hotkeys)

Tất cả phím tắt được gom tập trung tại `src/App.tsx`. Khi thêm phím tắt mới:
1. Bỏ qua phím khi người dùng đang thao tác trong form:
   ```typescript
   if (['INPUT', 'TEXTAREA'].includes((e.target as HTMLElement).tagName)) {
     return;
   }
   ```
2. Ngăn ngừa hành vi cuộn trang mặc định của trình duyệt với phím Space (`e.preventDefault()`).
3. Gửi lệnh trực tiếp sang `wasmBridge` hoặc chuyển đổi state hiển thị modal tương ứng.
