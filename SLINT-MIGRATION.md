# Slint Migration

The Tauri app and JavaScript frontend have been removed. Rai Pal is now a Slint
app with a Rust workspace at the repository root. This file tracks what still
needs an equivalent after that removal.

## Done

- Removed the Tauri app, frontend, Node tooling, and Tauri-only build config.
- Moved the Cargo workspace, `crates/backend`, `crates/proc-macros`, and `crates/frontend` under `crates/`.
- Replaced npm scripts with shell scripts for dev, formatting, linting, builds,
  and release helpers.
- App shell, games/mods/thanks tabs, app menu.
- Language selection and light/dark/auto color scheme.
- Games list with search, sorting, release date, tags, provider and engine styling.
- Game filters: toggle, lock, only, reset group, reset all, expanded state persistence.
- Game details page and game mod sections: potential, installed, available, hidden, incompatible.
- Game actions: start/run, provider commands, open game/mods/data/Wine folders, refresh game, debug data.
- Mod actions: install, update, reinstall, uninstall, uninstall all, run, configure, open folder, download config.
- Mods tab: mod families, mod rows, run/install/website actions, open local mods folder.
- About popup.
- Bundled gettext translations. The `.po` catalogs in `crates/frontend/translations` are
  the source of truth now that the TypeScript localizations are gone.
- Shared core filter mutation and provider-action selection logic.
- Central Slint theme, branding colors, sidebar/section components, global mod actions.
- Refresh all games, mod sources, and remote databases from the games and mods sidebars.
- URL mod source management, including source toggles, removal, and add confirmation.
- Steam cache reset from the app menu.
- Steam shortcut creation from the app menu.
- Discord user menu with login and logout.
- Toast notifications for errors and provider command feedback.
- Confirmation dialogs with a persisted skip-confirmation setting.
- Thanks page content, including Patreon leaderboard and donation links.
- Manually adding games: executable picker, drag and drop, directory scanning with progress and confirmation, saved directory management, and removing manually added games.
- Game thumbnail visibility setting.
- Download progress tracking and status menu, driven by shared install-step generation.
- Loading, empty, and error states for the games list, game mods, and mods loading, including a startup refresh and data-query error notifications.
- Deep links (`rai-pal://`) registration and handling on Linux and Windows,
  including `add-mod-source`, `install-mod`, `uninstall-mod`, and `run-mod`.
  macOS bundles declare the URL scheme, but incoming open-url events are not wired.
- Single-instance behavior, including focusing the running window and forwarding
  command-line arguments (deep links) to it.
- Window size, position, and maximized-state persistence in the app settings.
- App menu actions to reset Rai Pal settings and open the logs folder.
- File logging to the app data logs folder, with stdout/stderr mirroring and
  basic size-based rotation.
- Analytics events for app start, mod install/update/uninstall/run, provider
  commands, sign-in, and error notifications.
- Window icon wired from the shared graphics assets at runtime.
- Windows NSIS installer packaging, including installer icon/sidebar, Start menu
  and desktop shortcuts, uninstall metadata, and WinGet release publishing.
- App update check/download flow on Linux and Windows. The app checks signed
  `latest.json` every two minutes, shows the changelog, verifies Minisign
  signatures, replaces the running AppImage on Linux, and runs the NSIS
  installer silently on Windows. Release publishing uploads `latest.json` and
  updater signatures.

## Not done

Nothing currently.

## Core/Slint cleanup

- Removed the Tauri-era game JSON round-trip: game rows no longer carry a
  serialized `Game`. Selecting a row reloads it from the local database, and
  the details debug popup still shows pretty JSON from the selected game.
- Kept Slint row structs and display formatting on the Slint side. Frontend data
  types use a `View` suffix: `GameRowView` is populated by the `ReleaseDate` and
  formatting helpers in `crates/frontend/src/model.rs`, and fetch workers send
  the typed `Game` back to the UI thread instead of building preformatted data.
- Backend names now distinguish the manifest from the per-game entry: `Mod` is
  the manifest (`mods::manifest`), `GameMod` is a manifest plus its `ModInfo` and
  remote-config state (`mods::game_mods`), `GameModGroups` holds classified
  entries, and `ModGroup` holds mod-tab groups (`mods::mod_groups`). The Slint app
  maps those domain results into `GameModView` rows and derives the UI action
  flags there.
