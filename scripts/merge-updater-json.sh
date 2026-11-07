#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
PARTIALS_DIR="${1:-}"

if [[ -z "$PARTIALS_DIR" ]]; then
	echo "Usage: $0 <partials-dir>" >&2
	exit 1
fi

if ! command -v jq >/dev/null 2>&1; then
	echo "Missing 'jq'." >&2
	exit 1
fi

VERSION=""
PLATFORMS="{}"

while IFS= read -r partial; do
	partial_version="$(jq -r '.version' "$partial")"
	platform_key="$(jq -r '.platformKey' "$partial")"

	if [[ -z "$VERSION" ]]; then
		VERSION="$partial_version"
	elif [[ "$partial_version" != "$VERSION" ]]; then
		echo "Version mismatch in $partial: expected $VERSION, got $partial_version" >&2
		exit 1
	fi

	platform_data="$(jq -c '.platformData' "$partial")"
	PLATFORMS="$(jq --arg key "$platform_key" --argjson data "$platform_data" '. + {($key): $data}' <<<"$PLATFORMS")"
	echo "Merged platform: $platform_key"
done < <(find "$PARTIALS_DIR" -type f -name 'updater-partial-*.json')

if [[ -z "$VERSION" ]]; then
	echo "No updater partials found." >&2
	exit 1
fi

CHANGELOG_PATH="$ROOT_DIR/changelogs/v$VERSION.md"
if [[ -f "$CHANGELOG_PATH" ]]; then
	NOTES="$(cat "$CHANGELOG_PATH")"
else
	NOTES="Version $VERSION"
fi

jq -n \
	--arg version "v$VERSION" \
	--arg notes "$NOTES" \
	--arg pub_date "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
	--argjson platforms "$PLATFORMS" \
	'{version: $version, notes: $notes, pub_date: $pub_date, platforms: $platforms}' \
	> "$ROOT_DIR/latest.json"

echo "Generated latest.json with platforms: $(jq -r '.platforms | keys | join(", ")' "$ROOT_DIR/latest.json")"
