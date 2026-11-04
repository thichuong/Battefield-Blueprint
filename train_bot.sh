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
# Lịch trình huấn luyện:
#   - Mỗi kịch bản (1v1, 2v2, 3v3, 4v4, 2v2v2, FFA8):
#     + Vòng 1 (7 ván đầu): Luyện với đủ 7 thể loại Auto có sẵn.
#     + Từ ván 8 trở đi: Ngắt Auto, chuyển hẳn sang Bot vs Bot tự đối kháng (Self-Play).
#
# Cú pháp sử dụng:
#   ./train_bot.sh [EPISODES] [DT] [MAX_TIME] [THREADS] [RESUME]
#
# Ví dụ:
#   ./train_bot.sh                          # Mặc định (128 ván, dt=0.25s, max_time=1800s, max threads/4, resume)
#   ./train_bot.sh 128 0.25 1800 null       # Dùng threads mặc định (max threads/4)
#   ./train_bot.sh 128 0.25 1800 null false # Huấn luyện từ đầu (không nạp kết quả cũ)
# ==============================================================================

TOTAL_CORES=$(nproc 2>/dev/null || echo 8)
CALC_THREADS=$(( TOTAL_CORES / 4 ))
if [ "$CALC_THREADS" -lt 1 ]; then
    CALC_THREADS=1
fi

EPISODES=${1:-128}
DT=${2:-0.25}
MAX_TIME=${3:-1800}

# Nếu THREADS rỗng hoặc là chuỗi "null", mặc định là max threads / 4
THREADS=$4
if [ -z "$THREADS" ] || [ "$THREADS" = "null" ]; then
    THREADS=$CALC_THREADS
fi

RESUME_ARG=${5:-true}
EXTRA_FLAGS=()

# Xử lý cờ resume hoặc scratch
if [ "$RESUME_ARG" = "false" ] || [ "$RESUME_ARG" = "0" ] || [ "$RESUME_ARG" = "scratch" ] || [ "$RESUME_ARG" = "fresh" ]; then
    EXTRA_FLAGS+=(--scratch)
    MODE_DESC="🌱 Huấn luyện lại từ đầu (Khởi tạo ngẫu nhiên He/Xavier)"
else
    EXTRA_FLAGS+=(--resume)
    MODE_DESC="🔄 Tiếp tục huấn luyện lũy kế từ kết quả trước đó (assets/rl_model.json)"
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR/rust-engine"

echo "============================================================"
echo "🧠 BATTLEFIELD BLUEPRINT // HUẤN LUYỆN 1 MODEL AI DUY NHẤT TRÊN GPU"
echo "   - Kiến trúc Neural:     Deep Dual-Head MLP (64 -> 512 -> 256 -> 15 + 6)"
echo "   - Số Neurons ẩn:        768 (512 L1 + 256 L2)"
echo "   - Tổng tham số:         170,005 weights & biases"
echo "   - Tăng tốc phần cứng:   NVIDIA CUDA GPU (RTX 3060 Persistent VRAM Buffer)"
echo "   - Đa luồng mô phỏng:    $THREADS parallel CPU worker threads (Tổng $TOTAL_CORES cores / 4)"
echo "   - Số ván (Episodes):    $EPISODES ván"
echo "   - Tốc độ tua nhanh (dt): ${DT}s / step"
echo "   - Max Game Time:        ${MAX_TIME}s (30 phút game)"
echo "   - Quy luật Auto:        1 vòng đầu (7 ván) đủ 7 Archetypes -> Bot vs Bot tự đối kháng"
echo "   - Chế độ tiếp tục:      $MODE_DESC"
echo "   - Model duy nhất:       assets/rl_model.json"
echo "============================================================"

cargo run --release --bin train_bot -- \
    --episodes "$EPISODES" \
    --dt "$DT" \
    --max-time "$MAX_TIME" \
    --threads "$THREADS" \
    "${EXTRA_FLAGS[@]}"

echo ""
echo "✅ Huấn luyện hoàn tất! Model AI duy nhất 170k tham số đã được cập nhật:"
echo "   - assets/rl_model.json"
echo "   - rust-engine/assets/rl_model.json"
echo "============================================================"
