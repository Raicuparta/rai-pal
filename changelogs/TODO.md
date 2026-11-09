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

- Slint/Rust bridge is now feature-scoped instead of one state file. `AppSettings` is the only shared settings global; `GamesList` lives in games-page, `GamesQuery` in games-sidebar, `GameSidebarState` in game-sidebar, `GameMods` in game-details-page, `Refresh` in refresh-button, and each modal/single-feature state lives with its owning component. `state.slint` was removed.
- Modal open state is local to MainWindow when only the UI controls it (about, URL sources, steam cache/shortcut). Modals that Rust also opens/closes keep an open property colocated with the modal (add-game, confirmation, confirm mod source, update); Slint does not allow handling a global callback in UI code, so host-triggered open/close needs exposed state.
- Replaced operation bool soup (`busy`, `installing`, `cache-running/success`, `shortcut-running/success`, `confirm-loading/saving`, `toggling`) with the existing generic `ActionState` enum plus an error string where needed. `Refresh.refreshing` stays a bool because it represents a long-lived multi-task refresh indicator, not a request lifecycle.
- Added `ActionKind` + `ActionKey` to replace magic action-id strings between Slint and Rust. Rust builds them through `action_state::key(...)`; action buttons/sidebar actions take a typed key.
- Callbacks remain centralized in `actions.slint` for now: they are the native host command boundary and all handlers are registered centrally in `app.rs`. UI-only child->parent callbacks stay local (`about-clicked`, `add-game-clicked`, `open-url-mod-sources-clicked`, steam tool callbacks). If `actions.slint` grows further, split by feature/owner.
- Type-safety notes left for later: provider actions still cross Slint as strings and are parsed back into `ProviderCommandAction`; a mirrored/generated Slint enum would avoid that but risks duplicating the core enum. Filter group/value keys are dynamic by design. Action status IDs are now typed.
- Cleaned up unused ids/imports and one-use props (`ProviderIcon` size, `Modal` content-spacing/close-tooltip/header-action-enabled, `NavButton` top/bottom padding, `SelectMenu` enabled, `SidebarAction` state).
- Dark mode: when `Palette.color-scheme` is dark/light, `Theme.dark` uses that directly. When it is Unknown (auto), Slint's resolved system scheme (`SlintInternal.color-scheme`) is not exposed to app code, so `Theme.dark` falls back to the resolved `Palette.background` lightness only in that case.
- Runtime-checked via Slint MCP: tabs, game details, app menu, URL sources popup, add-game popup, steam cache popup.
