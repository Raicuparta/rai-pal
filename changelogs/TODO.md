# TODO

This file is here for me to track things I wanna fix for the next release. If there are things here on the main branch, I probably forgot to delete them. It should never have todos I only intend to complete in future releases, that's what GitHub issues are for.

For agents: if asked to complete todos, do these one by one, one commit each (simple commit message, no description). Each commit finishes the task for the parts that are 100% clear and updates this todo file by removing done items, and updating the report section below. If something you come across isn't clear or requires some choices (only when it's really needed), you can finish some tasks partially, updating the todo list acordingly, and adding that to the report.

# Do these now:

- Filters scrolling still always scrolls entire sidebar.
- debug data currently missing a bunch of data. adding more buttons for game-mods maybe would be good.
- update the intro text.
- colored buttons probably need a "subtle" mode or something like in tauri.
- button size and icon spacing needs to be adjusted to match the tauri one better.
- process name is still rai-pal-slint on appimage at least.
- some modals have clipped titles (add to steam as example, i saw on deck)
- maybe a bit too small on steam deck?

# Report:

- Refactored the Slint/Rust bridge from root MainWindow properties/callbacks to domain globals in `ui/state.slint` and `ui/actions.slint` (`AppState`, `UserState`, `Refresh`, `GamesState`, `SelectedGameState`, `ModsState`, `DownloadsState`, `NotificationsState`, `ThanksState`, `DebugState`, `AddGameState`, `ModSourcesState`, `SteamToolsState`, `UpdateState`, `ConfirmationState`, `AppActions`, `GamesActions`, `DialogActions`). Components talk to these directly; `main-window.slint` went from 726 to ~200 lines.
- Removed the duplicated `has-*` mod flags; `GameDetailsPage` derives them from list lengths.
- Popups/overlays now own their open/close state and actions instead of being wired through `MainWindow`. Reusable leaf widgets (`GameRow`, `Button`, `Modal`, etc.) keep props.
- Cleaned up unused ids/imports and one-use props (`ProviderIcon` size, `Modal` content-spacing/close-tooltip/header-action-enabled, `NavButton` top/bottom padding, `SelectMenu` enabled, `SidebarAction` state).
- Dark mode: when `Palette.color-scheme` is dark/light, `Theme.dark` uses that directly. When it is Unknown (auto), Slint's resolved system scheme (`SlintInternal.color-scheme`) is not exposed to app code, so `Theme.dark` falls back to the resolved `Palette.background` lightness only in that case.
- Rust still has repetitive `window.global::<crate::...>()` call sites; left as-is to keep this change reviewable.
- Single-use state globals now live next to their owning component (`UserState`, `ModsState`, `DownloadsState`, `NotificationsState`, `ThanksState`, `DebugState`, `AddGameState`, `UpdateState`, `ConfirmationState`). Shared/cross-feature state (`AppState`, `Refresh`, `GamesState`, `SelectedGameState`, `ModSourcesState`, `SteamToolsState`) and all action globals stay in `state.slint`/`actions.slint`.
