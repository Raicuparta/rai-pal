# Dev socket

Debug builds of Rai Pal open a TCP socket on `127.0.0.1` that evaluates
arbitrary JavaScript inside the app's webview and returns the result. This is
how agents (and humans) read the DOM and drive the UI, since the webview's
`console.*` output and DOM aren't visible from the terminal.

## How it works

- Only compiled in dev builds (`#[cfg(debug_assertions)]`), so it never ships
  to users.
- The socket lives in `backend/tauri-app/src/dev_socket.rs`.
- Protocol is newline-delimited JSON:

  ```text
  → {"id": "…", "eval": "document.body.innerText"}
  ← {"id": "…", "ok": true,  "value": "…"}
  ← {"id": "…", "ok": false, "error": "…"}
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

`RAI_PAL_DEV_SOCKET_PORT` overrides the port (default `25899`); the same env
var is read by the app and the client.

### Examples

```sh
npm run dev-socket -- "document.title"
npm run dev-socket -- "document.body.innerText"
npm run dev-socket -- "Array.from(document.querySelectorAll('button')).map(b => b.textContent)"
npm run dev-socket -- "await new Promise(r => setTimeout(r, 500)); document.querySelector('.mantine-Tabs-tab')?.textContent"
npm run dev-socket -- "document.querySelector('.some-button').click()"
```
