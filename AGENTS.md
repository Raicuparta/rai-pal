# Instructions for AI agents

Rai Pal is a Tauri v2 desktop app: Rust backend in `/backend`, TypeScript/React
frontend in `/frontend`.

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
