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

APP_NAME="RaiPal-Slint"
BINARY_NAME="rai-pal-slint"
PACKAGE_DIR="$ROOT_DIR/crates/slint-app"
DIST_DIR="$PACKAGE_DIR/dist"
APP_DIR="$DIST_DIR/$APP_NAME.app"
BINARY="$ROOT_DIR/target/release/$BINARY_NAME"
ICON_FILE="$PACKAGE_DIR/graphics/icon.icns"
CARGO_TOML="$ROOT_DIR/Cargo.toml"

# The git winit patch used for Wayland drag and drop brings its own `dpi` crate,
# which conflicts with the one muda expects on macOS. Only needed on Linux.
sed -i '' '/^# Needed for wayland drag and drop/,/^winit = /d' "$CARGO_TOML"

if [[ "$SKIP_BUILD" -eq 0 ]]; then
	cargo build --release -p "$BINARY_NAME"
else
	echo "Skipping cargo build (using existing $BINARY)"
fi

if [[ ! -x "$BINARY" ]]; then
	echo "Build did not produce an executable at $BINARY" >&2
	exit 1
fi

VERSION="$(sed -n 's/^version = "\(.*\)"/\1/p' "$CARGO_TOML" | head -n 1)"
if [[ -z "$VERSION" ]]; then
	echo "Could not determine version from Cargo.toml." >&2
	exit 1
fi

mkdir -p "$DIST_DIR"
rm -rf "$APP_DIR"
rm -f "$DIST_DIR"/*.zip

mkdir -p "$APP_DIR/Contents/MacOS" "$APP_DIR/Contents/Resources"
cp "$BINARY" "$APP_DIR/Contents/MacOS/$BINARY_NAME"

if [[ -f "$ICON_FILE" ]]; then
	cp "$ICON_FILE" "$APP_DIR/Contents/Resources/AppIcon.icns"
fi

cat > "$APP_DIR/Contents/Info.plist" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
	<key>CFBundleName</key>
	<string>$APP_NAME</string>
	<key>CFBundleDisplayName</key>
	<string>$APP_NAME</string>
	<key>CFBundleIdentifier</key>
	<string>com.raicuparta.rai-pal-slint</string>
	<key>CFBundleExecutable</key>
	<string>$BINARY_NAME</string>
	<key>CFBundleIconFile</key>
	<string>AppIcon</string>
	<key>CFBundlePackageType</key>
	<string>APPL</string>
	<key>CFBundleShortVersionString</key>
	<string>$VERSION</string>
	<key>CFBundleVersion</key>
	<string>$VERSION</string>
	<key>LSMinimumSystemVersion</key>
	<string>11.0</string>
	<key>NSHighResolutionCapable</key>
	<true/>
</dict>
</plist>
EOF

# Unsigned arm64 binaries won't launch, ad-hoc signing is enough for local testing.
codesign --force --deep --sign - "$APP_DIR"

ditto -c -k --keepParent "$APP_DIR" "$DIST_DIR/$APP_NAME-$VERSION-macos.zip"

echo "Built $DIST_DIR/$APP_NAME-$VERSION-macos.zip"
