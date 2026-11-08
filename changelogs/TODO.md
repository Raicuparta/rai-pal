# TODO

This file is here for me to track things I wanna fix for the next release. If there are things here on the main branch, I probably forgot to delete them. It should never have todos I only intend to complete in future releases, that's what GitHub issues are for.

For agents: if asked to complete todos, do these one by one, one commit each (simple commit message, no description). Each commit finishes the task for the parts that are 100% clear and updates this todo file by removing done items, and updating the report section below. If something you come across isn't clear or requires some choices (only when it's really needed), you can finish some tasks partially, updating the todo list acordingly, and adding that to the report.

# Do these now:

- theme variables are named after pixels, that's dumb. let's try to do md, sm, xl, xs, and reduce very specific numbers. look through usages and make a report of all sizes we need, and some that differ by only 1px could be merged into one. unused ones can be deleted.
- missing borders between mod rows.
- the border between game rows seems to "eat" into the game row height, making things look misaligned vertically. the border should be outside the game row and add to the height of it i think, like pushing things down. mod rows should have the same border.
- need to adjust styling for the dropdowns inside menus:
  - some horizontal padding to align better with the other menu items.
  - title above instead of inside the element, just like the tauri app.
- mod action buttons in mods tab are too big, looks like they're filling the height or something? they should be made to look the same as in the game details page.
- "view" slint files are confusing, do they need to be separate from the main slint files? seems like we could clean this up and merge some files. if not, explain it to me.
- toast notification timer says 5s but definitely taking way more than 5 seconds to disappear.
- mod update button missing color. update should be green.
- cursor wrong for checkbox part of checkbox row. should be pointer while hovering the whole row, including the checkbox part.
- bug: can click through toast notifications.
- bug: unchecking all filters not taking into account locked (shouldnt count locked as something that needs to be unchecked for the reset to happen, unchecking all unlocked items is what matters and should trigger the reset to check all that are unlocked).

# Do not do yet:

- Filters scrolling still always scrolls entire sidebar.
- debug data currently missing a bunch of data. adding more buttons for game-mods maybe would be good.
- the way the dark bool in theme.slint is set is insane, fix that.
- update the intro text.
- colored buttons probably need a "subtle" mode or something like in tauri.
- button size and icon spacing needs to be adjusted to match the tauri one better.

# Super succint report on what is done, why, what was left for me to answer questions, etc.
