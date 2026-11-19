# TODO

This file is here for me to track things I wanna fix for the next release. If there are things here on the main branch, I probably forgot to delete them. It should never have todos I only intend to complete in future releases, that's what GitHub issues are for.

For agents: if asked to complete todos, do the ones in the "Do now" section one by one, one commit each (simple commit message, no description). Each commit finishes the task for the parts that are 100% clear and updates this todo file by removing done items, and updating the report section below. Skip the "Do later" section completely. If something you come across isn't clear or requires some choices (only when it's really needed), you can finish some tasks partially, updating the todo list acordingly, and adding consise information about it to the notes section below (only for stuff that needs special attention). Anything you do should be moved to the Done section, with a little extra info on what exactly was done (but keep it short), to avoid repeated/reverted work in the future. If issues mentioned, include "closes #X" in the commit description when fully solved, or "for #X" when not fully solved.

## Do now

## Do later

## Notes, questions, decisions, etc

- Games now have an `external_ids` map instead of a single `external_id`. `Manual` is a local-only ID derived from the executable file stem, present whenever a game has an executable. `NormalizedTitle` IDs are generated from the game title for provider-discovered games. When matching against game/mod databases, `Manual` and `NormalizedTitle` are treated as the same provider.
- The local database schema is now version 12. External IDs live in `game_external_ids`; the old `normalized_titles` table is gone. Existing local databases are recreated, which keeps mod files because installed mod paths are based on the executable hash.
- Database repo changes are needed and are expected to be published as game-db 3 and mod-db 4. Game-db 3 renames the old `Manual` ID provider to `NormalizedTitle`; mod-db 4 uses `NormalizedTitle` in `gameIds`.
- Provider commands were audited. They already store fully built commands, but provider-specific helpers that need the provider's external ID now read it from `external_ids`.
- Recommended configs were left untouched; they did not block the external ID refactor.

## Done
