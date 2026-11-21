# TODO

This file is here for me to track things I wanna fix for the next release. If there are things here on the main branch, I probably forgot to delete them. It should never have todos I only intend to complete in future releases, that's what GitHub issues are for.

For agents: if asked to complete todos, do the ones in the "Do now" section one by one, one commit each (simple commit message, no description). Each commit finishes the task for the parts that are 100% clear and updates this todo file by removing done items, and updating the report section below. Skip the "Do later" section completely. If something you come across isn't clear or requires some choices (only when it's really needed), you can finish some tasks partially, updating the todo list acordingly, and adding consise information about it to the notes section below (only for stuff that needs special attention). Anything you do should be moved to the Done section, with a little extra info on what exactly was done (but keep it short), to avoid repeated/reverted work in the future. If issues mentioned, include "closes #X" in the commit description when fully solved, or "for #X" when not fully solved.

## Do now

## Do later

- Make prereleases use a different domain for dbs.
- "Manual" rename to "Executable" i think?
- Suddenly refresh individual game way way slower? oh because of big mod db. maybe rethink.
- merge game db with mod db for one mega big db and maybe fetch only needed bits? (see investigation notes below)
- look into what happens when a mod disappears from remote db but is already installed. seemed to disappear from rai pal ui even though we have an installed manifest? should show something so it gets uninstalled.
- i installed firewatch, refreshed, and i was in a weird stale state where the list showed it as not installed for a bit until refreshing a few times. look into.
- new way of loading the thumbnail to allow for selecting a game from any window seems super messy, might warrant a refactor.
- "show game" action for mods, maybe super messy? could refactor.

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

### Revised options given a full game catalog and a future Discover tab

A Discover tab changes the earlier recommendation: the client will need access to games the user does not own, so caching only rows referenced by local games is a dead end. The remote catalog must be a first-class browsable dataset. A roughly 100x larger game DB (several hundred MB if mostly text) changes the priorities: full unconditional downloads, full local copies, and one giant merged file stop being acceptable before HTTP VFS itself becomes necessary.

1. **Keep one logical schema but separate physical files.**
   - `games.db` (full game catalog), `mods.db` (mods, tags, mod-to-game mappings), and optionally a small precomputed `facets.db`/`mod-games.db`.
   - Client attaches the files it needs; SQLite joins across attached files work normally.
   - Pros: mod-only updates stay tiny and hourly, game catalog can update independently, game-source failures don't block mod publishing, and each file can have its own download/sharding strategy.
   - Pros: a single combined DB would make every hourly mod update invalidate a potentially hundreds-of-MB game file.

2. **Give every game a stable internal identity and normalize external IDs.**
   - One row per real game, plus `game_external_ids(provider_id, external_id, game_id)`.
   - Use a deterministic stable ID that survives regeneration (e.g. preferred Steam appid, otherwise a stable provider/id hash), not row order or an autoincrement that changes when sources change.
   - Detect duplicate external IDs across merged games and resolve/report them instead of silently dropping 471 IDs like today.
   - Pros: stable deep links, sharding by game ID, incremental updates, cached Discover data, and mod mappings that don't need re-resolving on the client.
   - Pros: enables future content-addressed/chunked updates without changing the client-facing identity.

3. **Split the catalog into a compact browse/search core and heavier details.**
   - Core: stable game ID, display title, normalized title, store IDs, engine, release date, tags, maybe thumbnail URL. This is what Discover lists, searches, sorts, and filters.
   - Details: long descriptions, images, install metadata, anything only needed when a game is opened, either as separate columns/tables or a separate detail shard fetched on demand.
   - Pros: Discover can load a much smaller working set; search/filter/sort over compact rows is faster; users don't download hundreds of MB of descriptions to browse titles.
   - Pros: also makes future HTTP VFS useful, because the common path touches a small, page-local core.

4. **Build the update/transport layer before the catalog gets huge.**
   - Now: a small `manifest.json` with schema version, content hash, size, and file URLs; conditional GET/ETag; pre-compressed snapshots (zstd/gzip) where it helps.
   - Next: partition the catalog into shards by provider, stable game-ID hash, or alphabet/tag, and fetch only the shards needed. For Discover, fetch the compact core index first, then detail shards on demand.
   - Later: delta patches or content-addressed chunks so a catalog update doesn't require re-downloading everything.
   - Pros: avoids the current behavior of downloading the whole DB on every app start, scales from today's few MB to hundreds of MB without a backend, and keeps static hosting viable.
   - Pros: sharding is simpler and less risky than HTTP VFS, and can ship long before VFS is worth it.

