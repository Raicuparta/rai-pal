# Error triage report

Source: production `ErrorNotification` analytics from v0.22.0–v0.23.0. This
report classifies the errors, records where context was added, and captures
what the message strongly suggests about the fix. It is a living document —
update it when new error classes show up.

## How to read a message

| Prefix | Source |
| --- | --- |
| `Failed to execute command: ...` | Any error returned by a Tauri command (`use-async-command.ts`). The body is the serialized Rust `Error`. |
| `Failed to initialize data update (<Provider>): ...` | Provider refresh (`refresh_games` / `refresh_mods` / `refresh_remote_games`). |
| `Failed to get app updates: ...` | `tauri-plugin-updater` path (`use-app-updater.ts`). |
| `Failed to read auth state: ...` | Auth state fetch. |
| `Sign-in failed: ...` | Auth flow. |

Localized OS text is not stable, but the trailing `(os error N)` is:

| Code | Meaning | Typical cause |
| --- | --- | --- |
| 2 | Not found | Missing file/exe, stale path, missing Steam/Game cache |
| 5 | Access denied | Permissions, protected folder, locked/restricted context |
| 32 | Sharing violation | File is open in another process (usually the game) |
| 193 | Not a valid Win32 application | Trying to execute a non-PE/script/wrong-arch file |
| 740 | Requires elevation | Target exe has a `requireAdministrator` manifest |

## Errors with a clear remedy

### io / OS errors

| Pattern | Likely cause | Suggested fix / user hint |
| --- | --- | --- |
| `os error 740` | `CreateProcess` on an exe that requires admin. Rai Pal cannot elevate. | Launch via `ShellExecute` with the `runas` verb, or run the game/launcher elevated; the target itself is the problem. |
| `os error 193` | Executing a non-executable (`.bat`/`.lnk`/script) or wrong architecture. | Invoke scripts through `cmd /c` (or `wineconsole` under Wine); verify the exe arch. |
| `os error 32` | File locked by another process (game/Steam still running). | Close the game/launcher before installing/updating/uninstalling mods; retry. |
| `os error 5` | Access denied (permissions/protected folder). | Now includes path after `fs-err`; check folder permissions / antivirus. For spawn, check elevation. |
| `os error 2` | Missing file. | Now includes path after `fs-err`; the path tells whether it is a game, cache, or config file. |
| `os error 2` on `dunce::canonicalize` | Path does not exist. | Already logs the path; returns a fallback path rather than aborting. |
| `Wine prefix not initialized at <path>` | Game has never been started, so the prefix doesn't exist. | Start the game once through Steam/Heroic, then retry. |
| `Steam Proton ... Wine prefix not initialized at <path>` | Same as above, wrapped with Proton context. | Start the game once through Steam. |
| `Failed locating the steam dir` / `Failed to find any Steam installation` / `Failed validating steam dir` | Steam not installed, never launched, or library moved. | Launch Steam once; verify `steamapps` / library folders. |
| `Itch wine prefix not found. Tried: ...` | Itch has not created its Wine prefix yet. | Open the Itch app once (or launch a game through it), then retry. |
| `Failed to find the Rai Pal executable path` | `current_exe()` failed. | Unexpected; report if reproducible. |

### Archives / downloads

| Pattern | Likely cause | Suggested fix / user hint |
| --- | --- | --- |
| `invalid Zip archive: Could not find EOCD` | Corrupt or partial download (HTML error page saved as zip, interrupted download). | Delete the cached zip / retry the download. |
| `error decoding response body` | Malformed/compressed body or connection dropped mid-transfer. | Retry; check proxy/antivirus interception. |
| `error sending request for url (...)` | DNS/TLS/connect/proxy failure, or removed asset. | Network/proxy issue; the URL identifies the host. |
| `HTTP status server error (504 ...)` | Upstream/proxy timeout (GitHub or CDN). | Retry. |

