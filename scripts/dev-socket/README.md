# Dev socket

Debug builds of Rai Pal evaluate arbitrary JavaScript inside the app's webview
and return the result. This is how agents (and humans) read the DOM and drive
the UI, since the webview's `console.*` output and DOM aren't visible from the
terminal.

## How it works

- Dev commands are served through the app's **user socket** (see
  `backend/core/src/user/user_socket.rs`), as a special endpoint only registered
  in dev builds (`#[cfg(debug_assertions)]`), so it never ships to users. The
  command logic lives in `backend/tauri-app/src/dev_commands.rs`.
- The user socket uses a **dynamic port** in `43950..=43960`, so the client
  discovers it by probing `/check` for the `RAI PAL` phrase.
- Endpoint is `GET /dev/eval?code=<url-encoded JS>`, responding with JSON:

  ```text
  → GET /dev/eval?code=document.body.innerText
  ← {"ok": true,  "value": "…"}
  ← {"ok": false, "error": "…"}
  ```

- The JS runs inside an async IIFE, so `await` works and exceptions come back
  with their stack trace. Just write the expression — no `return` needed.

## Usage

Start the app (background it), then eval:

```sh
npm run dev &                           # starts Vite + the app (opens a window)
npm run dev-socket -- "document.title"  # one-shot eval
echo 'document.body.innerText' | npm run dev-socket   # pipe JS in
```

### Examples

```sh
npm run dev-socket -- "document.title"
npm run dev-socket -- "document.body.innerText"
npm run dev-socket -- "Array.from(document.querySelectorAll('button')).map(b => b.textContent)"
npm run dev-socket -- "await new Promise(r => setTimeout(r, 500)); document.querySelector('.mantine-Tabs-tab')?.textContent"
npm run dev-socket -- "document.querySelector('.some-button').click()"
```
