#!/usr/bin/env bash
set -e

# ==============================================================================
# BATTLEFIELD BLUEPRINT // GPU-ACCELERATED MULTI-MODE & PARALLEL RL TRAINER
# ==============================================================================
# Kiến trúc mạng Neural mới:
#   - Deep Dual-Head MLP (64 Inputs -> 512 H1 -> 256 H2 -> 15 Actions + 6 Formations)
#   - Tổng nơ-ron: 768 Neurons ẩn (Layer 1: 512, Layer 2: 256)
#   - Tổng tham số: 170,005 Weights & Biases
#   - Tăng tốc phần cứng: NVIDIA CUDA GPU (RTX 3060) qua kernel FFI (Tự động fallback CPU nếu không có GPU)
#
# Cú pháp sử dụng:
#   ./train_bot.sh [EPISODES] [DT] [MAX_TIME] [MODE]
#
# Ví dụ:
#   ./train_bot.sh                      # Mặc định (60 eps/mode, dt=0.25s, max_time=1800s, all modes trên GPU)
#   ./train_bot.sh 24 0.25 1800         # 24 eps mỗi mode cho toàn bộ 6 kịch bản song song
#   ./train_bot.sh 50 0.25 1800 1v1     # Chỉ train riêng chế độ 1v1
#   ./train_bot.sh 50 0.25 1800 4v4     # Chỉ train riêng chế độ 4v4
#   ./train_bot.sh 50 0.25 1800 ffa     # Chỉ train riêng chế độ 8-FFA
# ==============================================================================

EPISODES=${1:-60}
DT=${2:-0.25}
MAX_TIME=${3:-1800}
MODE=${4:-"all"}

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR/rust-engine"

echo "============================================================"
echo "🧠 BATTLEFIELD BLUEPRINT // HUẤN LUYỆN AI ĐA CHẾ ĐỘ TRÊN GPU"
echo "   - Kiến trúc Neural:     Deep Dual-Head MLP (64 -> 512 -> 256 -> 15 + 6)"
echo "   - Số Neurons ẩn:        768 (512 L1 + 256 L2)"
echo "   - Tổng tham số:         170,005 weights & biases"
echo "   - Tăng tốc phần cứng:   NVIDIA CUDA GPU (RTX 3060 / libcudar)"
echo "   - Số ván (Episodes):    $EPISODES / mode"
echo "   - Tốc độ tua nhanh (dt): ${DT}s / step"
echo "   - Max Game Time:        ${MAX_TIME}s (30 phút game)"
echo "   - Chế độ huấn luyện:    $MODE"
echo "============================================================"

EXTRA_ARGS=""
if [ "$MODE" != "all" ]; then
    EXTRA_ARGS="--mode $MODE"
fi

cargo run --release --bin train_bot -- \
    --episodes "$EPISODES" \
    --dt "$DT" \
    --max-time "$MAX_TIME" \
    $EXTRA_ARGS

echo ""
echo "✅ Huấn luyện hoàn tất! Các model 170k tham số đã được cập nhật:"
echo "   - assets/rl_model_1v1.json"
echo "   - assets/rl_model_2v2.json"
echo "   - assets/rl_model_3v3.json"
echo "   - assets/rl_model_4v4.json"
echo "   - assets/rl_model_2v2v2.json"
echo "   - assets/rl_model_ffa8.json"
echo "   - assets/rl_model.json (Fallback)"
echo "============================================================"
