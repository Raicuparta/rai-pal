# TODO

This file is here for me to track things I wanna fix for the next release. If there are things here on the main branch, I probably forgot to delete them. It should never have todos I only intend to complete in future releases, that's what GitHub issues are for.

For agents: if asked to complete todos, do the ones in the "Do now" section one by one, one commit each (simple commit message, no description). Each commit finishes the task for the parts that are 100% clear and updates this todo file by removing done items, and updating the report section below. Skip the "Do later" section completely. If something you come across isn't clear or requires some choices (only when it's really needed), you can finish some tasks partially, updating the todo list acordingly, and adding consise information about it to the notes section below (only for stuff that needs special attention). Anything you do should be moved to the Done section, with a little extra info on what exactly was done (but keep it short), to avoid repeated/reverted work in the future.

## Do now

- accessibility! i tihnk we're mostly ok but we need a full pass on the entire slint app. one thing that's currently broken is there is no outline at all for when pressing tabto cycle between elements. they seem to work but no visible way to tell. then, i think simply pressing tab doesn't let us escape some parts. i seem to usually be stuck in the sidebar? not sure. have a look and see what we can improve. i'd prefer not to have to mess with every single style and do something global, but maybe not possible? give me a report and tell me what we should do, but feel free to already fix the obvious stuff in commits.
- add missing tooltips on stuff that have no text to explain. the two ones i know are games search input, and the sort order button. for the rest you probably need to confirm with me whether they actually need a tooltip, unless it's really obvious. somewhat related to accessibility since tooltip text should be used as accessible labels for stuff that has no text.
- make the text in all popup notifications selectable.

## Do later

- "Manual" should be renamed to something but dunno what.

## Notes, questions, decisions, etc

- Minimized/unfocused window: the visible range stays the same, so visible thumbnails and game fetches still happen. The easy version would be to treat a minimized window as an empty visible range (nothing gets fetched while minimized). Unfocused windows are harder to infer reliably and probably not worth it.

## Done

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
