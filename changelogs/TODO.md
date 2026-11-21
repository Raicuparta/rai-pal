# TODO

This file is here for me to track things I wanna fix for the next release. If there are things here on the main branch, I probably forgot to delete them. It should never have todos I only intend to complete in future releases, that's what GitHub issues are for.

For agents: if asked to complete todos, do the ones in the "Do now" section one by one, one commit each (simple commit message, no description). Each commit finishes the task for the parts that are 100% clear and updates this todo file by removing done items, and updating the report section below. Skip the "Do later" section completely. If something you come across isn't clear or requires some choices (only when it's really needed), you can finish some tasks partially, updating the todo list acordingly, and adding consise information about it to the notes section below (only for stuff that needs special attention). Anything you do should be moved to the Done section, with a little extra info on what exactly was done (but keep it short), to avoid repeated/reverted work in the future. If issues mentioned, include "closes #X" in the commit description when fully solved, or "for #X" when not fully solved.

## Do now

## Do later

- Make prereleases use a different domain for dbs.
- "Manual" rename to "Executable" i think?
- Suddenly refresh individual game way way slower? oh because of big mod db. maybe rethink.
- merge game db with mod db for one mega big db and maybe fetch only needed bits? (see investigation notes below)

## Notes, questions, decisions, etc

- Mod families were replaced with mod tags. A mod can have multiple tags. In the mods tab a mod matches if any of its tags is enabled; in the games tab a game matches if it has a compatible mod with any enabled tag. Incompatible mods are only hidden from game details when all of their tags also belong to compatible mods.
- Mod tags are stored as JSON arrays, consistent with game tags. The local database schema is now version 15. Mod DB 4 was regenerated in place because it was not released yet.
- Old `family` fields in custom URL mod sources and local mod manifests are ignored, so those sources lose their tags. This is intentional.
- Analytics mod events now send a `tags` array instead of `family`.
- Old settings that contain `modFamilies` filters or collapsed-filter keys are still read through serde aliases, so upgrading should not reset settings.
- Configs are now mods. The client no longer reads `config-db`; `config-db/0` stays as the source of truth for the DB generator, which turns each `configs.json` entry into a hidden game-specific mod tagged `<tag>-config` (e.g. `uuvr-config`). Parent mods list that tag as an optional dependency. Old `loaderId` is ignored, and `modIdOverride` was removed.
- Config mods get all matching game IDs from the game DB, plus the normalized config folder name, plus a small Steam-ID alias table in `scripts/mod-db/configs.ts` for folders named after executables rather than titles (e.g. `dew`, `ksp_x64`). Extend that table when adding configs for games whose exe name doesn't normalize to the title.
- Added `install.copy` for non-zip downloads. `ModDownload::download` now decides whether to extract based on the install action. Config download URLs include config version and content hash in the query string, so cached temp downloads are busted when a config file changes.
- The "Download Recommended Config" action and `has_remote_config` UI were removed. Existing users keep already-downloaded config files, but the new config mod only installs when the parent mod is installed or updated. That first install will overwrite the current config file once, including manual edits.
- Mod DB 4 was regenerated in place again because it still isn't released.

- Games now have an `external_ids` map instead of a single `external_id`. `Manual` is a local-only ID derived from the executable file stem, present whenever a game has an executable. `NormalizedTitle` IDs are generated from the game title for provider-discovered games. When matching against game/mod databases, `Manual` and `NormalizedTitle` are treated as the same provider.
- The local database schema is now version 12. External IDs live in `game_external_ids`; the old `normalized_titles` table is gone. Existing local databases are recreated, which keeps mod files because installed mod paths are based on the executable hash.
- Database repo changes are needed and are expected to be published as game-db 3 and mod-db 4. Game-db 3 renames the old `Manual` ID provider to `NormalizedTitle`; mod-db 4 uses `NormalizedTitle` in `gameIds`.
- Provider commands were audited. They already store fully built commands, but provider-specific helpers that need the provider's external ID now read it from `external_ids`.
- Recommended configs were left untouched; they did not block the external ID refactor.

### Copy of report about filtering stuff, to clean up, ignore for now:

Bigger filtering refactor

Yes — the current query is much better (~0.3s instead of ~2.6s), but it still derives the game-to-family mapping on every filter query. The bigger win is to materialize that mapping.

Options:

1.  Materialize game_mod_families — recommended
    - Store (provider_id, game_id, family) for every compatible mod family.
    - Rebuild when games, mods, or remote game metadata changes, or lazily when marked dirty.
    - The filter becomes a simple indexed EXISTS/NOT EXISTS; should be ~1ms.
    - Main cost: cache invalidation and rebuild timing. This is the cleanest architectural fix.

