#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
WASM_DIR="$PROJECT_DIR/wasm"

CARGO="/home/prachwal/.cargo/bin/cargo"
WASM_BINDGEN="/home/prachwal/.cargo/bin/wasm-bindgen"

$CARGO build --target wasm32-unknown-unknown --release
$WASM_BINDGEN target/wasm32-unknown-unknown/release/z80_core.wasm \
  --target web --out-dir "$WASM_DIR"

echo "WASM build complete: $WASM_DIR"
