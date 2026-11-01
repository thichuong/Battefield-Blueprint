#!/usr/bin/env bash
set -e

EPISODES=${1:-100}
DT=${2:-0.15}

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR/rust-engine"

echo "Running Bot RL Trainer with $EPISODES episodes (dt=$DT)..."
cargo run --release --bin train_bot -- --episodes "$EPISODES" --dt "$DT" --output "assets/rl_model.json"
echo "Training finished successfully! Weights saved to rust-engine/assets/rl_model.json"

