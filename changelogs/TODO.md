# TODO

This file is here for me to track things I wanna fix for the next release. If there are things here on the main branch, I probably forgot to delete them. It should never have todos I only intend to complete in future releases, that's what GitHub issues are for.

For agents: if asked to complete todos, do the ones in the "Do now" section one by one, one commit each (simple commit message, no description). Each commit finishes the task for the parts that are 100% clear and updates this todo file by removing done items, and updating the report section below. Skip the "Do later" section completely. If something you come across isn't clear or requires some choices (only when it's really needed), you can finish some tasks partially, updating the todo list acordingly, and adding consise information about it to the notes section below (only for stuff that needs special attention). Anything you do should be moved to the Done section, with a little extra info on what exactly was done (but keep it short), to avoid repeated/reverted work in the future. If issues mentioned, include "closes #X" in the commit description when fully solved, or "for #X" when not fully solved.

## Do now

- The update notification modal is using the narrow width, and its action buttons (update or ignore) don't fit. So let's do two things:
  - Narrow width is the minimum width.
  - The buttons are shown vertically on top of each other: update first, ignore after.
  - The button text must never be clipped or elided or whatever its called.
  - If the button text makes the button wider than the modal width, the modal should grow with it, never clip anything.
  - The update changelog text should never make the modal wider, it wraps as expected.
  - Add a "--test-update-notification" param to dev.sh that makes the app show the update notification (dev mode only).

## Do later

## Notes, questions, decisions, etc

## Done

- Game refresh triggered by window focus no longer changes the refresh button state, so buttons clicked while the window was unfocused now work. Closes #336.
- Added a per-group availability key to the persisted filter state. Provider and mod family filters now reload their available values (mod families are read fresh from the database on every filter refresh, and mod source changes also refresh filters), and reset if the available set changes. Removed the old auto-disabling of unavailable providers, which also fixes the stuck reset button for unavailable options. Closes #335, closes #334.
