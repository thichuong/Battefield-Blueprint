/**
 * Tiện ích tự động đo đạc và phát hiện tần số quét màn hình (Hz)
 * Hỗ trợ các chuẩn màn hình phổ biến: 60Hz, 75Hz, 90Hz, 120Hz, 144Hz, 165Hz, 240Hz, 360Hz...
 */

let detectedHz = 60;
let isDetectionFinished = false;
const listeners: Array<(hz: number) => void> = [];

const STANDARD_RATES = [60, 75, 90, 120, 144, 165, 240, 360];

function snapToStandardRate(rawHz: number): number {
  for (const std of STANDARD_RATES) {
    // Nếu sai lệch dưới 4.5% so với chuẩn (ví dụ 143.8Hz so với 144Hz, 59.9Hz so với 60Hz)
    if (Math.abs(rawHz - std) / std <= 0.045) {
      return std;
    }
  }
  return Math.max(30, Math.min(360, Math.round(rawHz)));
}

/**
 * Bắt đầu quy trình lấy mẫu đo tần số quét màn hình qua requestAnimationFrame.
 * Hoạt động ngầm ngay khi ứng dụng khởi chạy và thông báo khi hoàn thành.
 */
export function initScreenRefreshRateDetection(): () => void {
  if (isDetectionFinished) return () => {};

  let frameCount = 0;
  let lastTimestamp = 0;
  const deltas: number[] = [];
  const TOTAL_SAMPLES = 25;
  let animationId: number | null = null;

  const measure = (timestamp: number) => {
    if (lastTimestamp > 0) {
      const dt = timestamp - lastTimestamp;
      // Bỏ qua frame warmup đầu tiên và các frame bị gián đoạn do đổi tab
      if (frameCount > 2 && dt > 2 && dt < 60) {
        deltas.push(dt);
      }
    }
    lastTimestamp = timestamp;
    frameCount++;

    if (deltas.length >= TOTAL_SAMPLES) {
      // Sắp xếp để lấy trung vị (median) - loại trừ tuyệt đối mọi đột biến gián đoạn
      deltas.sort((a, b) => a - b);
      const medianDt = deltas[Math.floor(deltas.length / 2)];
      if (medianDt > 0) {
        const rawHz = 1000 / medianDt;
        detectedHz = snapToStandardRate(rawHz);
      }
      isDetectionFinished = true;
      listeners.forEach((fn) => fn(detectedHz));
      listeners.length = 0;
      return;
    }

    animationId = requestAnimationFrame(measure);
  };

  animationId = requestAnimationFrame(measure);

  return () => {
    if (animationId !== null) {
      cancelAnimationFrame(animationId);
    }
  };
}

/**
 * Lấy tần số quét màn hình đã phát hiện (mặc định 60 nếu chưa xong).
 */
export function getDetectedScreenHz(): number {
  return detectedHz;
}

/**
 * Đăng ký nhận thông báo khi đã phát hiện xong tần số quét.
 */
export function onScreenRefreshRateReady(callback: (hz: number) => void): () => void {
  if (isDetectionFinished) {
    callback(detectedHz);
    return () => {};
  }
  listeners.push(callback);
  return () => {
    const idx = listeners.indexOf(callback);
    if (idx !== -1) listeners.splice(idx, 1);
  };
}
