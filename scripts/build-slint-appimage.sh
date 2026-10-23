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

APP_ID="rai-pal-slint"
BINARY_NAME="rai-pal-slint"
PACKAGE_DIR="$ROOT_DIR/backend/slint-app"
DIST_DIR="$PACKAGE_DIR/dist"
APPDIR="$DIST_DIR/$APP_ID.AppDir"
DESKTOP_FILE="$PACKAGE_DIR/packaging/$APP_ID.desktop"
APPRUN_FILE="$PACKAGE_DIR/packaging/AppRun"
ICON_FILE="$ROOT_DIR/backend/tauri-app/graphics/icon.png"
BINARY="$ROOT_DIR/backend/target/release/$BINARY_NAME"

if [[ "$SKIP_BUILD" -eq 0 ]] && ! command -v cargo >/dev/null 2>&1; then
	echo "Missing 'cargo'. Run this script inside 'nix develop'." >&2
	exit 1
fi

linuxdeploy_appimage=""

if [[ -n "${LINUXDEPLOY:-}" ]]; then
	if [[ "$LINUXDEPLOY" == *.AppImage ]]; then
		linuxdeploy_appimage="$LINUXDEPLOY"
	else
		linuxdeploy_cmd="$LINUXDEPLOY"
	fi
elif command -v linuxdeploy >/dev/null 2>&1; then
	linuxdeploy_cmd="linuxdeploy"
elif [[ -x "${XDG_CACHE_HOME:-$HOME/.cache}/tauri/linuxdeploy-x86_64.AppImage" ]]; then
	linuxdeploy_appimage="${XDG_CACHE_HOME:-$HOME/.cache}/tauri/linuxdeploy-x86_64.AppImage"
else
	echo "Missing 'linuxdeploy'. Run inside 'nix develop' or the Linux builder container." >&2
	exit 1
fi

if [[ -n "$linuxdeploy_appimage" ]]; then
	linuxdeploy_cmd="$(mktemp -t rai-pal-linuxdeploy.XXXXXX.AppImage)"
	trap 'rm -f "$linuxdeploy_cmd"' EXIT
	cp "$linuxdeploy_appimage" "$linuxdeploy_cmd"
	dd if=/dev/zero of="$linuxdeploy_cmd" bs=1 count=4 seek=8 conv=notrunc status=none
	chmod +x "$linuxdeploy_cmd"
	export APPIMAGE_EXTRACT_AND_RUN=1
fi

for file in "$DESKTOP_FILE" "$ICON_FILE"; do
	if [[ ! -f "$file" ]]; then
		echo "Missing required file: $file" >&2
		exit 1
	fi
done

VERSION="$(sed -n 's/^version = "\(.*\)"/\1/p' "$ROOT_DIR/backend/Cargo.toml" | head -n 1)"
if [[ -z "$VERSION" ]]; then
	echo "Could not determine version from backend/Cargo.toml." >&2
	exit 1
fi

if [[ "$SKIP_BUILD" -eq 0 ]]; then
	cargo build --release --manifest-path "$ROOT_DIR/backend/Cargo.toml" -p "$BINARY_NAME"
else
	echo "Skipping cargo build (using existing $BINARY)"
fi

if [[ ! -x "$BINARY" ]]; then
	echo "Build did not produce an executable at $BINARY" >&2
	exit 1
fi

mkdir -p "$DIST_DIR"
rm -rf "$APPDIR"
rm -f "$DIST_DIR"/*.AppImage

EXCLUDES=(
	"libEGL.so.*"
	"libGL.so.*"
	"libGLX.so.*"
	"libOpenGL.so.*"
	"libdrm.so.*"
	"libgbm.so.*"
	"libX11.so.*"
	"libX11-xcb.so.*"
)

linuxdeploy_args=(
	--appdir "$APPDIR"
	--executable "$BINARY"
	--desktop-file "$DESKTOP_FILE"
	--custom-apprun "$APPRUN_FILE"
	--icon-file "$ICON_FILE"
	--icon-filename "$APP_ID"
	--output appimage
)

for library in "${EXCLUDES[@]}"; do
	linuxdeploy_args+=(--exclude-library "$library")
done

for library in libfontconfig.so.1 libfreetype.so.6 libexpat.so.1 libz.so.1; do
	library_path="$(ldd "$BINARY" | awk -v library="$library" '$1 == library { print $3 }')"
	if [[ -n "$library_path" ]]; then
		linuxdeploy_args+=(--library "$library_path")
	fi
done

force_pkg_library() {
	local module="$1"
	local library="$2"
	local library_dir
	library_dir="$(pkg-config --variable=libdir "$module" 2>/dev/null || true)"

	if [[ -n "$library_dir" && -e "$library_dir/$library" ]]; then
		linuxdeploy_args+=(--library "$library_dir/$library")
	else
		echo "Warning: could not find $library from pkg-config module $module" >&2
	fi
}

force_pkg_library wayland-client libwayland-client.so.0
force_pkg_library wayland-cursor libwayland-cursor.so.0
force_pkg_library wayland-egl libwayland-egl.so.1
force_pkg_library xkbcommon libxkbcommon.so.0
force_pkg_library xkbcommon-x11 libxkbcommon-x11.so.0
force_pkg_library xcb libxcb-xkb.so.1


pushd "$DIST_DIR" >/dev/null
"$linuxdeploy_cmd" "${linuxdeploy_args[@]}"
popd >/dev/null

GENERATED_APPIMAGE="$(find "$DIST_DIR" -maxdepth 1 -name '*.AppImage' -print -quit)"
if [[ -z "$GENERATED_APPIMAGE" ]]; then
	echo "linuxdeploy did not produce an AppImage." >&2
	exit 1
fi

FINAL_APPIMAGE="$DIST_DIR/RaiPal-Slint-$VERSION-x86_64.AppImage"
mv "$GENERATED_APPIMAGE" "$FINAL_APPIMAGE"
chmod +x "$FINAL_APPIMAGE"

echo "Built $FINAL_APPIMAGE"
