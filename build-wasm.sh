#!/usr/bin/env bash
set -e
echo "Compiling Rust engine to WebAssembly..."
cd rust-engine
cargo build --target wasm32-unknown-unknown --release
cd ..
mkdir -p src/wasm-pkg
wasm-bindgen --target web --out-dir src/wasm-pkg ./rust-engine/target/wasm32-unknown-unknown/release/rust_engine.wasm
echo "WASM build successfully generated in src/wasm-pkg/!"

