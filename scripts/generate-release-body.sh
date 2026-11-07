#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
VERSION="${1:-}"

if [[ -z "$VERSION" ]]; then
	echo "Usage: $0 <version>" >&2
	exit 1
fi

CHANGELOG_PATH="$ROOT_DIR/changelogs/v$VERSION.md"
if [[ -f "$CHANGELOG_PATH" ]]; then
	CHANGELOG="$(cat "$CHANGELOG_PATH")"
else
	echo "No changelog found at $CHANGELOG_PATH" >&2
	CHANGELOG="No changelog 🤷"
fi

BODY_PATH="$ROOT_DIR/temp_release_body.md"
RELEASE_BODY="$CHANGELOG

[![Download for Windows](https://shields.io/badge/-🪟_Download_Rai_Pal_for_Windows-3c7dcf?style=for-the-badge)](https://github.com/Raicuparta/rai-pal/releases/download/v$VERSION/RaiPal.exe)
[![Download for Linux](https://shields.io/badge/-🐧_Download_Rai_Pal_for_Linux-dc4131?style=for-the-badge)](https://github.com/Raicuparta/rai-pal/releases/download/v$VERSION/RaiPal.AppImage)
"

printf '%s' "$RELEASE_BODY" > "$BODY_PATH"

if [[ -n "${GITHUB_OUTPUT:-}" ]]; then
	echo "body_path=$BODY_PATH" >> "$GITHUB_OUTPUT"
fi

echo "Written to: $BODY_PATH"