2.  Materialize full compatible mods per game
    - Store (provider_id, game_id, mod_id) for every compatible mod.
    - Filtering by family and loading game details both become simple joins.
    - Even better for game details performance, but more disk and a bigger rebuild.

3.  Normalize the mod database further
    - E.g. explicit game-ID/family relation in the remote DB instead of gameIds JSON.
    - Removes JSON work, but still requires mapping external IDs to the local library, so it has less impact than local materialization.

4.  Build an in-memory family index in Rust
    - Load a compact (provider, game_id) -> family bitmask mapping and filter in Rust.
    - Fast, but moves filtering out of SQL, which complicates sorting, search, and pagination. Not ideal.

Changing the remote mod DB alone won’t eliminate the expensive part; the local game-ID mapping is what needs materializing. If you want, I can implement option 1 next.

## Investigation: merge game DB and mod DB (future)

### Current state

- Remote game DB: `game-db/3/games.db`, 8.26 MB SQLite. One denormalized row per `(provider_id, external_id)`, 124,225 rows. It has no game identity and no titles; it is an external-ID-to-engine-metadata lookup table. The matching `games.json` is 7.5 MB and is not used by the client.
- Remote mod DB: `mod-db/4/mods.json`, 291 KB JSON. 394 mods, 14 tags, 1,201 mod game-ID references. The client downloads it, parses it, and inserts every mod into the local SQLite DB as JSON columns.
- Client refresh downloads both remote files in full on every app start (`crates/frontend/src/app.rs` calls `refresh_all`, which calls `download_database`). `games.db` is then attached and copied in full into local `remote_games` with `INSERT OR IGNORE`.
- Local DB is therefore huge: `local-15.db` is 11 MB plus an 11 MB WAL, of which `remote_games` alone is 5.5 MB (124,225 rows). Only 1,174 of those remote rows are actually referenced by the 2,431 local games / 5,070 local external IDs. The copy is also stale by design: `INSERT OR IGNORE` never updates changed engine/release data, and rows for old games are never removed.
- Local query paths only ever join the remote data against locally installed games. The client does not display the full remote game list. This is the key fact for any partial-fetch design.

### Size experiments (using the current game-db/3/games.json)

All created with `node:sqlite`, same data, measured after `VACUUM`:

- Current shape (TEXT provider/engine/date, rowid table): ~7.6 MB (current shipped file is 8.26 MB without vacuum).
- Same shape with `WITHOUT ROWID`: 4.45 MB.
- Integer enum IDs for provider/engine + integer epoch-day dates, `WITHOUT ROWID`, no titles: 2.50 MB (~70% smaller than current).
- Same plus titles: 3.22 MB.
- Normalized `games` + `game_external_ids`, integer enums/dates, titles: 3.68 MB (no secondary reverse index). Normalization costs a little size because most games have exactly one external ID, but it gives real game identity.

So the merge itself is not a size problem. A proper schema should comfortably land around 2.5-4 MB for games plus well under 1 MB for mods, even as the game list grows. The current 8 MB is mostly avoidable format overhead, not data volume.

Also found in the current generator: `games.json` contains 124,696 external IDs but only 124,225 unique `(provider, external_id)` pairs. The 471 duplicates are silently dropped by `INSERT OR IGNORE`. Merging by connected components can leave the same external ID in more than one component, so the new generator should detect and resolve/report this instead of silently losing IDs.

### HTTP VFS / partial fetch findings

- The current host (`dev.rai-pal.pages.dev`, Cloudflare Pages) does **not** support HTTP Range requests for these files: `Range: bytes=...` returns a normal `200` with the full body. HTTP VFS cannot work against the current hosting without moving the DB to something that supports ranges (GitHub Pages does return `206`; R2 or a Worker can too).
- `sqlite-vfs-http` is the obvious Rust crate, but it is AGPL-3.0. Rai Pal is GPL-3.0, so using it would effectively force the combined work to AGPL. That needs a deliberate decision, or a small in-house VFS on `rusqlite`'s `vfs` feature.
- VFS only pays off if queries are point/range lookups. Current queries are not suitable: `get_game_ids` scans local games and runs several joins/JSON `json_each` over all mods, and `get_game_mods` joins across games, external IDs, mods, installed mods, remote metadata, and dependency JSON. Against a remote VFS those shapes would pull large parts of the file page by page. A VFS design needs a deliberately query-shaped remote schema (precomputed mappings, covering indexes, no full scans).
- VFS also adds per-page HTTP latency and needs a persistent page cache, otherwise every app start refetches the working set. A 2.5-4 MB whole-file download is likely faster and much simpler than many small range requests for the amount of data this client actually needs.

