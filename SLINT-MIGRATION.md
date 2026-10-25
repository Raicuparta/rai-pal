# Slint Migration

Tracks feature parity between the Tauri app (`/frontend` + `/backend/tauri-app`)
and the Slint app (`/backend/slint-app`). Move items between lists as they are
implemented.

## Done

- App shell, games/mods/thanks tabs, app menu.
- Language selection and light/dark/auto color scheme.
- Games list with search, sorting, release date, tags, provider and engine styling.
- Game filters: toggle, lock, only, reset group, reset all, expanded state persistence.
- Game details page and game mod sections: potential, installed, available, hidden, incompatible.
- Game actions: start/run, provider commands, open game/mods/data/Wine folders, refresh game, debug data.
- Mod actions: install, update, reinstall, uninstall, uninstall all, run, configure, open folder, download config.
- Mods tab: mod families, mod rows, run/install/website actions, open local mods folder.
- About popup.
- Bundled translations and localization pipeline.
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
- Manually adding games: executable picker, directory scanning with progress and confirmation, saved directory management, and removing manually added games.
- Game thumbnail visibility setting.
- Download progress tracking and status menu, driven by shared install-step generation.
- Loading, empty, and error states for the games list, game mods, and mods loading, including a startup refresh and data-query error notifications.

## Not done
- App settings: reset settings and open logs folder.
- App update check/download flow.
- Mod details page and remaining mod UI parity.
- Window size/position persistence, single-instance behavior, deep links.
- Analytics events.
