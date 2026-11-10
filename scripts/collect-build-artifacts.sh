#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
PLATFORM="${1:-}"
VERSION="${2:-}"

if [[ -z "$PLATFORM" || -z "$VERSION" ]]; then
	echo "Usage: $0 <windows|linux> <version>" >&2
	exit 1
fi

OUTPUT_DIR="$ROOT_DIR/release-assets"
mkdir -p "$OUTPUT_DIR"

case "$PLATFORM" in
	windows)
		SOURCE="$ROOT_DIR/target/bundles/nsis/RaiPal-$VERSION-x86_64-setup.exe"
		TARGET="$OUTPUT_DIR/RaiPal.exe"
		PLATFORM_KEY="windows-x86_64"
		;;
	linux)
		SOURCE="$ROOT_DIR/target/bundles/appimage/RaiPal-$VERSION-x86_64.AppImage"
		TARGET="$OUTPUT_DIR/RaiPal.AppImage"
		PLATFORM_KEY="linux-x86_64"
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

if [[ -n "${UPDATE_PRIVATE_KEY:-}" ]]; then
	SIGNATURE="$(cargo run --quiet --release -p rai-pal-updater-signer -- "$TARGET")"
	TARGET_NAME="$(basename "$TARGET")"
	cat > "$ROOT_DIR/updater-partial-$PLATFORM.json" <<JSON
{
  "version": "$VERSION",
  "platformKey": "$PLATFORM_KEY",
  "platformData": {
    "signature": "$SIGNATURE",
    "url": "https://github.com/Raicuparta/rai-pal/releases/download/v$VERSION/$TARGET_NAME"
  }
}
JSON
	echo "Generated updater-partial-$PLATFORM.json"
else
	echo "Skipping updater signature (missing UPDATE_PRIVATE_KEY)."
fi
