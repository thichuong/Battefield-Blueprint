#!/usr/bin/env bash
set -e
if ! command -v cargo &> /dev/null; then
    if [ -f "src/wasm-pkg/rust_engine_bg.wasm" ]; then
        echo "Cargo not found, using existing pre-built WASM in src/wasm-pkg/."
        exit 0
    else
        echo "Error: cargo is not installed and prebuilt WASM not found."
        exit 1
    fi
fi
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

