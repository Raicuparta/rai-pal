#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

exec cargo clippy --workspace --all-targets --fix --allow-dirty --allow-staged "$@"
