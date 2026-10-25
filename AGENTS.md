# Instructions for AI agents

Rai Pal is a desktop app with a shared Rust backend in `/backend`. The Tauri v2
app (`/backend/tauri-app` + `/frontend`) is being migrated to Slint
(`/backend/slint-app`).

## Finding things

Avoid system-wide searches when you're looking for stuff. If the user is likely
to know where something is, ask first before starting a long task.

## Formatting

Ignore warnings, clippy, and formatting until the implementation is confirmed
correct. Run `npm run format` before committing so formatting churn is part of
the same commit.

## Comments

Do not leave comments. Exception only for actual hacks or genuinely weird code.
Do not remove pre-existing comments.

## Running the app

- `npm run dev` — Vite dev server and Tauri app.
- `npm run slint-dev` — Slint app with MCP enabled.
- `npm run slint-dev-live` — Slint live preview (no bundled translations).
- Backend logs go to stdout/stderr. The Tauri frontend runs in a native webview,
  so its `console.*` output and DOM are **not** visible from the terminal.

### Inspecting the Tauri frontend — dev socket

Debug builds expose dev commands through the user socket (dynamic port in
`43950..=43960`) that evaluate JavaScript in the webview and return the result.
See `scripts/dev-socket/README.md`.

```sh
npm run dev &
npm run dev-socket -- "document.title"
echo 'document.body.innerText' | npm run dev-socket
```

- Write the expression directly; no `return` needed.
- `await` works; thrown errors return with a stack trace.
- Results are JSON-serialized and pretty-printed; strings print plainly.
- Prefer reading DOM/text over screenshots; some models cannot view images.

## Styles

Tauri: use Mantine (`https://mantine.dev/llms/core-accordion.md`). Avoid custom
CSS unless necessary. Don't pass props that are already Mantine defaults or
defaults in `frontend/theme.ts`.

Slint: use `ui/theme.slint` for all spacing, sizes, radii, font sizes, and
semantic colors. Use `ui/branding.slint` for provider/engine colors. Don't
hardcode magic numbers or theme-dependent colors in components. Don't repeat
styles that a widget/layout already applies by default.

## Slint

Read `.agents/skills/slint/SKILL.md` and the relevant
`.agents/skills/slint/reference/*.md` before changing `.slint` files.

- Prefer native Slint elements/widgets over hand-rolled equivalents:
  `TabWidget`, `ContextMenuArea` + `Menu`/`MenuItem`, `PopupWindow` for non-menu
  overlays, and built-in layouts instead of manual `x`/`y` positioning.
- Use `preferred-width`/`preferred-height` for resizable windows; `width`/
  `height` pin the size.
- Model shared state and cross-cutting actions as exported globals instead of
  forwarding callbacks through every layer. UI-local state stays local.
- Use typed structs/enums and `Theme` constants instead of magic strings,
  indexes, and numbers. Keep persisted JSON field names/serde representations
  compatible with the Tauri app.
- Use `in`/`out`/`in-out` deliberately. A callback that only reads or only
  writes state should not be routed through a component that does neither.
- Give custom interactive components `accessible-role` and `accessible-label`
  (and default actions where applicable).
- Mark user-visible strings with `@tr`, and use `{}` placeholders instead of
  string concatenation so translations can reorder arguments.
- Reserve space for hover-only controls and track hover across the whole row so
  layouts don't shift or flicker.
- Compile-check with `cargo check -p rai-pal-slint`; for visual/runtime checks
  use the Slint MCP server or `slint-viewer`, not only source inspection.

## Tauri to Slint migration

The Tauri app remains the reference for behaviour and information hierarchy
until the migration is complete.

- Match structure, hierarchy, and behaviour, not exact styling. Keep primary
  (bold/high-contrast) vs secondary (thin/low-contrast) emphasis and alignment
  consistent.
- Prefer native Slint components over recreating Tauri/Mantine widgets; small
  structural changes to use Slint built-ins better are fine.
- Icons are used less than in Tauri. Regular buttons/menu items have no icons;
  compact icon-only controls (filter lock/reset, overflow menus) may use them.
- Put shared logic/state in `rai-pal-core` instead of duplicating it between
  `tauri-app` and `slint-app`. Be careful not to introduce extra lock
  contention or make hot paths synchronous.
- Persisted state must use the same files, locations, and JSON structure in both
  apps.
- Refer to the Tauri implementation for detailed behaviour (filters, locking,
  reset buttons, empty states, etc.) instead of inventing new behaviour.
- Localization source of truth is `frontend/localizations`. Regenerate Slint
  gettext catalogs with `node scripts/ts-localizations-to-po.mjs` after changes.

## Error handling in the backend

Errors use thiserror enums in the two `result.rs` files (core and tauri-app).
Errors reach the frontend mostly as strings. `fs-err` already adds useful
context; for dependency `io::Error`s and similar, add context with
`thiserror-context` whenever the error can reach the frontend.

## Tests

Don't add tests unless asked. If a change warrants tests (especially near
existing tests), ask first.
