#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"

VERSION="$(sed -n 's/^version = "\(.*\)"/\1/p' "$ROOT_DIR/Cargo.toml" | head -n 1)"
if [[ -z "$VERSION" ]]; then
	echo "Could not determine version from Cargo.toml." >&2
	exit 1
fi

if [[ -n "${GITHUB_OUTPUT:-}" ]]; then
	echo "version=$VERSION" >> "$GITHUB_OUTPUT"
	echo "tag_name=v$VERSION" >> "$GITHUB_OUTPUT"
else
	echo "version=$VERSION"
	echo "tag_name=v$VERSION"
fi
