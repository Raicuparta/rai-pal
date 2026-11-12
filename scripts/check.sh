#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

MODE="fix"
if [[ "${1:-}" == "--check" ]]; then
	MODE="check"
	shift
fi

if ! command -v slint-tr-extractor >/dev/null 2>&1; then
	echo "slint-tr-extractor is required to check the project." >&2
	echo "It is part of the flake dev shell (nix develop)." >&2
	exit 2
fi

if [[ "$MODE" == "check" ]]; then
	cargo fmt --all -- --check
	cargo clippy --workspace --all-targets
else
	cargo fmt --all
	cargo clippy --workspace --all-targets --fix --allow-dirty --allow-staged
fi

# The build script drops these from its normal output, but the live preview and
# the editor show them, so fail on them here instead of letting them pile up.
RAI_PAL_CHECK_UI_DIAGNOSTICS=1 cargo check -p rai-pal-frontend

scripts/check-translations.sh
