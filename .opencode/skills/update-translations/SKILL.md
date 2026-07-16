---
name: update-translations
description: Use when asked to update translations, localize strings, add new languages, fix missing translation keys, or sync language files in this project. Triggered by keywords: translation, localize, locale, i18n, language strings, translate, en-us, localization files.
---

# Update Translations

## Step 1: Validate source of truth

Run TypeScript check on `frontend/localizations/en-us.ts`:

```bash
npx tsc --noEmit frontend/localizations/en-us.ts
```

If there are **any** TypeScript errors, cancel the entire process and report the error to the user. Do not proceed.

`en-us.ts` is the source of truth with contextual comments. No other language files have or should have comments.

## Step 2: Check and fix other language files

For every other `.ts` file in `frontend/localizations/` (excluding `en-us.ts` and `localizations.ts`):

1. Run `npx tsc --noEmit frontend/localizations/<file>.ts` — if it passes, skip it entirely.
2. If it has errors, the TypeScript errors will tell you exactly which keys are missing or have wrong parameter signatures (because they all use the `Localization` type derived from `en-us`).
3. Translate the missing entries **one by one**, respecting the context comments in `en-us.ts`. Leave all existing translations untouched.
4. After fixing each file, re-run the type check to confirm it passes.

## Rules

- Never add comments to non-en-us files.
- Never modify `en-us.ts` or `localizations.ts`.
- Keep existing translations as-is — only add missing entries or fix parameter mismatches.
- Match the same parameters/variables from `en-us.ts` (e.g. `{items}`, `{authorName}`, `{provider}`).
