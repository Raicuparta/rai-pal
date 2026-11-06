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

BINARY_NAME="rai-pal-slint"
TARGET="x86_64-pc-windows-msvc"
PACKAGE_DIR="$ROOT_DIR/crates/slint-app"
DIST_DIR="$PACKAGE_DIR/dist"
BINARY="$ROOT_DIR/target/$TARGET/release/$BINARY_NAME.exe"

VERSION="$(sed -n 's/^version = "\(.*\)"/\1/p' "$ROOT_DIR/Cargo.toml" | head -n 1)"
if [[ -z "$VERSION" ]]; then
	echo "Could not determine version from Cargo.toml." >&2
	exit 1
fi

if [[ "$SKIP_BUILD" -eq 0 ]]; then
	cargo xwin build --release -p "$BINARY_NAME" --target "$TARGET"
else
	echo "Skipping cargo build (using existing $BINARY)"
fi

if [[ ! -f "$BINARY" ]]; then
	echo "Build did not produce an executable at $BINARY" >&2
	exit 1
fi

mkdir -p "$DIST_DIR"
FINAL_EXE="$DIST_DIR/RaiPal-Slint-$VERSION-x86_64.exe"
cp "$BINARY" "$FINAL_EXE"

echo "Built $FINAL_EXE"