### Databases / caches / parsers

| Pattern | Likely cause | Suggested fix / user hint |
| --- | --- | --- |
| `Query returned no rows` | `rusqlite::query_row` found nothing — usually "no cached data yet" or a stale game/mod row. | Treat as empty/optional (`.optional()`); it is not necessarily a failure. If it persists, refresh/re-scan. |
| `no such table: Builds` | GOG Galaxy database schema mismatch or old/new Galaxy version. | Update GOG Galaxy (or regenerate its DB); otherwise skip GOG. |
| `missing field \`games\`` (Heroic) | Heroic store cache (`store_cache/gog_library.json`) is empty/corrupt or changed format. | Open Heroic / refresh its store cache; delete the cache to regenerate. |
| `Invalid symbol 0, offset 0` (Epic) | Corrupt/empty Epic `catcache.bin` base64. | Open Epic Games Launcher, or clear/rebuild its catalog cache. |
| `unexpected end of VDF data` | Truncated/corrupt Steam `appinfo.vdf` / `packageinfo.vdf` / `shortcuts.vdf`, often Steam not fully closed. | Close Steam, delete/rebuild `appcache`, restart Steam. |
| `Invalid symbol` / binary VDF type errors in shortcuts | Corrupt `shortcuts.vdf`. | Back up and regenerate shortcuts via Steam. |

### Auth / updater

| Pattern | Likely cause | Suggested fix / user hint |
| --- | --- | --- |
| `Auth failed: Timed out waiting for auth callback` | Browser sign-in never reached the local callback (firewall, closed tab, port blocked). | Retry and keep the browser open; check firewall. |
| `Command plugin:process\|restart not allowed by ACL` | Tauri capability config missing the `process` restart permission. | Add the permission to the capability config (build/config fix). |
| `Could not fetch a valid release JSON from the remote` | Updater endpoint returned non-JSON (network captive portal, outage, missing asset). | Retry; verify the release endpoint. |

### Expected states (should probably not be error notifications)

These are validation/state messages rather than bugs; consider using info
notifications or returning an empty/default value instead of an error.

- `This operation requires mod X to be installed.`
- `Required information for mod X is missing. Expected ...`
- `Can't do this without a game. If possible, try it from the games tab instead.`
- `This operation requires game X to be installed, but the installed game wasn't found.`
- `Game engine detection missing for this operation: open_data_folder` (refresh the game to re-detect).
- `Can't launch game X with environment variables: <provider> can't set them...` (expected provider limitation; user must launch another way or remove the mod).
- `Query returned no rows` (first-run / no cache).

## Context added in this pass

Added `Error::Context` + `.context(...)` (no dependency) and applied it to:

1. Bare io: SQLite opens now include the DB path (`local`, `GOG`, `Itch`); raw process spawn already names the program; `fs-err` covers file paths.
2. Zip: archive path on open, entry index/name on read; zip errors are no longer flattened to `io::Error::other`.
3. SQL: `get_game` / `get_mod` / installed-mod lookups now name the entity.
4. GOG: open + query errors include the Galaxy DB path.
5. Heroic JSON: parse errors include the Heroic file path.
6. HTTP decode: remote configs and remote game DB include the source URL; downloads include the URL.
7. VDF: Steam appinfo/packageinfo/shortcuts parse errors include the file path.
8. Epic: catalog cache decode/parse and manifest parse include the file path.
9. Steam locate: context added around `locate_all`.
10. Updater: frontend/plugin errors; not contextualized in this pass.
11. `Error: URL is empty`: source not found in the repo or dependency sources; still needs tracking.

## Open questions

- Where is `Error: URL is empty` constructed? It appears verbatim with an
  `Error: ` prefix that does not match our error formatting.
- Is `Query returned no rows` always the "no cached data yet" case, or are there
  real missing rows? Needs confirmation against a fresh install.
- Should the expected-state messages above move out of error notifications?
