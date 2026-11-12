#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

SLINT_VERSION="$(sed -n '/^name = "slint"$/{n;s/^version = "\(.*\)"$/\1/p;}' Cargo.lock)"
SLINT_SOURCE="$(sed -n '/^name = "slint"$/,/^$/s/^source = "\(.*\)"$/\1/p' Cargo.lock)"

if [[ "${1:-}" == "--install-tool" ]]; then
	if [[ "$SLINT_SOURCE" == git+* ]]; then
		SLINT_URL="${SLINT_SOURCE#git+}"
		cargo install --locked --git "${SLINT_URL%%\?*}" --rev "${SLINT_URL##*#}" slint-tr-extractor
	else
		cargo install --locked "slint-tr-extractor@$SLINT_VERSION"
	fi
	exit 0
fi

if ! command -v slint-tr-extractor >/dev/null 2>&1; then
	echo "slint-tr-extractor is required to extract the strings from the UI." >&2
	echo "It is part of the flake dev shell (nix develop). To install it without Nix:" >&2
	echo "  scripts/check-translations.sh --install-tool" >&2
	exit 2
fi

EXTRACTOR_VERSION="$(slint-tr-extractor --version | awk '{ print $2 }')"

if [[ "$EXTRACTOR_VERSION" != "$SLINT_VERSION" ]]; then
	echo "Warning: slint-tr-extractor $EXTRACTOR_VERSION does not match Slint $SLINT_VERSION" >&2
fi

WORK_DIR="$(mktemp -d)"
trap 'rm -rf "$WORK_DIR"' EXIT

mapfile -t UI_FILES < <(find crates/frontend/ui -name '*.slint' | sort)

slint-tr-extractor -o "$WORK_DIR/messages.pot" "${UI_FILES[@]}"

cargo run --quiet -p rai-pal-i18n-check -- \
	--pot "$WORK_DIR/messages.pot" \
	--translations crates/frontend/translations
