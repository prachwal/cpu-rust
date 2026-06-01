#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

CARGO="/home/prachwal/.cargo/bin/cargo"
WASM_BINDGEN="/home/prachwal/.cargo/bin/wasm-bindgen"

$CARGO build --target wasm32-unknown-unknown --release
$WASM_BINDGEN "$SCRIPT_DIR/target/wasm32-unknown-unknown/release/chip8_core.wasm" \
  --target web --out-dir "$PROJECT_DIR/wasm"

echo "WASM build complete"
