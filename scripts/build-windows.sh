#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
SKIP_BUILD=0

for arg in "$@"; do
	case "$arg" in
		--skip-build) SKIP_BUILD=1 ;;
		-h|--help)
			echo "Usage: $0 [--skip-build]"
			exit 0
			;;
		*)
			echo "Unknown argument: $arg" >&2
			exit 2
			;;
	esac
done

PACKAGE_NAME="rai-pal-frontend"
BINARY_NAME="Rai-Pal"
TARGET="x86_64-pc-windows-msvc"
PACKAGE_DIR="$ROOT_DIR/crates/frontend"
BUNDLE_DIR="$ROOT_DIR/target/bundles/nsis"
BINARY="$ROOT_DIR/target/$TARGET/release/$BINARY_NAME.exe"
INSTALLER_SCRIPT="$PACKAGE_DIR/packaging/windows-installer.nsi"
INSTALLER_ICON="$PACKAGE_DIR/graphics/icon.ico"
INSTALLER_SIDEBAR="$PACKAGE_DIR/graphics/installer-sidebar.bmp"

# The git winit patch is only needed for Wayland drag and drop, and it brings
# its own dpi crate which conflicts with muda on Windows. Remove it here.
sed -i '/^# Needed for wayland drag and drop/,/^winit = /d' "$ROOT_DIR/Cargo.toml"

VERSION="$(sed -n 's/^version = "\(.*\)"/\1/p' "$ROOT_DIR/Cargo.toml" | head -n 1)"
if [[ -z "$VERSION" ]]; then
	echo "Could not determine version from Cargo.toml." >&2
	exit 1
fi

if [[ "$SKIP_BUILD" -eq 0 ]]; then
	cargo xwin build --release -p "$PACKAGE_NAME" --target "$TARGET"
else
	echo "Skipping cargo build (using existing $BINARY)"
fi

if [[ ! -f "$BINARY" ]]; then
	echo "Build did not produce an executable at $BINARY" >&2
	exit 1
fi

if ! command -v makensis >/dev/null 2>&1; then
	echo "Missing 'makensis'. Install NSIS to build the Windows installer." >&2
	exit 1
fi

for file in "$INSTALLER_SCRIPT" "$INSTALLER_ICON" "$INSTALLER_SIDEBAR"; do
	if [[ ! -f "$file" ]]; then
		echo "Missing required installer file: $file" >&2
		exit 1
	fi
done

mkdir -p "$BUNDLE_DIR"
FINAL_EXE="$BUNDLE_DIR/RaiPal-$VERSION-x86_64.exe"
FINAL_INSTALLER="$BUNDLE_DIR/RaiPal-$VERSION-x86_64-setup.exe"
cp "$BINARY" "$FINAL_EXE"

STAGING_DIR="$(mktemp -d)"
trap 'rm -rf "$STAGING_DIR"' EXIT
cp "$BINARY" "$STAGING_DIR/Rai-Pal.exe"

makensis \
	-DVERSION="$VERSION" \
	-DSOURCE_EXE="$STAGING_DIR/Rai-Pal.exe" \
	-DOUTPUT_FILE="$FINAL_INSTALLER" \
	-DICON_FILE="$INSTALLER_ICON" \
	-DSIDEBAR_FILE="$INSTALLER_SIDEBAR" \
	"$INSTALLER_SCRIPT"

echo "Built $FINAL_EXE"
echo "Built $FINAL_INSTALLER"
