# Scripts

## Development

```sh
scripts/dev.sh        # run the app with the Slint MCP server enabled
scripts/dev-live.sh   # run with live preview and MCP, without bundled translations
scripts/format.sh     # cargo fmt
scripts/lint.sh       # cargo clippy --fix
```

## Packaging

```sh
scripts/build-slint-appimage.sh
scripts/build-slint-windows.sh
scripts/build-slint-macos.sh
```

Windows builds use `cargo-xwin` and package an NSIS installer with `makensis`.
The macOS script removes the Linux-only winit patch from `Cargo.toml` before
building.

Updater releases are signed with `UPDATE_PRIVATE_KEY` and
`UPDATE_KEY_PASSWORD`.

## Releases

```sh
scripts/get-version.sh
scripts/generate-release-body.sh <version>
scripts/collect-build-artifacts.sh <windows|linux> <version>
scripts/merge-updater-json.sh <partials-dir>
```

## Containers

```sh
podman build -f Containerfile.linux -t rai-pal-builder:linux-local .
podman run --rm -v "$PWD:/app" -w /app rai-pal-builder:linux-local scripts/build-slint-appimage.sh

podman run --rm -v "$PWD:/app" -w /app ghcr.io/raicuparta/rai-pal-builder:windows-latest scripts/build-slint-windows.sh
```

## Steam Deck

```sh
scripts/deploy-deck.sh
```