### Options

1. **Keep whole-file downloads, but optimize size and caching (recommended first step).**
   - Merge into one logically designed SQLite schema (can still be two physical files; see below).
   - Use integer enum IDs (`provider`, `engine`, `architecture`, `OS`, `unity backend`), integer dates (epoch day or unix seconds), `WITHOUT ROWID` mapping tables, and only the indexes actually queried.
   - Resolve mod `gameIds` to internal game IDs at generation time. The client should not re-implement external-ID matching for mods.
   - Optionally precompute filterable `game_mod_tags` or `game_mod_ids` in the remote DB, so client filtering is an indexed lookup instead of recomputing compatibility per query. This directly addresses the existing local filtering TODO.
   - Publish a small `manifest.json` containing DB version, content hash, size, and URL. The client uses `If-None-Match`/ETag or the hash to skip the download when nothing changed. ETags already work on the current host.
   - Consider serving a pre-compressed `.db.gz`/brotli file if transfer size matters; decompression of 2-4 MB is negligible.
   - This is likely enough for a long time and preserves the current static-hosting/no-backend architecture.

2. **One schema, separate physical files (probably the best merge shape).**
   - `games.db` (game identity, external IDs, engine/release metadata) and `mods.db` (mods, tags, mod-to-game mappings) generated by the same script and sharing conventions.
   - Client attaches both; SQLite joins across attached files work normally.
   - Mod DB changes hourly, while game DB changes rarely/manually. Keeping them as separate files avoids re-downloading the whole game DB every time a mod changes, and avoids coupling game-source failures to mod updates.
   - Still removes the JSON format, duplicated matching logic, and the separate game/mod DB code paths.
   - If the goal is literally one file, this is the main downside to accept: mod-only updates invalidate and re-download the combined file.

3. **Single physical merged DB (simple, acceptable if size stays small).**
   - One `rai-pal.db` with games, external IDs, mods, tags, and mappings. One download, one attach, one version.
   - Fine if the combined file stays around a few MB and hourly mod updates re-downloading it is acceptable. With conditional GET, users only re-download when the file actually changed.

4. **HTTP VFS or per-provider/hash-bucket shards (later, only if size grows a lot).**
   - Requires range-capable hosting, read-only prepared DB (`journal_mode=MEMORY`, vacuumed, fixed page size), a VFS license decision or in-house implementation, persistent page caching, and query patterns redesigned for point lookups.
   - Simpler than a generic VFS: publish shards keyed by provider or by hash prefix of `external_id`, and let the client fetch only the buckets containing local game IDs. This avoids random SQLite page access and licensing issues, at the cost of custom shard tooling.
   - Worth revisiting only if the remote DB reaches tens of MB or whole-file refresh becomes a real bandwidth/latency problem.

5. **Delta patches** (`sqldiff`/bsdiff) between DB versions.
   - Could make hourly mod-only updates tiny, but adds a patch tool, failure/rebuild path, and version-chain management. Not worth it at current sizes; conditional whole-file download is simpler.

### Recommended client-side changes even without a merge

- Stop copying the entire remote game DB into local `remote_games`. Import only rows referenced by local external IDs (and their game identity + all IDs of that game), with upsert + delete-stale semantics. This alone should shrink the local DB from ~11 MB to roughly 1-2 MB and fix stale remote metadata.
- Store the remote content hash/ETag in the local DB or app data, and skip re-download if unchanged.
- Materialize the local game-to-compatible-mod/tag mapping when remote mod data is imported, instead of rebuilding it inside every filter query.
- Keep the local DB as the fast/offline query surface. Querying an attached remote file directly is possible, but every reader connection would need the remote DB attached, and it complicates offline behavior and the reader pool. A small imported cache is simpler and faster.

### Suggested order

1. Fix the local full-copy/stale-data problem and add conditional downloads (independent of the merge, immediate win).
2. Redesign the DB generator and schema for size (integer enums/dates, normalized game identity, resolved mod game IDs), keeping separate `games.db` and `mods.db` files initially.
3. Remove JSON from the client path and unify the game/mod remote DB code.
4. Revisit single-file packaging or HTTP VFS/shards only if real sizes or refresh timings justify it.

## Done

- Configs as mods: removed the runtime config DB from Rai Pal, added config mods to the mod DB with `<tag>-config` tags and optional dependencies, added non-zip mod installs, and removed the download-config UI.
