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

## Not done

- Thanks page content.
- Add game, scan/add game directory, remove manually added game.
- App settings beyond language/color scheme: hide thumbnails, skip confirmations, etc.
- Steam cache reset and Steam shortcut tools.
- User login/logout and user menu.
- App update check/download flow.
- Download progress UI.
- Toast/error/notification UI.
- Confirmation dialogs.
- Mod details page and remaining mod UI parity.
- Loading, empty, and error states parity.
- Window size/position persistence, single-instance behavior, deep links.
- Analytics events.
