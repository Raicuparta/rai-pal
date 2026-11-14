# Provider parsing fixtures

Each fixture directory contains real provider files plus an `expected.json` file.

`expected.json` is a flat list of external IDs that must be present after
parsing. Only the listed games are checked; the fixture may contain others.

```json
["some-provider-id", "another-provider-id"]
```
