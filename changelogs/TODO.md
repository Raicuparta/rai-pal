# TODO

This file is here for me to track things I wanna fix for the next release. If there are things here on the main branch, I probably forgot to delete them. It should never have todos I only intend to complete in future releases, that's what GitHub issues are for.

For agents: if asked to complete todos, do the ones in the "Do now" section one by one, one commit each (simple commit message, no description). Each commit finishes the task for the parts that are 100% clear and updates this todo file by removing done items, and updating the report section below. Skip the "Do later" section completely. If something you come across isn't clear or requires some choices (only when it's really needed), you can finish some tasks partially, updating the todo list acordingly, and adding consise information about it to the notes section below (only for stuff that needs special attention). Anything you do should be moved to the Done section, with a little extra info on what exactly was done (but keep it short), to avoid repeated/reverted work in the future.

## Do now:

- macOS: the bundle declares the `rai-pal://` URL scheme, but incoming open-url events are not handled (registration only exists for Linux and Windows). fix that.
- Revisit the `serializable_struct`/`serializable_enum` proc macros. They still
  make sense because remote manifests, settings, and debug JSON need serde, but
  their names and bundled SQLite impls no longer reflect why most types use
  them. Splitting serde and SQLite behaviour into separate derives would be
  clearer.
- The Slint row structs still need an explicit field-to-field mapping because
  Slint cannot import arbitrary Rust structs. A small conversion macro could
  remove the boilerplate if it becomes annoying, but it should stay in the
  Slint crate.

## Do later:

- update the intro text.
- colored buttons probably need a "subtle" mode or something like in tauri.
- button size and icon spacing needs to be adjusted to match the tauri one better.
- maybe a bit too small on steam deck?

## Notes, questions, decisions, etc.

- macOS automatic updates are implemented, but release bundles are still ad-hoc signed and not notarized; first launch still needs the usual Gatekeeper bypass.

## Done

- Added macOS automatic updates: `darwin-aarch64`/`darwin-x86_64` platform entries, app bundle replacement and relaunch from the release zip, and signed updater partials in the release workflow.
- Bumped the local database version to 10 so the Slint app no longer shares database files with the Tauri app, whose `provider_commands` JSON format it can't read.
- Added a debug data option to every game mod's three-dot menu in the game details page.
- Removed leftover AI-written comments from the Slint app and new backend modules, keeping only notes that explain non-obvious behaviour.
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
- Bundled gettext translations. The `@tr` calls in the `.slint` files are the source of truth for English; `crates/frontend/translations` holds only the other languages, and there is no English catalog. `scripts/check-translations.sh` validates every catalog against the strings extracted from the UI.
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
- The `serializable_enum` proc macro now generates `ALL` value lists, so adding
  an enum variant automatically adds it to iteration without a separate copy.
  `FilterGroupKey` derives its persisted strings and iteration from strum.
- Games sort labels live in Slint next to the menu that renders them, while Rust keeps the `(GamesSortBy, descending)` order for index mapping and persistence.
- Centralized the current-thread Tokio runtime setup and shared `initials` in
  the Slint app.
- Removed the unused `maps` helper module and `byteorder` dependency.
- Moved filter group metadata into `rai-pal-backend::filter_views`: group order,
  typed group/value labels, display values, and per-value notes now come from
  core, and `crates/frontend/src/filters.rs` only maps those views into Slint.
  Filter callbacks, group labels, and expanded-filter settings use the typed
  `FilterGroupKey`; value labels and notes are typed enums.
- Moved all enum boundary conversions into `crates/frontend/src/convert.rs`, so
  `TabId`/`AppTab`, `ColorScheme`/`AppColorScheme`, provider ids, engine brands,
  filter ids, and provider actions each have a single conversion.
- Exposed provider actions as typed Slint items, and use the typed provider and
  engine ids in icons, colors, rows, and the sidebar instead of matching strings.
- Centralized language metadata in core (`AppLocale::ALL`, `code`, `native_name`)
  and build the language menu model from it, removing the duplicated list and
  hand-written index mapping in `app_menu.slint`.
- Replaced the thanks-page brand color string mapping with a typed `Brand` enum,
  and typed confirmation skip ids as persisted `SkipConfirmation` values.
- Routed the remaining host- and Slint-side strings through gettext. Rust no longer holds or reads translated text at all (see the typed messages entry below); the file dialog titles and filters are translated in Slint and passed to the native dialog as command arguments.
- Rust never produces user-visible sentences. Anything it needs to say travels as a typed value and Slint turns it into text: `MessageData`/`MessageContext` for toasts and inline popup errors, `ConfirmKind` for confirmation dialogs, `RefreshTaskData` for load-task labels, `DownloadItemKind` for download rows, and `MessageContext`-free enums/labels for the rest. Adding a message means adding an enum variant plus one branch in `translations.slint`, never a `format!` in Rust. Backend and dependency error text still reaches the UI as the untranslated `detail` payload.
- Derived mod row action visibility in Slint from the underlying mod facts
  (installed/outdated/compatibility plus which manifest operations exist)
  instead of carrying one Rust-computed boolean per action.
- Reduced the confirmation flow to typed pending actions instead of boxed
  callbacks, keyed action state by typed `ActionKey`, and replaced worker-thread
  sleeps for notifications/action resets with `slint::Timer::single_shot`.
- Buttons and sidebar actions share the spinner/status icon presentation through
  `action_status_icon.slint` (the 500 ms long-loading timer, success/error
  icons, and spinner rotation), and share tint contrast/hover/pressed math
  through the `Contrast` global in `contrast.slint`.