- Moved mod display helpers (`release_date_display`, `engine_text`,
  `download_version`) and standalone action checks into the Slint app, while
  keeping the underlying `Mod` manifest fields typed and public in core.
- Replaced awkward core tuples with named domain types: `GameId` for query
  results, `GameEnvironment` for launch environment variables plus contributing
  mod IDs, and `InstallStep`/`InstallStepKind` instead of preformatted step
  names.
- Added `ALL` value lists for provider, engine, tag, OS, architecture, Unity
  backend, and install-state enums, removing copies in the Slint filters and
  refresh code.
- Kept the games sort option order in the Slint app, where the menu lives.
- Centralized the current-thread Tokio runtime setup and shared `initials` in
  the Slint app.
- Removed the unused `maps` helper module and `byteorder` dependency.
- Removed the single-field `ProviderActionItem` Slint struct; provider actions
  are now exposed as plain strings.

## Further cleanup candidates

These are plausible simplifications that need a product or API decision before
changing.

- Move filter group metadata (labels, translation keys, display values, and
  notes) from `crates/frontend/src/filters.rs` into core, so the UI only maps
  core filter definitions into Slint models. The value lists and mutation logic
  are already shared, but the labels are still duplicated and can drift from
  the core filter groups.
- Replace the duplicated `TabId`/`AppTab` and `ColorScheme`/`AppColorScheme`
  pairs. Slint enums cannot directly use the core types, so this needs a
  decision between generated enum bindings, integer codes, or keeping the
  explicit boundary conversion.
- Make `AppSettings.expanded_filters` a set of `FilterGroupKey` values instead
  of strings, with a serialization format that stays compatible with existing
  settings files. This would remove string parsing and casts in the UI.
- Expose provider actions as typed items (action plus label/menu metadata)
  instead of strings that are parsed back into `ProviderCommandAction` when the
  callback fires. This was intentionally left plain when removing
  `ProviderActionItem`.
- Revisit the `serializable_struct`/`serializable_enum` proc macros. They still
  make sense because remote manifests, settings, and debug JSON need serde, but
  their names and bundled SQLite impls no longer reflect why most types use
  them. Splitting serde and SQLite behaviour into separate derives would be
  clearer.
- The Slint row structs still need an explicit field-to-field mapping because
  Slint cannot import arbitrary Rust structs. A small conversion macro could
  remove the boilerplate if it becomes annoying, but it should stay in the
  Slint crate.
- The mods-tab row's `debug_json` serializes every mod when the mods tab loads
  even though the debug popup is rarely opened. Making it lazy would avoid that
  work, at the cost of an extra callback and per-mod lookup.

## Images

### Done

- Metadata and image fetching are separate; rows render before thumbnails arrive.
- Dedicated image worker pool with URL coalescing, visibility priority, and cancellable downloads.
- Visible rows wait 100 ms before fetching; cached visible rows skip the wait.
- Disk cache only: compressed bytes keyed by URL + 200x120 + cache version, stored in
  `~/.local/share/rai-pal/thumbnail-cache`.
- Disk cache capped at 200 MB, pruned at startup to ~180 MB.
- GIF decoding enabled, so GIF thumbnails cache normally. Only the first frame is displayed.
- Failed requests retry after 1 second while the row is still wanted.
- Temporary debug logging added during investigation and removed.

### CPU profiling

Scrolling CPU is dominated by rendering, not image fetching/decoding:

```text
80.1%  llvmpipe-* threads       software OpenGL rasterizer
19.7%  Rai-Pal                  app/main-thread work
 0.1%  reqwest/async-io         network-related threads
```

Main hotspot: Mesa `llvmpipe` software rasterization (`util_fill_rect`, shading, z/stencil clears).

### Remaining

1. Investigate why OpenGL falls back to `llvmpipe`; fixing that is the biggest CPU win.
2. If software rendering remains, reduce repaint frequency/area during scrolling.
3. Optional: memory cache if decoding becomes a profile hotspot.
4. Optional: first-frame-only GIF downloads or resized itch GIF URLs to reduce first-view bandwidth.
5. Optional: reduce row reset churn.
