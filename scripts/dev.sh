#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

LIVE=0
if [ "${1:-}" = "--live" ]; then
  LIVE=1
  shift
fi

export SLINT_EMIT_DEBUG_INFO=1
export SLINT_MCP_PORT="${SLINT_MCP_PORT:-9315}"

FEATURES="slint/mcp"

# Use a build directory specific to this feature set so its incremental artifacts
# are not invalidated by the other mode, release builds, or rust-analyzer.
if [ "$LIVE" -eq 1 ]; then
  export SLINT_LIVE_PREVIEW=1
  export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$ROOT_DIR/target/dev-live}"
  FEATURES="slint/live-preview,$FEATURES"
else
  export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$ROOT_DIR/target/dev}"
fi

BIN="$CARGO_TARGET_DIR/debug/Rai-Pal"

build() {
  cargo build -p rai-pal-frontend --features "$FEATURES"
}

if [ "$LIVE" -eq 0 ]; then
  build
  exec "$BIN" "$@"
fi

STAMP_DIR="$(mktemp -d)"
RESTART_STAMP="$STAMP_DIR/restart"

APP_PID=""
WATCHER_PID=""

changed_since() {
  [ -n "$(find crates Cargo.toml Cargo.lock \
    \( -name '*.rs' -o -name 'Cargo.toml' -o -name 'Cargo.lock' \) \
    -newermt "@$1" -print -quit 2>/dev/null)" ]
}

wait_for_change() {
  while ! changed_since "$1"; do
    sleep 0.5
  done
}

watch_for_changes() {
  while kill -0 "$$" 2>/dev/null; do
    sleep 0.5
    if changed_since "$1"; then
      touch "$RESTART_STAMP"
      kill -TERM "$APP_PID" 2>/dev/null || true
      return
    fi
  done
}

cleanup() {
  [ -n "$WATCHER_PID" ] && kill "$WATCHER_PID" 2>/dev/null || true
  [ -n "$APP_PID" ] && kill -TERM "$APP_PID" 2>/dev/null || true
  rm -rf "$STAMP_DIR"
}

trap cleanup EXIT
trap 'exit 130' INT TERM

while true; do
  since="$(date +%s.%N)"

  if ! build; then
    echo "==> Build failed, waiting for Rust changes"
    wait_for_change "$(date +%s.%N)"
    continue
  fi

  "$BIN" "$@" &
  APP_PID=$!

  watch_for_changes "$since" &
  WATCHER_PID=$!

  status=0
  wait "$APP_PID" || status=$?
  APP_PID=""

  kill "$WATCHER_PID" 2>/dev/null || true
  wait "$WATCHER_PID" 2>/dev/null || true
  WATCHER_PID=""

  if [ -e "$RESTART_STAMP" ]; then
    rm -f "$RESTART_STAMP"
    echo "==> Rust change detected, restarting"
    continue
  fi

  if [ "$status" -eq 0 ]; then
    exit 0
  fi

  echo "==> App exited with status $status, waiting for Rust changes"
  wait_for_change "$(date +%s.%N)"
done
