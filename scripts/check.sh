#!/usr/bin/env bash
set -euo pipefail

# Fast checks (formatting, translations) run by default.
# Pass --full to also run the slow compile-heavy checks (clippy + UI diagnostics),
# which is what builds and pushes to main use.

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

MODE="fix"
FULL=0
for arg in "$@"; do
	case "$arg" in
	--check) MODE="check" ;;
	--full) FULL=1 ;;
	*)
		echo "Unknown argument: $arg" >&2
		echo "Usage: $0 [--check] [--full]" >&2
		exit 2
		;;
	esac
done

if ! command -v slint-tr-extractor >/dev/null 2>&1; then
	echo "slint-tr-extractor is required to check the project." >&2
	echo "It is part of the flake dev shell (nix develop)." >&2
	exit 2
fi

if [[ "$MODE" == "check" ]]; then
	cargo fmt --all -- --check
else
	cargo fmt --all
fi

scripts/check-translations.sh

if [[ "$FULL" == "1" ]]; then
	if [[ "$MODE" == "check" ]]; then
		cargo clippy --workspace --all-targets
	else
		cargo clippy --workspace --all-targets --fix --allow-dirty --allow-staged
	fi

	# The build script drops these from its normal output, but the live preview and
	# the editor show them, so fail on them here instead of letting them pile up.
	RAI_PAL_CHECK_UI_DIAGNOSTICS=1 cargo check -p rai-pal-frontend
fi
