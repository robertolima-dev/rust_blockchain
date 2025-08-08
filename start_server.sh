#!/usr/bin/env bash
set -euo pipefail

# Defaults for a nice DX
export RUST_LOG="${RUST_LOG:-info}"
export RUST_BACKTRACE=1

echo "🔁 Starting dev server with hot reload (cargo-watch)..."
echo "   Watching: src/, Cargo.toml, .env"
echo

# -q: quiet, -c: clear screen, -w: watch paths
cargo watch -q -c -w src -w Cargo.toml -w .env -x 'run'
