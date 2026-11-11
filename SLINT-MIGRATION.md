# Slint Migration

The Tauri app and JavaScript frontend have been removed. Rai Pal is now a Slint
app with a Rust workspace at the repository root. This file tracks what still
needs an equivalent after that removal.

## Done

- Removed the Tauri app, frontend, Node tooling, and Tauri-only build config.
- Moved the Cargo workspace, `crates/backend`, `crates/proc-macros`, and `crates/frontend` under `crates/`.
- Replaced npm scripts with shell scripts for dev, formatting, linting, builds,
  and release helpers.
- App shell, games/mods/thanks tabs, app menu, user menu.
- Central Slint theme (`theme.slint`) with t-shirt spacing/font scales,
  palette-driven dark/light/auto mode, and semantic colors; branding colors live
  in `branding.slint`.
- Shared component set: buttons and icon buttons, nav tabs, list rows, sidebars,
  popup menus, select dropdowns, modals, dividers, message states, and tooltips.
- SVG icon set exposed through a single `Icon` enum in `svg_icon.slint`,
  including provider, engine, and brand logos.
- Command layer: one `CommandHandler` stored in a thread-local and looked up at
  callback time through `on_command!`, so callback wiring no longer captures
  handler clones. UI state lives in exported Slint globals colocated with the
  components that use it.
- Typed action keys (`ActionKind` + `ActionKey`) and a Rust-side action-state
  store, so buttons and sidebar rows render loading/success/error states without
  per-callback state plumbing.
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
- Refresh all games, mod sources, and remote databases from the games and mods sidebars.
- Automatic refresh of the selected game's executable and installed mods when it
  is selected and whenever the window regains focus, matching the Tauri game
  page behavior.
- URL mod source management, including source toggles, removal, and add confirmation.
- Steam cache reset from the app menu.
- Steam shortcut creation from the app menu.
- Discord user menu with login and logout.
- Toast notifications for errors and provider command feedback, with dismissal
  and expiry driven from Rust; clicks on a toast no longer fall through to the
  content behind it.
- Confirmation dialogs with a persisted skip-confirmation setting.
- Thanks page content, including Patreon leaderboard and donation links.
- Manually adding games: executable picker, drag and drop, directory scanning with progress and confirmation, saved directory management, and removing manually added games.
- Game thumbnail visibility setting.
- Download progress tracking and status menu, driven by shared install-step generation.
- Loading, empty, and error states for the games list, game mods, and mods loading, including a startup refresh and data-query error notifications.
- Debug data for games and mods, serialized on demand when the debug popup opens
  instead of being precomputed for every row.
- Deep links (`rai-pal://`) registration and handling on Linux and Windows,
  including `add-mod-source`, `install-mod`, `uninstall-mod`, and `run-mod`.
- Single-instance behavior, including focusing the running window and forwarding
  command-line arguments (deep links) to it.
- Window size, position, and maximized-state persistence in the app settings,
  including Gamescope/Steam Deck scale-factor handling and monitor size clamping.
- App menu actions to reset Rai Pal settings and open the logs folder.
- File logging to the app data logs folder, with stdout/stderr mirroring and
  basic size-based rotation.
- Analytics events for app start, mod install/update/uninstall/run, provider
  commands, sign-in, and error notifications.
- Window icon wired from the shared graphics assets at runtime.
- Platform release builds: AppImage on Linux, NSIS installer on Windows
  (including installer icon/sidebar, Start menu and desktop shortcuts, uninstall
  metadata, and WinGet publishing), and a macOS app zip built by CI.
- App update check/download flow on Linux and Windows. The app checks signed
  `latest.json` every two minutes, shows the changelog, verifies Minisign
  signatures, replaces the running AppImage on Linux, and runs the NSIS
  installer silently on Windows. Release publishing uploads `latest.json` and
  updater signatures.

## Not done

- macOS: the bundle declares the `rai-pal://` URL scheme, but incoming open-url
  events are not handled (registration only exists for Linux and Windows).
- macOS: no automatic updates. The updater only has platform entries and
  installers for Linux x86_64 (AppImage) and Windows x86_64 (NSIS).

## Core/Slint cleanup

- Removed the Tauri-era game JSON round-trip: game rows no longer carry a
  serialized `Game`. Selecting a row reloads it from the local database, and
  the details debug popup serializes the selected game on demand.
- Kept Slint row structs and display formatting on the Slint side. Frontend data
  types use a `Data` suffix: `GameRowData` is populated by the `ReleaseDate` and
  formatting helpers in `crates/frontend/src/model.rs`, and fetch workers send
  the typed `Game` back to the UI thread instead of building preformatted data.
- Backend names now distinguish the manifest from the per-game entry: `Mod` is
  the manifest (`mods::manifest`), `GameMod` is a manifest plus its `ModInfo` and
  remote-config state (`mods::game_mods`), `GameModGroups` holds classified
  entries, and `ModGroup` holds mod-tab groups (`mods::mod_groups`). The Slint app
  maps those domain results into `GameModData` rows and derives the UI action
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
- Kept the games sort option labels in the Slint app's menu, with the option
  order and index mapping in `SORT_OPTIONS` in
  `crates/frontend/src/commands/settings.rs`.
