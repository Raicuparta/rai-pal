# TODO

This file is here for me to track things I wanna fix for the next release. If there are things here on the main branch, I probably forgot to delete them. It should never have todos I only intend to complete in future releases, that's what GitHub issues are for.

For agents: if asked to complete todos, do the ones in the "Do now" section one by one, one commit each (simple commit message, no description). Each commit finishes the task for the parts that are 100% clear and updates this todo file by removing done items, and updating the report section below. Skip the "Do later" section completely. If something you come across isn't clear or requires some choices (only when it's really needed), you can finish some tasks partially, updating the todo list acordingly, and adding consise information about it to the notes section below (only for stuff that needs special attention). Anything you do should be moved to the Done section, with a little extra info on what exactly was done (but keep it short), to avoid repeated/reverted work in the future. If issues mentioned, include "closes #X" in the commit description when fully solved, or "for #X" when not fully solved.

## Do now

Database over at ~/Repos/rai-pal-db. Feel free to change and commit there if needed, with the expectation that things won't be live until i push.

## Do later

- Updating database versions too frequently now, this makes all mods stop showing as installed.

## Notes, questions, decisions, etc

## Done

- Mod sources are an ordered list now: the default source is always first, custom sources can be reordered with up/down buttons, later sources override mods with the same id from earlier ones, and local folder mods always win. Removed mod scopes, source hashes, `ModProviderId`, and the per-source reconciliation code; the catalog is rebuilt from the enabled sources on each refresh.
- Mod DB `install.manifestPath` is now `install.installKind` (`Shared` or `PerGame`). Manifest paths are derived as `installed-mods/<exe hash>/manifests/<id>.json` or `shared-mods/<id>/rai-pal-manifest.json`; every mod that was already in the database matched one of these exactly.
- Installed mods are detected by scanning those two locations instead of the catalog, so mods that disappear from the DB (or come from a disabled/offline source) keep their installed state. Old scoped ids in installed manifests are normalized by dropping the old prefix, and unknown manifests still fall back to an id-only read. Mods that only exist as installed manifests show as installed, without update/reinstall actions.
- Installed manifests that can't be parsed now show as a fake "broken mod" row (title is the manifest file name) with only the "Uninstall all mods" action, instead of being silently invisible. The description string still needs translations.
