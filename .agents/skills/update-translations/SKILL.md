---
name: update-translations
description: "Use when asked to update translations, localize strings, add new languages, fix missing translation keys, or sync language files in this project. Triggered by keywords: translation, localize, locale, i18n, language strings, translate, en-us, localization files."
---

# Update Translations

Translations are gettext catalogs:

```text
crates/frontend/translations/<language>/LC_MESSAGES/rai-pal-frontend.po
```

Languages use their short code (`de`, `es`, `fr`, `ja`, `ko`, `pt`, `wa`, `zh`). There is no English catalog: English comes from the `@tr` msgids in the `.slint` files, which are index 0 of Slint's bundled translation table.

## Adding or changing a string

User-visible text always originates in `.slint` files. Rust never holds a translated sentence: it sends typed values (`MessageData`, `ConfirmKind`, `RefreshTaskData`, `DownloadItemKind`, ids, paths, version strings) and the mapping functions in `crates/frontend/ui/translations.slint` turn them into `@tr(...)` calls. When adding a message, add the enum variant in `crates/frontend/ui/app_types.slint` first, then the branch in `ui/translations.slint`, then the catalog entry.

1. Add or update the `@tr` call in the `.slint` file. That is the source of truth for `msgid` and `msgctxt`.
2. Use the same `msgctxt` as the surrounding entries. The context comes from the
   `@tr("context" => "...")` call in the `.slint` file.
3. Keep placeholders exactly as numbered arguments: `{0}`, `{1}`, and so on.
   Never reorder or rename them.
4. Add the translated entry to every catalog in the same change. Catalogs are
   always complete; there is no "untranslated until later" state.

## Keeping catalogs in sync

Every catalog must translate every string, with the same `msgctxt`, the same
placeholders, and no extra or obsolete entries. When a string changes in the UI,
update all catalogs in the same change. Never fill entries with copied English,
`TODO`, or placeholder text.

## Validation

```bash
scripts/check-translations.sh
```

The script extracts `@tr` strings from `crates/frontend/ui` with
`slint-tr-extractor` (the UI is the source of truth, not the English catalog),
then checks every catalog for missing, unused, empty, fuzzy, obsolete, or
duplicate entries and for placeholder mismatches. `scripts/check-translations.sh
--install-tool` installs the extractor version that matches Slint.

The check is not part of the build, so normal builds never run it; CI runs the
same script from `.github/workflows/check.yml`.
