#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
PARTIALS_DIR="${1:-}"
EXISTING_PATH="${2:-}"

if [[ -z "$PARTIALS_DIR" ]]; then
	echo "Usage: $0 <partials-dir> [existing-latest-json]" >&2
	exit 1
fi

if ! command -v jq >/dev/null 2>&1; then
	echo "Missing 'jq'." >&2
	exit 1
fi

EXISTING_VERSION=""
EXISTING_PLATFORMS="{}"
EXISTING_PUB_DATE=""

if [[ -n "$EXISTING_PATH" && -f "$EXISTING_PATH" ]]; then
	EXISTING_VERSION="$(jq -r '.version // ""' "$EXISTING_PATH" | sed 's/^v//')"
	EXISTING_PLATFORMS="$(jq -c '.platforms // {}' "$EXISTING_PATH")"
	EXISTING_PUB_DATE="$(jq -r '.pub_date // ""' "$EXISTING_PATH")"
fi

VERSION=""
PLATFORMS="{}"

if [[ -d "$PARTIALS_DIR" ]]; then
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
fi

if [[ -z "$VERSION" ]]; then
	VERSION="$EXISTING_VERSION"
fi

if [[ -z "$VERSION" ]]; then
	echo "No updater partials and no existing latest.json, skipping updater JSON."
	if [[ -n "${GITHUB_OUTPUT:-}" ]]; then
		echo "has_latest=false" >> "$GITHUB_OUTPUT"
	fi
	exit 0
fi

if [[ "$EXISTING_VERSION" == "$VERSION" ]]; then
	PLATFORMS="$(jq -n --argjson existing "$EXISTING_PLATFORMS" --argjson partials "$PLATFORMS" '$existing + $partials')"
	PUB_DATE="$EXISTING_PUB_DATE"
else
	PUB_DATE=""
fi

if [[ -z "$PUB_DATE" ]]; then
	PUB_DATE="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
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
	--arg pub_date "$PUB_DATE" \
	--argjson platforms "$PLATFORMS" \
	'{version: $version, notes: $notes, pub_date: $pub_date, platforms: $platforms}' \
	> "$ROOT_DIR/latest.json"

if [[ -n "${GITHUB_OUTPUT:-}" ]]; then
	echo "has_latest=true" >> "$GITHUB_OUTPUT"
fi

echo "Generated latest.json with platforms: $(jq -r '.platforms | keys | join(", ")' "$ROOT_DIR/latest.json")"
