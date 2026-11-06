# Slint Migration

The Tauri app and JavaScript frontend have been removed. Rai Pal is now a Slint
app with a Rust workspace at the repository root. This file tracks what still
needs an equivalent after that removal.

## Done

- Removed the Tauri app, frontend, Node tooling, and Tauri-only build config.
- Moved the Cargo workspace, `crates/core`, `crates/proc-macros`, and `crates/slint-app` under `crates/`.
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
- Bundled gettext translations. The `.po` catalogs in `crates/slint-app/translations` are
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

## Not done

- App update check/download flow. The release workflow no longer creates signed
  updater artifacts or `latest.json`; there is no updater endpoint or UI yet.
- Windows installer packaging. Releases currently contain a raw `.exe`; the old
  NSIS installer, installer icon/sidebar, and WinGet publishing are gone.
- Deep links (`rai-pal://`) registration and handling. Some UI text still
  mentions the add-mod-source deep link.
- Single-instance behavior.
- Window size/position persistence.
- App settings: reset settings and open logs folder.
- File logging. Tauri wrote logs to the app data logs folder; the Slint app logs
  to stdout/stderr only.
- Analytics events.
- Application/window icon wiring beyond the packaging assets used by AppImage
  and the macOS bundle.
- Mod details page and remaining mod UI parity.
