---
name: update-translations
description: "Use when asked to update translations, localize strings, add new languages, fix missing translation keys, or sync language files in this project. Triggered by keywords: translation, localize, locale, i18n, language strings, translate, en-us, localization files."
---

# Update Translations

Translations are gettext catalogs:

```text
crates/frontend/translations/<language>/LC_MESSAGES/rai-pal-frontend.po
```

`crates/frontend/translations/en/LC_MESSAGES/rai-pal-frontend.po` is the source of truth
for `msgid` and `msgctxt`. Other languages are under their short language code
(`de`, `es`, `fr`, `ja`, `ko`, `pt`, `wa`, `zh`).

## Adding or changing a string

1. Add or update the entry in the English catalog first.
2. Use the same `msgctxt` as the surrounding entries. The context comes from the
   `@tr("context" => "...")` call in the `.slint` file.
3. Keep placeholders exactly as numbered arguments: `{0}`, `{1}`, and so on.
   Never reorder or rename them.
4. Add the translated entry to every language file that already has a real
   translation for it. If a language does not have a translation yet, leave the
   entry out rather than inserting English placeholder text.

## Syncing other languages

Compare each non-English catalog against the English one. Missing keys are
expected to be untranslated until someone provides a real translation. Do not
fill them with copied English text, `TODO`, or machine-guessed filler.

When translating an existing key, leave all unrelated entries untouched.

## Validation

Compile-check the catalogs together with the UI:

```bash
cargo check -p rai-pal-frontend
```

If `msgfmt` is available, it can also check an individual catalog:

```bash
msgfmt --check -o /dev/null crates/frontend/translations/en/LC_MESSAGES/rai-pal-frontend.po
```
