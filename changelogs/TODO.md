# TODO

This file is here for me to track things I wanna fix for the next release. If there are things here on the main branch, I probably forgot to delete them. It should never have todos I only intend to complete in future releases, that's what GitHub issues are for.

For agents: if asked to complete todos, do the ones in the "Do now" section one by one, one commit each (simple commit message, no description). Each commit finishes the task for the parts that are 100% clear and updates this todo file by removing done items, and updating the report section below. Skip the "Do later" section completely. If something you come across isn't clear or requires some choices (only when it's really needed), you can finish some tasks partially, updating the todo list acordingly, and adding consise information about it to the notes section below (only for stuff that needs special attention). Anything you do should be moved to the Done section, with a little extra info on what exactly was done (but keep it short), to avoid repeated/reverted work in the future. If issues mentioned, include "closes #X" in the commit description when fully solved, or "for #X" when not fully solved.

## Do now

## Do later

- Use the real domains for database urls.
- "Manual" data rename to "Local Files".
- Suddenly refresh individual game way way slower? oh because of big mod db. maybe rethink.
- i installed firewatch, refreshed, and i was in a weird stale state where the list showed it as not installed for a bit until refreshing a few times. look into.
- new way of loading the thumbnail to allow for selecting a game from any window seems super messy, might warrant a refactor.
- "show game" action for mods, maybe super messy? could refactor.
- Appimage still doesnt seem to register itself properly, the shortcut can't open it.
- for mod database, need to store normalized exe name ids too. do we already support that kind of id? this is mostly for configs but can generally be for any game-specific mod. get rid of that steam-id alias table thing and do it properly. maybe mod database needs to be normalized a bit, flat list with one ts per mod.
- this probably needs to change: "When matching against game/mod databases, `Manual` and `NormalizedTitle` are treated as the same provider.". mod database probably only has NormalizedTitle, we need "Manual" too which should be NormalizedExe or something like that for clarity. Make sure that Provider and external id source are two different enums that may or may not map.
- why query string? "Config download URLs include config version and content hash in the query string, so cached temp downloads are busted when a config file changes."

## Notes, questions, decisions, etc

### Core assumptions

- Local DB is one disposable SQLite cache. Source of truth is provider cache files, mod manifests and remote DBs; schema changes bump the file version instead of migrating.
- A refresh rewrites or removes almost every game/mod row, so building from scratch and refreshing have effectively the same write cost. Optimize the bulk rewrite path.
- Local DB: query speed first, bulk build/write speed second. Duplicate data where it clearly helps queries; avoid materializations that mostly add write cost.
- Remote: game DB optimized for download size and cheap merge; mod DB stays JSON so default and custom sources share one format.
- Only cache remote metadata for games in the local library.

### Remote mod DB

- Keep JSON, shared by default and custom URL mod sources. Shape: `{ tags, mods }` with tag definitions stored once.
- Mod game IDs stay external IDs, so the mod DB does not depend on game DB generation.
- Keep complex fields (`install`, `paths`, `dependencies`, `download`) as JSON locally.

### Local refresh / build

- Hot queries use integer columns or normalized tables, not `json_extract` over every mod. Get candidate mods from `mod_game_ids`/`game_mod_tags`.
- Option if in-place writes are still too slow: build a fresh local DB in a temp file and atomically swap. Only worth it if measured; in-place slice replacement keeps the UI responsive.

## Done

- Remote game DB is now compact SQLite: integer provider/engine IDs with name dictionaries, numeric engine versions, epoch-day release dates, `WITHOUT ROWID`, and duplicate `(provider, external_id)` resolution that keeps the latest-looking engine and release date. Client resolves metadata only for local games into `game_remote_metadata`, dropping the full remote row copy and `REMOTE_GAME_JOIN` subquery.
- Mod tags are normalized locally in `mod_tag_links`. Mod refresh now bulk-inserts all mods in one transaction, and game mod tag materialization plus `get_game_mods` use normalized tags and generated columns instead of expanding tag JSON. Local DB schema bumped to 18.
- Game provider refresh replaces the provider's whole slice (delete + bulk insert) instead of per-row upserts and child deletes. Other providers and mod/remote refresh tasks remain independent. Derived engine metadata and mod tags are kept until the post-refresh rebuild so provider refreshes do not temporarily blank filters or engine info.
- Game mod tag filtering now uses a materialized `game_mod_tags` table, rebuilt once after full refresh and per-game on single-game/local-file inserts. Local DB schema bumped to 16. Measured: per-query tag CTE ~277 ms vs ~2.2 ms filter, with ~278 ms rebuild per full refresh. Rejected full compatible-mod materialization (~3 s rebuild, ~9 ms filter, only helps selected-game details) and in-memory bitmasks (no faster after load, complicates SQL sorting/pagination).

- Linux desktop entry / deep link registration now runs in a background thread instead of blocking the UI thread before the window is shown. `xdg-mime` plus `update-desktop-database` (and the shell forks `xdg-mime` does) were taking ~250ms of the launch.

- Fixed slow installs/uninstalls. The cost was `get_game_mods`, which took ~95ms per call because `REMOTE_GAME_JOIN` re-ran its correlated `remote_games` lookup once per mod (394x) inside the candidate mods CTE. The game-level metadata (installed + remote) is now built once in a `MATERIALIZED game_info` CTE, bringing the call to ~29ms on the same data. Also stopped `install_mods` from re-running `get_game_mods` once per mod when updating several mods at once. It was never the manifest scan: 45 installed games x 394 mods of `Path::exists` costs under 10ms, and `get_mod_map` is ~3ms.
- Mods that disappear from the mod DB (or whose source is offline/disabled) are no longer deleted from the local mod DB while they have an entry in `installed_mods`, so installed mods stay visible in game details with their last known data and can still be uninstalled. `remove_mods_except` keeps installed mods, same as it already did for deprecated ones.
