# TODO

This file is here for me to track things I wanna fix for the next release. If there are things here on the main branch, I probably forgot to delete them. It should never have todos I only intend to complete in future releases, that's what GitHub issues are for.

For agents: if asked to complete todos, do the ones in the "Do now" section one by one, one commit each (simple commit message, no description). Each commit finishes the task for the parts that are 100% clear and updates this todo file by removing done items, and updating the report section below. Skip the "Do later" section completely. If something you come across isn't clear or requires some choices (only when it's really needed), you can finish some tasks partially, updating the todo list acordingly, and adding consise information about it to the notes section below (only for stuff that needs special attention). Anything you do should be moved to the Done section, with a little extra info on what exactly was done (but keep it short), to avoid repeated/reverted work in the future. If issues mentioned, include "closes #X" in the commit description when fully solved, or "for #X" when not fully solved.

## Do now

## Do later

## Notes, questions, decisions, etc

## Done

- Game refresh triggered by window focus no longer changes the refresh button state, so buttons clicked while the window was unfocused now work. Closes #336.
- Added a per-group availability key to the persisted filter state. Provider and mod family filters now reload their available values (mod families are read fresh from the database on every filter refresh, and mod source changes also refresh filters), and reset if the available set changes. Removed the old auto-disabling of unavailable providers, which also fixes the stuck reset button for unavailable options. Closes #335, closes #334.
- Update notification now uses the narrow width as a minimum and grows to fit its buttons, which are stacked vertically with update first. The changelog no longer widens the modal. Added `--test-update-notification` to dev.sh.
