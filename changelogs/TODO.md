# TODO

This file is here for me to track things I wanna fix for the next release. If there are things here on the main branch, I probably forgot to delete them. It should never have todos I only intend to complete in future releases, that's what GitHub issues are for.

For agents: if asked to complete todos, do the ones in the "Do now" section one by one, one commit each (simple commit message, no description). Each commit finishes the task for the parts that are 100% clear and updates this todo file by removing done items, and updating the report section below. Skip the "Do later" section completely. If something you come across isn't clear or requires some choices (only when it's really needed), you can finish some tasks partially, updating the todo list acordingly, and adding consise information about it to the notes section below (only for stuff that needs special attention). Anything you do should be moved to the Done section, with a little extra info on what exactly was done (but keep it short), to avoid repeated/reverted work in the future.

## Do now

## Do later

## Notes, questions, decisions, etc

- `DownloadItemData.phase` is sent to the UI but never read there. Failed downloads show their last percentage forever. Either show the phase in the downloads menu, or drop the field.
- `build_filter_views` is a plain list of groups, so a new `FilterGroupKey` variant compiles fine but silently never shows up in the UI until it's added there too. Every other spot (apply/reset/convert/Slint enum) is compiler-checked.
- The Xbox provider logs a warning for every package registry key it can't read, which is most of them. Deep nesting also makes it hard to read; a helper returning `Option<Game>` would fix both.

## Done

- Added the missing translation for the "Data" section label in the game details sidebar.
- Removed the write-only `AppSettings.selected-language` Slint property; the selected language index is the only state now.
- Simplified the thumbnail cache path helper.
- Removed unused `Modal` header action props and the no-op `PopupMenuItem.checkable` property.
- Download items now hold their `InstallStep` instead of a formatted `"mod:step"` id string, which also dropped the unused `DownloadItemData.id`.
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
