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