- Centralized the current-thread Tokio runtime setup and shared `initials` in
  the Slint app.
- Removed the unused `maps` helper module and `byteorder` dependency.
- Removed the single-field `ProviderActionItem` Slint struct; provider actions
  are now exposed as plain strings.

## Further cleanup candidates

These are plausible simplifications that need a product or API decision before
changing.

- Move filter group metadata (labels, translation keys, display values, order,
  and notes) from `crates/frontend/src/filters.rs` and
  `crates/frontend/ui/translations.slint` into core, so the UI only maps core
  filter definitions into Slint models. The value lists and mutation logic are
  already shared, but each group is currently described in both the builder and
  the `Tr.filter-property`/`Tr.filter-value`/`Tr.filter-value-note` key maps,
  and the note lookup special-cases `"GameMaker"` and `"Xbox"` by display string.
- Replace the duplicated `TabId`/`AppTab` and `ColorScheme`/`AppColorScheme`
  pairs. Slint enums cannot directly use the core types, so this needs a
  decision between generated enum bindings, integer codes, or keeping the
  explicit boundary conversion. Note that the conversion is currently written
  twice (`app_tab` in `app.rs` and `core_tab` in `commands/settings.rs`).
- Make `AppSettings.expanded_filters` a set of `FilterGroupKey` values instead
  of strings, with a serialization format that stays compatible with existing
  settings files. This would remove string parsing and casts in the UI.
- Expose provider actions as typed items (action plus label/icon metadata)
  instead of strings that are parsed back into `ProviderCommandAction` when the
  callback fires and matched by string in `game_sidebar.slint` for icons and
  labels. This was intentionally left plain when removing `ProviderActionItem`.
- Stop dispatching provider ids, engine brands, and provider command actions by
  string in the UI. `provider_icon.slint`, `engine_icon.slint`,
  `branding.slint`, and `Tr.provider-command` match the same literals as the
  core enums; typed enums (or icon/color data sent from Rust with the row) would
  remove the repeated string chains. The thanks-page brand colors in
  `branding.slint` (`GitHub`, `Patreon`, `Paypal`) would still need their own
  mapping.
- Keep the UI-facing metadata lists in one place: the language list (codes and
  native names) exists in both `i18n.rs` and `app_menu.slint`, including a
  hand-written index mapping; the sort labels exist in `sort_menu.slint` while
  the corresponding order lives in `SORT_OPTIONS` in
  `commands/settings.rs`.
- Revisit the `serializable_struct`/`serializable_enum` proc macros. They still
  make sense because remote manifests, settings, and debug JSON need serde, but
  their names and bundled SQLite impls no longer reflect why most types use
  them. Splitting serde and SQLite behaviour into separate derives would be
  clearer.
- The Slint row structs still need an explicit field-to-field mapping because
  Slint cannot import arbitrary Rust structs. A small conversion macro could
  remove the boilerplate if it becomes annoying, but it should stay in the
  Slint crate.
- The mod row carries one boolean per possible action (`can_install`,
  `can_update`, `can_run`, `can_edit_config`, ...) derived in `commands/mods.rs`
  from the manifest and install state. Exposing the few underlying facts
  (installed/outdated/compatible plus which manifest operations exist) and
  letting the Slint side decide which buttons to show would shrink the row
  struct every time an action changes.
- The confirmation flow stores a boxed `FnOnce` plus UI-supplied text and labels
  in Rust, and builds `skip-confirm-` string ids. It could be reduced to a typed
  pending action (with Slint owning the dialog copy), which would address the
  "confirmation.rs seems overkill" note in `changelogs/TODO.md`.
- Notification dismissal and action-state resets each spawn a thread that
  sleeps and then hops back to the UI thread. A shared timer (or Slint `Timer`)
  would avoid one OS thread per notification/action.
- Action state is stored in a `HashMap<String, ActionState>` keyed by
  `format!("{:?}:{}", kind, target)`; if Slint's generated enums can be hashed,
  keying by `ActionKey` directly would remove the formatting and parsing.
- Button and sidebar action duplicate the same spinner/status presentation
  (500 ms long-loading timer, success/error icons, spinner angle) and the same
  contrast/hover/pressed color math in `base_button.slint` and
  `sidebar_action.slint`. Extracting these into shared components or `Theme`
  helpers would keep the states consistent.
- Some user-visible strings still bypass gettext: the manual-game file dialog
  titles and refresh task names are built in Rust, and a few Slint labels are
  plain literals (`"Only"` in `filter_group.slint`, `"Remove"` in
  `url_mod_sources_popup.slint`, the about popup lines, and the `"About..."`
  menu item). Routing them through `@tr`/`Tr` would finish translation coverage.
- Button parity with the Tauri UI is still open: colored buttons need a
  "subtle" variant, and sizes/icon spacing still need tuning (see
  `changelogs/TODO.md`). This is a design decision before it is a refactor.
