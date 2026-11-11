# Scripts

## Development

```sh
scripts/dev.sh        # run the app with the Slint MCP server enabled
scripts/dev-live.sh   # run with live preview and MCP, without bundled translations
scripts/format.sh     # cargo fmt
scripts/lint.sh       # cargo clippy --fix
```

## Checks

```sh
scripts/check-translations.sh  # validate every catalog against the @tr strings in the UI
```

Translation checks need `slint-tr-extractor`; `scripts/check-translations.sh --install-tool` installs the version that matches the Slint version in `crates/frontend/Cargo.toml`. The check is not part of the build, so normal builds never require it; CI runs it from `.github/workflows/check.yml`.

## Packaging

Bundles are written to `target/bundles/<type>/`:

| Script                  | Output                              |
| ----------------------- | ----------------------------------- |
| `build-appimage.sh`     | `target/bundles/appimage/`          |
| `build-windows.sh`      | `target/bundles/nsis/`              |
| `build-macos.sh`        | `target/bundles/macos/`             |

Windows builds use `cargo-xwin` and package an NSIS installer with `makensis`.
The macOS script removes the Linux-only winit patch from `Cargo.toml` before
building.

Updater releases are signed with `UPDATE_PRIVATE_KEY` and
`UPDATE_KEY_PASSWORD`.

`collect-build-artifacts.sh` publishes a main download (`RaiPal.exe` is the
NSIS installer, `RaiPal.AppImage` is the AppImage, and `RaiPal.macos.zip` is the
macOS app bundle) and a separate updater asset in the format the Tauri v1
updater used (`updater-windows.zip`, `updater-linux.tar.gz`,
`updater-macos.tar.gz`). Keeping them separate means the GitHub release download
counts updater downloads independently.

The Tauri v1 updater is built without deflate support, so the Windows updater
zip must be stored uncompressed (`zip -0`). macOS updater assets must be
`.app.tar.gz` with the `.app` bundle at the archive root.

## Releases

```sh
scripts/get-version.sh
scripts/generate-release-body.sh <version>
scripts/collect-build-artifacts.sh <windows|linux|macos> <version>
scripts/merge-updater-json.sh <partials-dir> [existing-latest.json]
```

`merge-updater-json.sh` builds `latest.json` from the platform partials, and from an existing `latest.json` when given one: platforms present in both are updated, platforms only in the existing file are kept, and the changelog becomes the updater notes. Running it with no partials only refreshes the notes.

## Containers

```sh
podman build -f Containerfile.linux -t rai-pal-builder:linux-local .
podman run --rm -v "$PWD:/app" -w /app rai-pal-builder:linux-local scripts/build-appimage.sh

podman run --rm -v "$PWD:/app" -w /app ghcr.io/raicuparta/rai-pal-builder:windows-latest scripts/build-windows.sh
```

## Steam Deck

```sh
scripts/deploy-deck.sh
```
