#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

for arg in "$@"; do
	case "$arg" in
		-h|--help)
			echo "Usage: $0"
			exit 0
			;;
		*)
			echo "Unknown argument: $arg" >&2
			exit 2
			;;
	esac
done

APP_ID="com.raicuparta.pal"
PACKAGE_DIR="$ROOT_DIR/crates/frontend"
FLATPAK_DIR="$PACKAGE_DIR/packaging/flatpak"
MANIFEST="$FLATPAK_DIR/$APP_ID.yml"
CARGO_SOURCES="$FLATPAK_DIR/cargo-sources.json"
BUNDLE_DIR="$ROOT_DIR/target/bundles/flatpak"
BUILD_DIR="$BUNDLE_DIR/build"
REPO_DIR="$BUNDLE_DIR/repo"
PAGES_DIR="$BUNDLE_DIR/pages"
GENERATOR="${FLATPAK_CARGO_GENERATOR:-$BUNDLE_DIR/flatpak-cargo-generator.py}"
RUNTIME_REPO="https://dl.flathub.org/repo/flathub.flatpakrepo"
REPO_URL="https://raicuparta.github.io/rai-pal/repo/"
GPG_KEY_ID="${FLATPAK_GPG_KEY_ID:-}"

VERSION="$(sed -n 's/^version = "\(.*\)"/\1/p' "$ROOT_DIR/Cargo.toml" | head -n 1)"
if [[ -z "$VERSION" ]]; then
	echo "Could not determine version from Cargo.toml." >&2
	exit 1
fi

for command in flatpak flatpak-builder python3 curl; do
	if ! command -v "$command" >/dev/null 2>&1; then
		echo "Missing '$command'. Run this script inside the Flatpak builder container." >&2
		exit 1
	fi
done

SIGN_ARGS=()
PUBLIC_KEY=""
if [[ -n "$GPG_KEY_ID" ]]; then
	if ! command -v gpg >/dev/null 2>&1; then
		echo "Missing 'gpg' for Flatpak repo signing." >&2
		exit 1
	fi

	SIGN_ARGS=(--gpg-sign="$GPG_KEY_ID")
	PUBLIC_KEY="$(gpg --export "$GPG_KEY_ID" | base64 --wrap=0)"

	if [[ -z "$PUBLIC_KEY" ]]; then
		echo "Could not export the public key for $GPG_KEY_ID." >&2
		exit 1
	fi
fi

if [[ ! -f "$MANIFEST" ]]; then
	echo "Missing Flatpak manifest: $MANIFEST" >&2
	exit 1
fi

mkdir -p "$BUNDLE_DIR"

if [[ ! -f "$GENERATOR" ]]; then
	curl -fsSL https://raw.githubusercontent.com/flatpak/flatpak-builder-tools/master/cargo/flatpak-cargo-generator.py \
		-o "$GENERATOR"
fi

python3 "$GENERATOR" "$ROOT_DIR/Cargo.lock" -o "$CARGO_SOURCES"

flatpak remote-add --if-not-exists --user flathub "$RUNTIME_REPO"

rm -rf "$BUILD_DIR" "$REPO_DIR" "$PAGES_DIR"
rm -f "$BUNDLE_DIR"/*.flatpak

flatpak-builder \
	--user \
	--force-clean \
	--disable-rofiles-fuse \
	--install-deps-from=flathub \
	"${SIGN_ARGS[@]}" \
	--repo="$REPO_DIR" \
	"$BUILD_DIR" \
	"$MANIFEST"

flatpak build-update-repo "${SIGN_ARGS[@]}" --prune --prune-depth=1 --generate-static-deltas "$REPO_DIR"

OUTPUT="$BUNDLE_DIR/RaiPal-$VERSION-x86_64.flatpak"
flatpak build-bundle "$REPO_DIR" "$OUTPUT" "$APP_ID" --runtime-repo="$RUNTIME_REPO"

mkdir -p "$PAGES_DIR"
mv "$REPO_DIR" "$PAGES_DIR/repo"
{
	cat <<EOF
[Flatpak Repo]
Title=Rai Pal
Url=$REPO_URL
Homepage=https://github.com/Raicuparta/rai-pal
Comment=Universal game mod manager
EOF
	if [[ -n "$PUBLIC_KEY" ]]; then
		printf 'GPGKey=%s\n' "$PUBLIC_KEY"
	fi
} > "$PAGES_DIR/$APP_ID.flatpakrepo"

echo "Built $OUTPUT"
echo "Built Flatpak repo at $PAGES_DIR"
