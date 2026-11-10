# TODO

This file is here for me to track things I wanna fix for the next release. If there are things here on the main branch, I probably forgot to delete them. It should never have todos I only intend to complete in future releases, that's what GitHub issues are for.

For agents: if asked to complete todos, do these one by one, one commit each (simple commit message, no description). Each commit finishes the task for the parts that are 100% clear and updates this todo file by removing done items, and updating the report section below. If something you come across isn't clear or requires some choices (only when it's really needed), you can finish some tasks partially, updating the todo list acordingly, and adding that to the report.

# Do these now:

- some modals have clipped titles (add to steam as example, i saw on deck) showing ellipsis. title text should wrap nicely.
- if i understand correctly, slint file naming supports both _ and - interchangably, but converts to _ for rust bindings, right? if that's right, then let's rename everything on slint side to use _ only ,for consistency with the rust side and easier searches.
- does monospace font family thing require some fonts that don't necessarily exist? can we handle this more nicely and make it sturdier? should we bundle a singe monospace font or is that overkill? is there no way to just tell slint "use whatever monospace font the system has" and if none just dont use monospace? i dont like what we're doing now.
- the debug modal currently shows no text at all. likely a bug introduced when we changed how modals render with the window. we need to keep the modal that way, let's try not to go overkill. bug the debug text needs to render properly, that's the whole point.

# Don't do these yet:

- Filters scrolling still always scrolls entire sidebar.
- debug data currently missing a bunch of data. adding more buttons for game-mods maybe would be good.
- update the intro text.
- colored buttons probably need a "subtle" mode or something like in tauri.
- button size and icon spacing needs to be adjusted to match the tauri one better.
- maybe a bit too small on steam deck?
- confused about confirmation.rs in frontend, seems overkill? can it be more in slint side?

# Report:
