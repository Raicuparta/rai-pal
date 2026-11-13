# TODO

This file is here for me to track things I wanna fix for the next release. If there are things here on the main branch, I probably forgot to delete them. It should never have todos I only intend to complete in future releases, that's what GitHub issues are for.

For agents: if asked to complete todos, do the ones in the "Do now" section one by one, one commit each (simple commit message, no description). Each commit finishes the task for the parts that are 100% clear and updates this todo file by removing done items, and updating the report section below. Skip the "Do later" section completely. If something you come across isn't clear or requires some choices (only when it's really needed), you can finish some tasks partially, updating the todo list acordingly, and adding consise information about it to the notes section below (only for stuff that needs special attention). Anything you do should be moved to the Done section, with a little extra info on what exactly was done (but keep it short), to avoid repeated/reverted work in the future.

## Do now

## Do later

## Notes, questions, decisions, etc

## Done

- Added the missing translation for the "Data" section label in the game details sidebar.
- Removed the write-only `AppSettings.selected-language` Slint property; the selected language index is the only state now.
- Removed the unused dev-mode command handler from the user socket (the dev socket scripts are gone).
- Restored camelCase field names for the saved games filter, so settings written by 1.0.0 still deserialize (the old app wrote `unityBackends`, etc.).
- Removed the redundant selected-game checks before applying refreshed mod lists.
- Added a shared `CommandHandler::spawn_action` and reused it (plus `runtime::block_on`) for all game/mod actions, removing the repeated thread + runtime + action-state boilerplate.
- Merged the three duplicated URL mod source actions into one shared helper in `commands/mod_sources.rs`.
- Replaced the hardcoded red dialog error texts with a shared `ErrorMessage` component using `Theme.status-error`.
