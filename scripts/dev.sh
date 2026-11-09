#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

export SLINT_EMIT_DEBUG_INFO=1
export SLINT_MCP_PORT="${SLINT_MCP_PORT:-9315}"

# Use a build directory specific to this feature set so its incremental artifacts
# are not invalidated by dev-live.sh, release builds, or rust-analyzer.
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$ROOT_DIR/target/dev}"

exec cargo run -p rai-pal-slint --features slint/mcp "$@"
