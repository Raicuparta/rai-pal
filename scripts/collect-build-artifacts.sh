#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
PLATFORM="${1:-}"
VERSION="${2:-}"

if [[ -z "$PLATFORM" || -z "$VERSION" ]]; then
	echo "Usage: $0 <windows|linux|macos> <version>" >&2
	exit 1
fi

OUTPUT_DIR="$ROOT_DIR/release-assets"
mkdir -p "$OUTPUT_DIR"

case "$PLATFORM" in
	windows)
		MAIN_SOURCE="$ROOT_DIR/target/bundles/nsis/RaiPal-$VERSION-x86_64.exe"
		SETUP_SOURCE="$ROOT_DIR/target/bundles/nsis/RaiPal-$VERSION-x86_64-setup.exe"
		MAIN_TARGET="$OUTPUT_DIR/RaiPal.exe"
		UPDATER_TARGET="$OUTPUT_DIR/updater-windows.zip"
		PLATFORM_KEY="windows-x86_64"
		;;
	linux)
		MAIN_SOURCE="$ROOT_DIR/target/bundles/appimage/RaiPal-$VERSION-x86_64.AppImage"
		MAIN_TARGET="$OUTPUT_DIR/RaiPal.AppImage"
		UPDATER_TARGET="$OUTPUT_DIR/updater-linux.tar.gz"
		PLATFORM_KEY="linux-x86_64"
		;;
	macos)
		ARCH="$(uname -m)"
		if [[ "$ARCH" == "arm64" ]]; then
			ARCH="aarch64"
		fi
		MAIN_SOURCE="$ROOT_DIR/target/bundles/macos/RaiPal.macos.zip"
		MAIN_TARGET="$OUTPUT_DIR/RaiPal.macos.zip"
		UPDATER_TARGET="$OUTPUT_DIR/updater-macos.zip"
		PLATFORM_KEY="darwin-$ARCH"
		;;
	*)
		echo "Unsupported platform: $PLATFORM. Expected: windows|linux|macos" >&2
		exit 1
		;;
esac

if [[ ! -f "$MAIN_SOURCE" ]]; then
	echo "Build artifact not found: $MAIN_SOURCE" >&2
	exit 1
fi

cp "$MAIN_SOURCE" "$MAIN_TARGET"
echo "Copied $MAIN_SOURCE -> $MAIN_TARGET"

STAGING_DIR="$(mktemp -d)"
trap 'rm -rf "$STAGING_DIR"' EXIT
rm -f "$UPDATER_TARGET"

case "$PLATFORM" in
	windows)
		if ! command -v zip >/dev/null 2>&1; then
			echo "Missing 'zip'. Install zip to package the Windows updater." >&2
			exit 1
		fi
		if [[ ! -f "$SETUP_SOURCE" ]]; then
			echo "Build artifact not found: $SETUP_SOURCE" >&2
			exit 1
		fi
		cp "$SETUP_SOURCE" "$STAGING_DIR/RaiPal-Setup.exe"
		(cd "$STAGING_DIR" && zip -q -X "$UPDATER_TARGET" RaiPal-Setup.exe)
		;;
	linux)
		if ! command -v tar >/dev/null 2>&1; then
			echo "Missing 'tar'. Install tar to package the Linux updater." >&2
			exit 1
		fi
		cp "$MAIN_SOURCE" "$STAGING_DIR/RaiPal.AppImage"
		tar -czf "$UPDATER_TARGET" -C "$STAGING_DIR" RaiPal.AppImage
		;;
	macos)
		cp "$MAIN_SOURCE" "$UPDATER_TARGET"
		;;
esac

echo "Created $UPDATER_TARGET"

if [[ -n "${UPDATE_PRIVATE_KEY:-}" ]]; then
	SIGNATURE="$(cargo run --quiet --release -p rai-pal-updater-signer -- "$UPDATER_TARGET")"
	UPDATER_NAME="$(basename "$UPDATER_TARGET")"
	cat > "$ROOT_DIR/updater-partial-$PLATFORM.json" <<JSON
{
  "version": "$VERSION",
  "platformKey": "$PLATFORM_KEY",
  "platformData": {
    "signature": "$SIGNATURE",
    "url": "https://github.com/Raicuparta/rai-pal/releases/download/v$VERSION/$UPDATER_NAME"
  }
}
JSON
	echo "Generated updater-partial-$PLATFORM.json"
else
	echo "Skipping updater signature (missing UPDATE_PRIVATE_KEY)."
fi
