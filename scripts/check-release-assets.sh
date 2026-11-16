#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
PLATFORM="${1:-}"

if [[ -z "$PLATFORM" ]]; then
	echo "Usage: $0 <windows|linux|macos>" >&2
	exit 1
fi

source "$ROOT_DIR/scripts/get-release-asset-names.sh"

if ! EXPECTED_ASSET_NAMES="$(release_asset_names "$PLATFORM")"; then
	echo "Unsupported platform: $PLATFORM. Expected: windows|linux|macos" >&2
	exit 1
fi

mapfile -t EXPECTED_ASSETS <<<"$EXPECTED_ASSET_NAMES"
mapfile -t PUBLISHED_ASSETS <<<"${PUBLISHED_ASSETS:-}"

COLLISIONS=()
for expected in "${EXPECTED_ASSETS[@]}"; do
	for published in "${PUBLISHED_ASSETS[@]}"; do
		if [[ "$expected" == "$published" ]]; then
			COLLISIONS+=("$expected")
			break
		fi
	done
done

if [[ ${#COLLISIONS[@]} -gt 0 ]]; then
	echo "Published release already has $PLATFORM assets:" >&2
	printf '  - %s\n' "${COLLISIONS[@]}" >&2
	echo "Refusing to overwrite assets on a published release. Bump the version, or re-run the workflow with 'Overwrite Published Assets' enabled." >&2
	exit 1
fi

echo "No conflicting published $PLATFORM assets."
