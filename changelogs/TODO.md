# TODO

This file is here for me to track things I wanna fix for the next release. If there are things here on the main branch, I probably forgot to delete them. It should never have todos I only intend to complete in future releases, that's what GitHub issues are for.

For agents: if asked to complete todos, do these one by one, one commit each (simple commit message, no description). Each commit finishes the task for the parts that are 100% clear and updates this todo file by removing done items, and updating the report section below. If something you come across isn't clear or requires some choices (only when it's really needed), you can finish some tasks partially, updating the todo list acordingly, and adding that to the report.

# Do these now:

- missing borders between mod rows.
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

- Theme sizes now use a t-shirt scale. Spacing is a 4px base (`2xs`=2, `xs`=4, `sm`=8, `md`=12, `lg`=16, `xl`=24, `2xl`=32). Merged old 3→2, 6→8, 10→12, 20→24; deleted unused sizes. Fonts are `2xs`=8, `xs`=10 (merged 10/11), `sm`=12, `md`=14, `lg`=16, `xl`=20. Radii are `sm`=4 (merged 3/4) and `md`=8. Kept semantic sizes like `control-height`, `icon-size`, and popup dimensions as-is.
- Game rows now put their divider below the 60px content as a sibling `RowDivider`, so the border adds a pixel instead of clipping the row content. Updated the games list visible-range math to match.
