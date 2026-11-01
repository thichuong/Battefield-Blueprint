#!/usr/bin/env bash
set -e

# ==============================================================================
# BATTLEFIELD BLUEPRINT // MULTI-MODE & PARALLEL RL TRAINING SCRIPT
# ==============================================================================
# Cú pháp sử dụng:
#   ./train_bot.sh [EPISODES] [DT] [MAX_TIME] [MODE]
#
# Ví dụ:
#   ./train_bot.sh                      # Chạy mặc định (100 eps/mode, dt=0.25s, max_time=1800s, all modes)
#   ./train_bot.sh 50 0.30 1800         # 50 eps mỗi mode, dt=0.30s, max_time=1800s
#   ./train_bot.sh 100 0.25 1800 1v1    # Chỉ train riêng chế độ 1v1
#   ./train_bot.sh 100 0.25 1800 4v4    # Chỉ train riêng chế độ 4v4
# ==============================================================================

EPISODES=${1:-100}
DT=${2:-0.25}
MAX_TIME=${3:-1800}
MODE=${4:-"all"}

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR/rust-engine"

echo "============================================================"
echo "🧠 BATTLEFIELD BLUEPRINT // HUẤN LUYỆN AI ĐA CHẾ ĐỘ CHƠI"
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
echo "✅ Huấn luyện hoàn tất! Các model đã được cập nhật:"
echo "   - assets/rl_model_1v1.json"
echo "   - assets/rl_model_2v2.json"
echo "   - assets/rl_model_3v3.json"
echo "   - assets/rl_model_4v4.json"
echo "   - assets/rl_model_2v2v2.json"
echo "   - assets/rl_model_ffa8.json"
echo "   - assets/rl_model.json (Fallback)"
echo "============================================================"
