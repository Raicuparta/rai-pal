# Provider parsing fixtures

Each fixture directory contains real provider files plus an `expected.json` file.

`expected.json` is a flat list of external IDs that must be present after
parsing. Only the listed games are checked; the fixture may contain others.

```json
["some-provider-id", "another-provider-id"]
```

Fixture roots:

- `steam/<name>`: Steam installation root, containing `appcache/appinfo.vdf` and `steamapps/`.
- `heroic-epic/<name>`, `heroic-gog/<name>`, `heroic-amazon/<name>`: Heroic config root.
- `epic/<name>`: Epic Games Launcher app data root, containing `Manifests/` and `Catalog/`.
- `gog/<name>`: directory containing `galaxy-2.0.db`.
- `itch/<name>`: directory containing `db/butler.db`.
