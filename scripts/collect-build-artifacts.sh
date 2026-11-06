#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
PLATFORM="${1:-}"
VERSION="${2:-}"

if [[ -z "$PLATFORM" || -z "$VERSION" ]]; then
	echo "Usage: $0 <windows|linux> <version>" >&2
	exit 1
fi

DIST_DIR="$ROOT_DIR/crates/slint-app/dist"
OUTPUT_DIR="$ROOT_DIR/release-assets"
mkdir -p "$OUTPUT_DIR"

case "$PLATFORM" in
	windows)
		SOURCE="$DIST_DIR/RaiPal-Slint-$VERSION-x86_64.exe"
		TARGET="$OUTPUT_DIR/RaiPal.exe"
		;;
	linux)
		SOURCE="$DIST_DIR/RaiPal-Slint-$VERSION-x86_64.AppImage"
		TARGET="$OUTPUT_DIR/RaiPal.AppImage"
		;;
	*)
		echo "Unsupported platform: $PLATFORM. Expected: windows|linux" >&2
		exit 1
		;;
esac

if [[ ! -f "$SOURCE" ]]; then
	echo "Build artifact not found: $SOURCE" >&2
	exit 1
fi

cp "$SOURCE" "$TARGET"
echo "Copied $SOURCE -> $TARGET"
