---
name: procedural-render-audio
description: "Quy trình phát triển đồ họa vector thủ tục 100% (Stickman Tactical Blueprint qua Canvas 2D và WebGPU WGSL shaders) cùng hệ thống âm thanh tổng hợp thời gian thực Web Audio API Procedural Synth (không dùng file ảnh hoặc mp3 tĩnh)."
---

# SKILL: Procedural Render & Audio (Battlefield Blueprint)

Kỹ năng này hướng dẫn Agent cách làm việc với hệ thống Đồ họa Vector Thủ tục (Procedural Vector Rendering) và Bộ Tổng hợp Âm thanh Thủ tục (Procedural Web Audio API Synthesizer) trong dự án Battlefield Blueprint.

---

## 1. Nguyên Tắc Cốt Lõi: 100% Procedural

- **Đồ họa**: Không sử dụng hình ảnh bitmap (PNG, JPG, SVG, WebP). Toàn bộ nhân vật Người Que, công trình kiến trúc, đạn đạo và hiệu ứng hạt được vẽ trực tiếp bằng các hàm toán học.
- **Âm thanh**: Không tải file audio từ đĩa (MP3, WAV, OGG). Toàn bộ âm thanh tiếng súng, tiếng laser, tiếng nổ, tiếng gươm kiếm và còi báo động được tổng hợp từ sóng âm thuần của Web Audio API.

---

## 2. Kỹ Thuật Vẽ Vector Stickman (Rust Renderer)

Tất cả các hàm vẽ nằm trong `rust-engine/src/renderer/`:
- `units.rs`: Vẽ 10 loại binh chủng người que.
- `buildings.rs`: Vẽ 6 công trình kiến trúc blueprint.
- `effects.rs`: Vẽ hiệu ứng đạn đạo, laser, tia chớp, sóng xung kích và vụ nổ.
- `gpu/`: Đường ống WebGPU (`shaders.wgsl`, `sprite_batch.rs`, `fog_pass.rs`).

### Cấu trúc cơ thể Stickman chuẩn:
1. **Đầu (Head)**: Hình tròn bán kính $R \approx 5-8px$, viền neon dày $1.5-2px$.
2. **Thân (Torso)**: Đường thẳng nối từ cổ xuống hông.
3. **Chi (Limbs)**: 2 tay và 2 chân cử động nhịp nhàng theo chu kỳ bước chạy dựa trên hàm sin:
   ```rust
   let leg_angle = (anim_timer * speed * 0.05).sin() * 0.6;
   let arm_angle = -(anim_timer * speed * 0.05).sin() * 0.6;
   ```
4. **Vũ khí / Hiệu ứng đặc trưng**:
   - `Laser Marksman`: Tia laser đỏ/cyan hội tụ.
   - `Swordmaster`: Phi kiếm xoay quanh cơ thể.
   - `Acid Baneling`: Bụng phát sáng phập phồng màu xanh lục acid.
   - `Titan`: Cơ thể khổng lồ với các đường vảy dậm đất.

---

## 3. Bộ Tổng Hợp Âm Thanh Thủ Tục (`src/audio/soundEngine.ts`)

File quản lý: `src/audio/soundEngine.ts`.

### Cách tạo Preset Âm Thanh Mới:
Mỗi âm thanh là một hàm kết hợp giữa `OscillatorNode`, `GainNode`, `BiquadFilterNode` và `BufferSourceNode` (White Noise):

```typescript
// Ví dụ: Tạo tiếng bắn Laser Plasma
playLaser(freq: number = 880, duration: number = 0.15) {
  if (!this.ctx) return;
  const osc = this.ctx.createOscillator();
  const gain = this.ctx.createGain();

  osc.type = 'sawtooth';
  // Quét tần số từ cao xuống thấp (Pitch Drop)
  osc.frequency.setValueAtTime(freq, this.ctx.currentTime);
  osc.frequency.exponentialRampToValueAtTime(110, this.ctx.currentTime + duration);

  // Suy giảm âm lượng nhanh (Envelope Decay)
  gain.gain.setValueAtTime(0.2, this.ctx.currentTime);
  gain.gain.exponentialRampToValueAtTime(0.001, this.ctx.currentTime + duration);

  osc.connect(gain);
  gain.connect(this.masterGain);

  osc.start();
  osc.stop(this.ctx.currentTime + duration);
}
```

### Kích hoạt Âm Thanh Từ Rust Engine:
Trong code mô phỏng Rust, khi xảy ra sự kiện chiến trường:
```rust
self.event_queue.push(Event::PlaySound { sound: "laser".to_string() });
```
Sự kiện sẽ được `wasmBridge.ts` nhận trong hàm `pollEvents()` và tự động chuyển đến `soundEngine.playSound("laser")`.
