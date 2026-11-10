# TODO

This file is here for me to track things I wanna fix for the next release. If there are things here on the main branch, I probably forgot to delete them. It should never have todos I only intend to complete in future releases, that's what GitHub issues are for.

For agents: if asked to complete todos, do these one by one, one commit each (simple commit message, no description). Each commit finishes the task for the parts that are 100% clear and updates this todo file by removing done items, and updating the report section below. If something you come across isn't clear or requires some choices (only when it's really needed), you can finish some tasks partially, updating the todo list acordingly, and adding that to the report.

# Do these now:


# Don't do these yet:

- Filters scrolling still always scrolls entire sidebar.
- debug data currently missing a bunch of data. adding more buttons for game-mods maybe would be good.
- update the intro text.
- colored buttons probably need a "subtle" mode or something like in tauri.
- button size and icon spacing needs to be adjusted to match the tauri one better.
- maybe a bit too small on steam deck?
- confused about confirmation.rs in frontend, seems overkill? can it be more in slint side?

# Report:
- Modal titles now wrap instead of eliding. Modal height is still the header + body preferred height sum, so
  wrapped titles grow the modal correctly.
- All `.slint` files (and their imports) renamed to snake_case. Non-Slint assets (icons, images) kept their
  kebab-case names.
- Monospace family is now looked up from the system through Slint's own fontique collection
  (`slint::fontique_011::shared_collection()`, requires the `unstable-fontique-011` feature), instead of
  hardcoding `DejaVu Sans Mono`/`Consolas`/`Menlo`. If the system has no monospace family, the font-family is
  left empty so text falls back to the default system font instead of a broken family name.
- Debug modal text was invisible because `Modal`'s body layout used `alignment: start`, so the `TextEdit`
  (which only ever sizes itself by stretch) collapsed to zero height. Removed that alignment so children can
  stretch inside `big` modals; content-sized modals are unaffected because their body height equals its
  preferred height.
