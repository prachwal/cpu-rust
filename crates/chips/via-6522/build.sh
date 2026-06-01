#!/usr/bin/env bash
# VIA 6522 is a library crate, no WASM build needed.
set -euo pipefail
cd "$(dirname "$0")"
cargo test
echo "VIA 6522 tests OK"
