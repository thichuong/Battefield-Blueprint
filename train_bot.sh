#!/usr/bin/env bash
set -e

# ==============================================================================
# BATTLEFIELD BLUEPRINT // GPU-ACCELERATED UNIFIED SINGLE-MODEL RL TRAINER
# ==============================================================================
# Kiến trúc mạng Neural:
#   - Deep Dual-Head MLP (64 Inputs -> 512 H1 -> 256 H2 -> 15 Actions + 6 Formations)
#   - Tổng nơ-ron: 768 Neurons ẩn (Layer 1: 512, Layer 2: 256)
#   - Tổng tham số: 170,005 Weights & Biases
#   - Tăng tốc phần cứng: NVIDIA CUDA GPU (RTX 3060) qua kernel FFI (Tự động fallback CPU nếu không có GPU)
#   - Model hợp nhất duy nhất: assets/rl_model.json
#
# Cú pháp sử dụng:
#   ./train_bot.sh [EPISODES] [DT] [MAX_TIME] [THREADS]
#
# Ví dụ:
#   ./train_bot.sh                      # Mặc định (64 ván, dt=0.25s, max_time=1800s, max threads)
#   ./train_bot.sh 100 0.25 1800 16     # Huấn luyện 100 ván trên 16 luồng CPU song song
# ==============================================================================

EPISODES=${1:-64}
DT=${2:-0.25}
MAX_TIME=${3:-1800}
THREADS=${4:-$(nproc 2>/dev/null || echo 8)}

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR/rust-engine"

echo "============================================================"
echo "🧠 BATTLEFIELD BLUEPRINT // HUẤN LUYỆN 1 MODEL AI DUY NHẤT TRÊN GPU"
echo "   - Kiến trúc Neural:     Deep Dual-Head MLP (64 -> 512 -> 256 -> 15 + 6)"
echo "   - Số Neurons ẩn:        768 (512 L1 + 256 L2)"
echo "   - Tổng tham số:         170,005 weights & biases"
echo "   - Tăng tốc phần cứng:   NVIDIA CUDA GPU (RTX 3060 Persistent VRAM Buffer)"
echo "   - Đa luồng mô phỏng:    $THREADS parallel CPU worker threads"
echo "   - Số ván (Episodes):    $EPISODES ván"
echo "   - Tốc độ tua nhanh (dt): ${DT}s / step"
echo "   - Max Game Time:        ${MAX_TIME}s (30 phút game)"
echo "   - Model duy nhất:       assets/rl_model.json"
echo "============================================================"

cargo run --release --bin train_bot -- \
    --episodes "$EPISODES" \
    --dt "$DT" \
    --max-time "$MAX_TIME" \
    --threads "$THREADS"

echo ""
echo "✅ Huấn luyện hoàn tất! Model AI duy nhất 170k tham số đã được cập nhật:"
echo "   - assets/rl_model.json"
echo "   - rust-engine/assets/rl_model.json"
echo "============================================================"