5. **Use the remote catalog as a read-only on-disk cache, not as rows imported into the local DB.**
   - Local DB should hold user state: installed games, installed mods, custom sources, caches that are actually local-only.
   - Attach downloaded remote files for querying; refresh by atomically swapping files, not by copying millions of rows into `remote_games`.
   - Pros: avoids duplicating a future hundreds-of-MB catalog into the local DB and its WAL, makes refresh/rollback atomic, and keeps the huge catalog out of user-state writes.
   - Pros: offline Discover works for whatever shards are cached, without rebuilding a second copy of the data.

6. **Precompute query-shaped tables in the generator.**
   - Resolve mod `gameIds` to internal game IDs at generation time.
   - Precompute the small `mod -> game` mapping and `game -> filterable mod tags` for modded games only. This stays small even when the full catalog is huge.
   - Precompute Discover facets/counts if the UI needs them: counts by engine, store, tag, release year, mod availability, etc.
   - Pros: client filtering becomes indexed lookups instead of scanning games × mods × JSON dependencies; scales to millions of games.
   - Pros: directly removes the expensive local mod-tag filtering work already documented above, and makes queries VFS-friendly later.

7. **Optimize the physical schema aggressively from the start.**
   - Integer enum/dictionary IDs for provider, engine, architecture, OS, Unity backend; integer dates; `WITHOUT ROWID` for mapping tables; compact text storage; only indexes that queries actually use.
   - Fixed page size, `VACUUM`, read-only, `journal_mode=MEMORY` for published files; deterministic generation.
   - Pros: the same data was measured at ~2.5-3.7 MB instead of 8.26 MB for the current engine-only DB, a 2-3x saving before considering compression.
   - Pros: fewer pages to cache, download, shard, or later fetch over a VFS; lower memory and disk pressure on clients.

8. **Make the layout VFS-ready without adopting VFS yet.**
   - Immutable versioned snapshots, fixed page size, stable clustering by common query (e.g. game ID for core/details, `(provider, external_id)` for lookups), no cross-shard joins on hot paths.
   - Keep the browse core and large details physically separable so a future VFS can fetch only the pages a Discover query needs.
   - Pros: if HTTP VFS is needed later, the DB is already prepared; if not, the same properties improve sharding, delta updates, and local query performance.
   - Pros: avoids committing to the AGPL `sqlite-vfs-http` crate or range-capable hosting until the size actually justifies it.

9. **Decide what Discover actually needs before designing its data source.**
   - Is it the entire catalog with arbitrary search/filter, or curated sections (popular, new releases, moddable games, specific stores)?
   - If curated, the client may only need a compact core plus on-demand detail shards, not the whole catalog locally.
   - Pros: this decision has a much bigger impact on download size than the game/mod DB merge itself.
   - Pros: designing stable IDs and the core/detail split now keeps either direction open.

10. **Fix the current local-copy and stale-data problem now, but in a forward-compatible way.**
    - Do not implement "import only locally owned rows", since Discover needs the wider catalog.
    - Instead stop importing the remote catalog into the local user DB, attach the remote files read-only, and replace `INSERT OR IGNORE` copying with atomic file swaps plus a manifest/hash check.
    - Pros: immediate shrink of the local DB and WAL, immediate fix for stale remote metadata, no throwaway work when the full catalog lands.
    - Pros: the client gets used to the file-download/cache model before sharding and partial fetches are added.

### Suggested order

1. Stop copying the remote DB into the local DB; attach it read-only and add manifest/hash-based conditional downloads.
2. Redesign the generator around stable game identity, normalized external IDs, compact integer schema, and resolved mod-to-game mappings.
3. Keep `games.db` and `mods.db` (and optional precomputed facets) as separate files under one logical schema.
4. Split the game catalog into a compact browse core and heavier detail data, and add sharding/partial downloads before the catalog becomes huge.
5. Add delta/chunked updates only if shard refreshes are still too expensive.
6. Revisit HTTP VFS only after the full catalog is large enough that the sharded whole-file approach is no longer good enough.

## Done

- Configs as mods: removed the runtime config DB from Rai Pal, added config mods to the mod DB with `<tag>-config` tags and optional dependencies, added non-zip mod installs, and removed the download-config UI.
- Mod rows in the mods tab and game details are clickable (with hover) and open the same menu as the three-dot button. Mod actions are now one ordered list per mod, shared by the menu and the buttons shown outside it (game details keeps its old outside buttons; the mods tab only has the menu). Mods with game IDs get a "Show {game}" menu item per matching game, which opens that game's details. Game details now also fetches the selected game's thumbnail itself, so it shows up for games that were never scrolled into view in the games list. The hide-thumbnails setting now only affects the games list, not the details page.
