---
name: update-translations
description: "Use when asked to update translations, localize strings, add new languages, fix missing translation keys, or sync language files in this project. Triggered by keywords: translation, localize, locale, i18n, language strings, translate, en-us, localization files."
---

# Update Translations

Translations are gettext catalogs at `crates/frontend/translations/<language>/LC_MESSAGES/rai-pal-frontend.po`. Languages use their short codes (`de`, `es`, `fr`, `ja`, `ko`, `pt`, `wa`, `zh`). There is no English catalog: English comes from the `@tr` msgids in the `.slint` files.

## Adding or changing a string

User-visible text always originates in `.slint` files. Rust never holds a translated sentence: it sends typed values (`MessageData`, `ConfirmKind`, `RefreshTaskData`, `DownloadItemKind`, ids, paths, version strings) and the mapping functions in `crates/frontend/ui/translations.slint` turn them into `@tr(...)` calls. When adding a message, add the enum variant in `crates/frontend/ui/app_types.slint` first, then the branch in `ui/translations.slint`, then the catalog entries.

1. Add or update the `@tr` call. Its context is the `@tr("context" => "...")` first argument and becomes the `msgctxt`.
2. Keep numbered placeholders (`{0}`, `{1}`, ...) exactly as they are. Never reorder or rename them.
3. Add the entry to every catalog in the same change. Catalogs are always complete, with no extra, obsolete, or English-copied entries.

## Validation

```bash
scripts/check-translations.sh
```

Extracts the `@tr` strings from `crates/frontend/ui` with `slint-tr-extractor` and checks every catalog for missing, unused, empty, fuzzy, obsolete, or duplicate entries and for placeholder mismatches. Also runs as part of `scripts/check.sh`.
