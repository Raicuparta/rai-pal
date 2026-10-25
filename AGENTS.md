# Instructions for AI agents

Rai Pal is a Tauri v2 desktop app: Rust backend in `/backend`, TypeScript/React
frontend in `/frontend`.

## Finding things

Avoid doing system-wide searches when you're looking for stuff. If the user is likely to know where something is, just ask first before starting a long task.

## Formatting

Ignore code warnings, clippy, format, etc, until I've confirmed that your implementation is correct. This way you don't waste time fixing tiny things that will be thrown away anyway.

Run `npm run format` before committing, so formatting changes are part of your
commit instead of showing up as unrelated churn later.

## Comments

Do not leave comments. Exception only for actual hacks or realy weird stuff, almost everything should be comment-free. Do not remove preexisting comments.

## Running the app

- `npm run dev` — starts the Vite dev server and the Tauri app (opens a native window).
- Backend logs go to stdout/stderr. The frontend runs in a native system webview, so
  its `console.*` output and DOM are **not** visible from the terminal.

## Inspecting the frontend — the dev socket

Debug builds expose dev commands through the user socket (dynamic port in
`43950..=43960`) that evaluate arbitrary JavaScript in the webview and read the
result back. This is how you read the DOM and drive the UI. See
`scripts/dev-socket/README.md`.

```sh
npm run dev &                          # start the app (backgrounded)
npm run dev-socket -- "document.title" # evaluate a single expression
echo 'document.body.innerText' | npm run dev-socket   # pipe JS in
```

Key points to remember:

- Write the expression directly — no `return` needed (`document.body.innerText` works).
- `await` is supported; thrown errors come back with a stack trace.
- Results are JSON-serialized and pretty-printed; strings print plainly.
- Prefer reading the DOM/text over screenshots; some models cannot view images.

## Styles

Use Mantine. Docs here: https://mantine.dev/llms/core-accordion.md

Avoid making your own custom styles unless absolutely necessary. Default to using Mantine's provided utilities and styles with no extra css files, only adjusting it when necessary for functionality, or when specifically asked for something that can't be done easily with Mantine directly. When passing props to Mantine components, make sure you're not being redundant by passing something that is either already a Mantine default, or a default in this project's theme (frontend/theme.ts)

## Slint

- Before working on `.slint` files, read `.agents/skills/slint/SKILL.md` and the relevant files in `.agents/skills/slint/reference/`.
- Prefer built-in Slint elements/widgets over hand-rolled equivalents: `TabWidget` for tabs, `ContextMenuArea` + `Menu`/`MenuItem` for context/dropdown menus, `PopupWindow` only for non-menu overlays.
- Use `preferred-width` / `preferred-height` when a window should stay resizable; setting `width` / `height` pins its size.
- Compile-check Slint changes with `cargo check -p rai-pal-slint`.

## Tauri to Slint migration

Rai Pal is being migrated from the Tauri app (`/backend/tauri-app` +
`/frontend`) to the Slint app (`/backend/slint-app`). The Tauri app remains the
reference implementation for behaviour and general UI structure until the
migration is complete.

- Match the general structure, information hierarchy, and behaviour of the Tauri
  UI, not its exact styling. What is primary (bold/high contrast) and what is
  secondary (thin/low contrast), and how things are aligned/centered, should
  stay consistent.
- Prefer native Slint components over recreating custom Tauri/Mantine widgets.
  It is fine to restructure slightly to use Slint's built-ins better.
- Icons are intentionally being used less than in the Tauri app. Regular buttons
  and menu items have no icons. Compact controls that are icon-only in the Tauri
  UI (filter lock/reset, overflow menus) can use icons.
- When logic or shared state can live in `rai-pal-core` instead of a specific
  app crate, move it there. Avoid duplicating behaviour between the Tauri and
  Slint apps.
- Persisted state must stay compatible between the two apps: use the same files
  in the same locations, with the same JSON structure.
- Don't hardcode colors that assume one theme. Use `Palette`/`ColorScheme` or
  values that read correctly in both light and dark mode.
- For controls revealed on hover, reserve their layout space and track hover
  across the whole row, so the layout doesn't shift or flicker.
- Refer to the Tauri implementation for detailed behaviour (filters, locking,
  reset buttons, empty states, etc) instead of inventing new behaviour.
- The Tauri localization files in `frontend/localizations` remain the source of
  truth. Regenerate the Slint gettext catalogs with
  `node scripts/ts-localizations-to-po.mjs` after changing them; Slint UI
  strings use @tr.
- Slint live preview does not support bundled translations. Use normal
  `npm run slint-dev` when working on translations; `npm run slint-dev-live` is
  only for live UI reloading.

## Error handling in the backend

We use thiserror via the enums defined in two result.rs files (core and tauri-app crates). Errors get thrown to the frontend mostly as-is, as strings. fs-err is used to add necessary context, so those are already fine. But other cases like io::Error coming from dependencies, etc, might need some extra context passed. For this, we use thiserror-context. Make sure to use that whenever we are throwing errors that can reach the frontend.

## Tests

Don't add tests unless asked to. When you make a change that you think warrants tests (especially if related to something that already has existing tests somewhere), ask the user if you should add tests first.
