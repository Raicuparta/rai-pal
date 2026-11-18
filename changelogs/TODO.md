# TODO

This file is here for me to track things I wanna fix for the next release. If there are things here on the main branch, I probably forgot to delete them. It should never have todos I only intend to complete in future releases, that's what GitHub issues are for.

For agents: if asked to complete todos, do the ones in the "Do now" section one by one, one commit each (simple commit message, no description). Each commit finishes the task for the parts that are 100% clear and updates this todo file by removing done items, and updating the report section below. Skip the "Do later" section completely. If something you come across isn't clear or requires some choices (only when it's really needed), you can finish some tasks partially, updating the todo list acordingly, and adding consise information about it to the notes section below (only for stuff that needs special attention). Anything you do should be moved to the Done section, with a little extra info on what exactly was done (but keep it short), to avoid repeated/reverted work in the future. If issues mentioned, include "closes #X" in the commit description when fully solved, or "for #X" when not fully solved.

## Do now

- We have a "mod" filter category that is dynamic, it changes depending on available mods. Currently the available filters here seem to only update on app startup, but there are cases where our mod list change. For instance refreshing could bring in new mods, or changing the mod sources could also add or remove mods. So we need to make sure these mod filters are updated accordingly. Important to also make sure the persisted mod filter state does't become broken. Maybe it would be a good idea to introduce the concept of filter category state invalidation, for filter categories like these. We already do something like this for the game provider filter category: if available providers change, we handle some special cases carefully to avoid the filters ending up in an invalid state (like no filters selected, causing empty results list). So we could refactor filter categories to have some kind of "cache key" (probably better name than this since it isn't a cache i guess? you decide, im not sure) that gets saved together with the filter persisted state. If that key gets changed, the filter state resets. So for game providers that key could be a hash of the available providers. For mods it could be a hash of available mod ids (which are already different per mod source, so no need to handle that). Issue #335.
- Reset filter button stuck on, not resetting filters with disabled items. This is somewhat related to the game provider thing mentioned above. Could be that doing the refactor above also simplifies the code related to this, i'd like that. But basically this bug is in game provider filter, when some options are disabled, the filter reset button is shown as if there's something to reset, but there isn't because the only checkbox to reset is disabled, so it just never resets and keeps the button active forever. So similar to locked checkboxes, the disabled ones shouldn't be counted towards the "is there anything to revert" checks. Issue #334.
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
