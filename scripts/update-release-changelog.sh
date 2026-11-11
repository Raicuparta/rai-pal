#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
TAG="${1:-}"

if ! command -v gh >/dev/null 2>&1; then
	echo "Missing 'gh'." >&2
	exit 1
fi

if [[ -z "$TAG" ]]; then
	TAG="$(bash "$ROOT_DIR/scripts/get-version.sh" | sed -n 's/^tag_name=//p')"
fi

VERSION="${TAG#v}"

if ! gh release view "$TAG" >/dev/null 2>&1; then
	echo "Release '$TAG' does not exist." >&2
	exit 1
fi

bash "$ROOT_DIR/scripts/generate-release-body.sh" "$VERSION"
gh release edit "$TAG" --notes-file "$ROOT_DIR/temp_release_body.md"
echo "Updated release body of $TAG"

TEMP_DIR="$(mktemp -d)"

if gh release download "$TAG" --pattern latest.json --dir "$TEMP_DIR" 2>/dev/null; then
	if ! command -v jq >/dev/null 2>&1; then
		echo "Missing 'jq'." >&2
		exit 1
	fi

	CHANGELOG_PATH="$ROOT_DIR/changelogs/v$VERSION.md"
	if [[ -f "$CHANGELOG_PATH" ]]; then
		NOTES="$(cat "$CHANGELOG_PATH")"
	else
		NOTES="Version $VERSION"
	fi

	jq --arg notes "$NOTES" '.notes = $notes' "$TEMP_DIR/latest.json" > "$TEMP_DIR/updated.json"
	mv "$TEMP_DIR/updated.json" "$TEMP_DIR/latest.json"
	gh release upload "$TAG" "$TEMP_DIR/latest.json" --clobber
	echo "Updated latest.json notes of $TAG"
else
	echo "No latest.json asset on $TAG, skipping updater notes."
fi
