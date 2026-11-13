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

PORTABLE_SOURCE=""
PORTABLE_TARGET=""
PORTABLE_UPDATER_TARGET=""
PORTABLE_PLATFORM_KEY=""

case "$PLATFORM" in
	windows)
		MAIN_SOURCE="$ROOT_DIR/target/bundles/nsis/RaiPal-$VERSION-x86_64-setup.exe"
		PORTABLE_SOURCE="$ROOT_DIR/target/bundles/nsis/RaiPal-$VERSION-x86_64.exe"
		MAIN_TARGET="$OUTPUT_DIR/RaiPal.exe"
		PORTABLE_TARGET="$OUTPUT_DIR/RaiPal.portable.exe"
		UPDATER_TARGET="$OUTPUT_DIR/updater-windows.zip"
		PORTABLE_UPDATER_TARGET="$OUTPUT_DIR/updater-windows-portable.zip"
		PLATFORM_KEY="windows-x86_64"
		PORTABLE_PLATFORM_KEY="windows-x86_64-portable"
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
		APP_SOURCE="$ROOT_DIR/target/bundles/macos/Rai Pal.app"
		MAIN_TARGET="$OUTPUT_DIR/RaiPal.macos.zip"
		UPDATER_TARGET="$OUTPUT_DIR/updater-macos.tar.gz"
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

if [[ -n "$PORTABLE_SOURCE" ]]; then
	if [[ ! -f "$PORTABLE_SOURCE" ]]; then
		echo "Build artifact not found: $PORTABLE_SOURCE" >&2
		exit 1
	fi
	cp "$PORTABLE_SOURCE" "$PORTABLE_TARGET"
	echo "Copied $PORTABLE_SOURCE -> $PORTABLE_TARGET"
fi

STAGING_DIR="$(mktemp -d)"
trap 'rm -rf "$STAGING_DIR"' EXIT
rm -f "$UPDATER_TARGET"
if [[ -n "$PORTABLE_UPDATER_TARGET" ]]; then
	rm -f "$PORTABLE_UPDATER_TARGET"
fi

case "$PLATFORM" in
	windows)
		if ! command -v zip >/dev/null 2>&1; then
			echo "Missing 'zip'. Install zip to package the Windows updater." >&2
			exit 1
		fi
		cp "$MAIN_SOURCE" "$STAGING_DIR/RaiPal-Setup.exe"
		(cd "$STAGING_DIR" && zip -q -X -0 "$UPDATER_TARGET" RaiPal-Setup.exe)
		cp "$PORTABLE_SOURCE" "$STAGING_DIR/RaiPal-Portable.exe"
		(cd "$STAGING_DIR" && zip -q -X -0 "$PORTABLE_UPDATER_TARGET" RaiPal-Portable.exe)
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
		if ! command -v tar >/dev/null 2>&1; then
			echo "Missing 'tar'. Install tar to package the macOS updater." >&2
			exit 1
		fi
		if [[ ! -d "$APP_SOURCE" ]]; then
			echo "Build artifact not found: $APP_SOURCE" >&2
			exit 1
		fi
		tar -czf "$UPDATER_TARGET" -C "$(dirname "$APP_SOURCE")" "$(basename "$APP_SOURCE")"
		;;
esac

echo "Created $UPDATER_TARGET"

write_updater_partial() {
	local archive="$1"
	local platform_key="$2"
	local partial_name="$3"

	if [[ -z "${UPDATE_PRIVATE_KEY:-}" ]]; then
		echo "Skipping updater signature for $archive (missing UPDATE_PRIVATE_KEY)."
		return
	fi

	local signature
	local updater_name
	signature="$(cargo run --quiet --release -p rai-pal-updater-signer -- "$archive")"
	updater_name="$(basename "$archive")"
	cat > "$ROOT_DIR/updater-partial-$partial_name.json" <<JSON
{
  "version": "$VERSION",
  "platformKey": "$platform_key",
  "platformData": {
    "signature": "$signature",
    "url": "https://github.com/Raicuparta/rai-pal/releases/download/v$VERSION/$updater_name"
  }
}
JSON
	echo "Generated updater-partial-$partial_name.json"
}

write_updater_partial "$UPDATER_TARGET" "$PLATFORM_KEY" "$PLATFORM"

if [[ -n "$PORTABLE_UPDATER_TARGET" ]]; then
	echo "Created $PORTABLE_UPDATER_TARGET"
	write_updater_partial "$PORTABLE_UPDATER_TARGET" "$PORTABLE_PLATFORM_KEY" "$PLATFORM-portable"
fi
