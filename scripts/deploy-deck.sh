#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
DECK_HOST="${DECK_HOST:-deck@192.168.8.166}"
APPIMAGE="$(find "$ROOT_DIR/target/bundles/appimage" -maxdepth 1 -name '*.AppImage' -print -quit)"

if [[ -z "$APPIMAGE" ]]; then
	echo "No AppImage found in target/bundles/appimage. Build one first." >&2
	exit 1
fi

scp "$APPIMAGE" "$DECK_HOST:/home/deck/Desktop/RaiPal.AppImage"
ssh "$DECK_HOST" chmod +x /home/deck/Desktop/RaiPal.AppImage
