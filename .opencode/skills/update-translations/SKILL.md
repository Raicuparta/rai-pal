---
name: update-translations
description: Use when asked to update translations, localize strings, add new languages, fix missing translation keys, or sync language files in this project. Triggered by keywords: translation, localize, locale, i18n, language strings, translate, en-us, localization files.
---

# Update Translations

## Step 1: Validate source of truth

Run TypeScript check on `frontend/localizations/en-us.ts`:

```bash
npx tsc --noEmit --ignoreConfig frontend/localizations/en-us.ts
```

If there are **any** TypeScript errors, cancel the entire process and report the error to the user. Do not proceed.

`en-us.ts` is the source of truth with contextual comments. No other language files have or should have comments.

## Step 2: Check and fix other language files

For every other `.ts` file in `frontend/localizations/` (excluding `en-us.ts` and `localizations.ts`):

1. Run `npx tsc --noEmit --ignoreConfig frontend/localizations/<file>.ts` — if it passes, skip it entirely.
2. If it has errors, the TypeScript errors will tell you exactly which keys are missing or have wrong parameter signatures (because they all use the `Localization` type derived from `en-us`).
3. Translate the missing entries **one by one**, respecting the context comments in `en-us.ts`. Leave all existing translations untouched.
4. After fixing each file, re-run the type check to confirm it passes.

## Rules

- Never add comments to non-en-us files.
- Never modify `localizations.ts`.
- When syncing other language files (Steps 1–2 above), never modify `en-us.ts`. Only modify `en-us.ts` when adding new keys for a feature (see below).
- Keep existing translations as-is — only add missing entries or fix parameter mismatches.
- Match the same parameters/variables from `en-us.ts` (e.g. `{items}`, `{authorName}`, `{provider}`).
- **NEVER put placeholder values in any language file.** No English text, no `TODO`, no copied en-us strings, no machine-guessed filler — a key either gets a real translation or it is not added to that file at all.

## Missing translations are signaled by type errors — on purpose

Type errors from missing translation keys are **by design**. They are the mechanism that tells us which translations are missing. Never "fix" them by inserting placeholder values.

### When the user says "don't translate yet" / "use placeholders"

Do NOT put placeholder values in any language file. Instead:

1. Use the translations hook as normal in the component code.
2. Purposely reference a new translation key that doesn't exist yet.
3. Leave **all** language files untouched, including `en-us.ts`.
4. The resulting type errors are intentional — leave them, and tell the user they mark the pending translations.

### When implementing features that need translated text (user gave no instructions about translations)

Default behavior:

1. Add the new keys **only** to `en-us.ts`, with contextual comments as usual.
2. Leave every other language file untouched.
3. Ignore the type errors this causes in the other language files — they are the intended signal that translations are missing.
