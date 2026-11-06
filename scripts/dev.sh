#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

export SLINT_EMIT_DEBUG_INFO=1
export SLINT_MCP_PORT="${SLINT_MCP_PORT:-9315}"

exec cargo run -p rai-pal-slint --features slint/mcp "$@"
