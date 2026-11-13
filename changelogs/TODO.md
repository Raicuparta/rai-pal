# TODO

This file is here for me to track things I wanna fix for the next release. If there are things here on the main branch, I probably forgot to delete them. It should never have todos I only intend to complete in future releases, that's what GitHub issues are for.

For agents: if asked to complete todos, do the ones in the "Do now" section one by one, one commit each (simple commit message, no description). Each commit finishes the task for the parts that are 100% clear and updates this todo file by removing done items, and updating the report section below. Skip the "Do later" section completely. If something you come across isn't clear or requires some choices (only when it's really needed), you can finish some tasks partially, updating the todo list acordingly, and adding consise information about it to the notes section below (only for stuff that needs special attention). Anything you do should be moved to the Done section, with a little extra info on what exactly was done (but keep it short), to avoid repeated/reverted work in the future.

## Do now

- accessibility follow-up: decide whether game rows should use arrow-key roving focus instead of one tab stop per visible row (ListView only instantiates visible rows, so tab cannot auto-scroll to offscreen games without custom scroll logic), preserve focus after a filter toggle (model rebuild currently drops it), and find a way to skip the long sidebar filter list when tabbing. Verify with a real screen reader.

## Do later

- "Manual" should be renamed to something but dunno what.

## Notes, questions, decisions, etc

- Accessibility report: tab order does work, but the sidebar exposes every filter value plus its hover-only actions as tab stops (roughly 70+ before reaching the main content), which makes it feel like focus is stuck there. Several custom clickable elements had no keyboard focus at all, and the filter rows' "Only"/lock buttons were still tab stops at `opacity: 0`, so focus could seem to disappear. Fixed: visible focus ring on buttons/icon buttons, nav tabs, select menus, download menu, external links, game rows and mod family headers; custom controls now activate with Space/Return and have accessible labels; filter row hover-only actions now appear when the row is focused and while tabbing through them. Zero-sized focus scopes avoid intercepting pointer clicks. Remaining: roving focus for game rows (ListView virtualization makes tab auto-scroll hard), preserving focus after filter toggles (models are rebuilt), a way to skip the sidebar filter list, and real screen reader testing.
- Escape key now closes modals that have a close button, closes popup menus like a click outside, clears a clearable text field, and goes back from the game details page. Modal/menu escape handling lives in those shared components, so new popups inherit it.
- Minimized/unfocused window: the visible range stays the same, so visible thumbnails and game fetches still happen. The easy version would be to treat a minimized window as an empty visible range (nothing gets fetched while minimized). Unfocused windows are harder to infer reliably and probably not worth it.

## Done

- Escape key handling: closes closable modals, closes popup menus, clears clearable text fields, and backs out of the game details page.

- Popup notification text is now a selectable `StyledText`, so errors and other messages can be copied.

- Added tooltips (also used as accessible labels) to the games search input and the sort button, and made the shared text field's clear button tooltip generic. The other icon-only controls already had tooltips; any further tooltip candidates aren't obvious and need confirmation.

- Accessibility: added visible keyboard focus rings to buttons, nav tabs, select menus, download menu, external links, game rows and mod family headers, and gave the custom controls Space/Return activation with accessible labels.

- The "update available" modal uses the narrow size again.

- Mod sources modal: now uses the wide size, the deep link is shown on its own selectable line (new `CopyableText` component) instead of making the whole description selectable, and all its inputs use a new shared `TextField` extracted from the games search input (which now uses it too). The URL input fills the remaining width so the Add source button no longer shifts while typing, and the Add source button is tinted like the other secondary buttons.

- Thumbnail loading: downloads and decodes are now separate stages (6 download workers, 2 decode workers), so more thumbnails are fetched in parallel without a decode/CPU spike, disk cache hits skip the download workers entirely, and a small LRU of decoded thumbnails in memory makes scrolling back to already-seen rows instant. The 100ms visible delay is unchanged.
- The repeating `Failed to request image: builder error` spam came from itch games with an empty cover URL: itch stored `Some("")` instead of no thumbnail, and failed thumbnails were retried immediately (plus a 1s sleep) forever. itch now stores `None` for empty covers, empty URLs are skipped before fetching, a failed thumbnail is not retried until its row leaves and re-enters the visible margin, and the error log names the URL. Manual games have no thumbnail URL, so they were never the cause.

- Added the missing translation for the "Data" section label in the game details sidebar.
- Removed the write-only `AppSettings.selected-language` Slint property; the selected language index is the only state now.
- Simplified the thumbnail cache path helper.
- Removed unused `Modal` header action props and the no-op `PopupMenuItem.checkable` property.
- Download items now hold their `InstallStep` instead of a formatted `"mod:step"` id string, which also dropped the unused `DownloadItemData.id`.
- The downloads menu now shows failed downloads as a failure instead of a stuck percentage, and finished ones with a check.
- Flattened the Xbox provider: one function reads a package with `?`, the caller logs and skips it. Only a missing executable config (i.e. not a game) is silent, so non-game packages no longer spam warnings.
- `build_filter_views` now builds groups from `FilterGroupKey` iteration and an exhaustive match, so a new group can't be silently missing (and the enum order shows in the UI).
- Documented why `run_query` bumps the fetch generation twice.
- Search text is no longer saved in settings (it was also written on every keystroke).
- Action states are kept in a thread-local like the other UI-thread-only state, instead of behind a `Mutex`.
- Added `runtime::with_ui_window` to replace the repeated "invoke on UI event loop, upgrade the weak window, ignore if gone" boilerplate.
- Settings reset and startup now share `apply_settings_to_ui`, so a new setting only needs to be applied in one place.
- `apply_initial_state` now derives the initial query/filters/tab from the saved settings instead of taking 5 precomputed arguments.
- The selected game state is a plain `RefCell` instead of an `Arc<Mutex>`, since it is only ever touched on the UI thread.
- `make_row` now gets its defaults from `reset_row` instead of listing every field twice.
- Dropped unused UI data: `GameModData.family` and four unused `Theme` popup size constants.
- Removed the unused dev-mode command handler from the user socket (the dev socket scripts are gone).
- Restored camelCase field names for the saved games filter, so settings written by 1.0.0 still deserialize (the old app wrote `unityBackends`, etc.).
- Removed the redundant selected-game checks before applying refreshed mod lists.
- Added a shared `CommandHandler::spawn_action` and reused it (plus `runtime::block_on`) for all game/mod actions, removing the repeated thread + runtime + action-state boilerplate.
- Merged the three duplicated URL mod source actions into one shared helper in `commands/mod_sources.rs`.
- Replaced the hardcoded red dialog error texts with a shared `ErrorMessage` component using `Theme.status-error`.
