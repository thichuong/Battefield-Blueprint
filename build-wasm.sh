#!/usr/bin/env bash
set -e
echo "Compiling Rust engine to WebAssembly..."
cargo build -p rust-engine --target wasm32-unknown-unknown --release
mkdir -p src/wasm-pkg
if [ -f "./target/wasm32-unknown-unknown/release/rust_engine.wasm" ]; then
    WASM_PATH="./target/wasm32-unknown-unknown/release/rust_engine.wasm"
else
    WASM_PATH="./rust-engine/target/wasm32-unknown-unknown/release/rust_engine.wasm"
fi
wasm-bindgen --target web --out-dir src/wasm-pkg "$WASM_PATH"
echo "WASM build successfully generated in src/wasm-pkg/ from $WASM_PATH!"

