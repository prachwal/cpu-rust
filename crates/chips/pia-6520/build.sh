#!/usr/bin/env bash
# PIA 6520 is a library crate, no WASM build needed.
# Its tests are run via `cargo test`.
set -euo pipefail
cd "$(dirname "$0")"
cargo test
echo "PIA 6520 tests OK"
