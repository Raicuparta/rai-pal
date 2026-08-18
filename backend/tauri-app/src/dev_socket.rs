//! Dev-only debugging socket.
//!
//! Enabled only in debug builds. Opens a TCP listener on `127.0.0.1` so that
//! agents (or humans) can send arbitrary JavaScript to be evaluated inside the
//! app's webview, and read the JSON-serialized result back over the same socket.
//!
//! Protocol (newline-delimited JSON):
//!
//! ```text
//! → {"id": "...", "eval": "document.body.innerText"}
//! ← {"id": "...", "ok": true, "value": "..."}
//! ← {"id": "...", "ok": false, "error": "..."}
//! ```
//!
//! The evaluated code runs inside an async IIFE, so `await` is supported and
//! exceptions are caught and reported with their stack trace. The result is
//! reported back to Rust via a Tauri event.

use std::{
	collections::HashMap,
	sync::{
		Mutex,
		OnceLock,
		atomic::{
			AtomicU64,
			Ordering,
		},
	},
	time::Duration,
};

use rai_pal_core::result::LogErrExt;
use serde::Deserialize;
use serde_json::{
	Value,
	json,
};
use tauri::{
	Listener,
	WebviewWindow,
};
use tokio::{
	io::{
		AsyncBufReadExt,
		AsyncWriteExt,
		BufReader,
	},
	net::{
		TcpListener,
		TcpStream,
	},
	sync::mpsc,
};

const DEFAULT_PORT: u16 = 25_899;
const EVAL_TIMEOUT: Duration = Duration::from_mins(1);
const RESULT_EVENT: &str = "__dev_eval_result";

type ResultSender = mpsc::UnboundedSender<String>;

/// In-flight evals, keyed by id, each sending its result back to the
/// connection that requested it.
fn pending() -> &'static Mutex<HashMap<String, ResultSender>> {
	static PENDING: OnceLock<Mutex<HashMap<String, ResultSender>>> = OnceLock::new();
	PENDING.get_or_init(|| Mutex::new(HashMap::new()))
}

fn insert_pending(id: &str, sender: ResultSender) {
	if let Ok(mut map) = pending().lock() {
		map.insert(id.to_string(), sender);
	}
}

fn remove_pending(id: &str) -> Option<ResultSender> {
	pending().lock().ok().and_then(|mut map| map.remove(id))
}

fn next_id() -> String {
	static COUNTER: AtomicU64 = AtomicU64::new(1);
	format!("srv-{}", COUNTER.fetch_add(1, Ordering::Relaxed))
}

#[derive(Deserialize)]
struct EvalRequest {
	#[serde(default)]
	id: String,
	eval: String,
}

#[derive(Deserialize)]
struct EvalResult {
	id: String,
	ok: bool,
	#[serde(default)]
	value: Option<String>,
	#[serde(default)]
	error: Option<String>,
}

/// Starts the dev socket in the background. Does nothing in release builds
/// (the caller is expected to gate the call with `#[cfg(debug_assertions)]`).
pub fn start(window: WebviewWindow<tauri::Wry>) {
	// The JS we inject reports back by emitting an event. This is simpler than
	// registering a custom invoke command, which would otherwise be rejected as
	// "command not found" by the specta handler.
	window.listen(RESULT_EVENT, |event| {
		let Ok(args) = serde_json::from_str::<EvalResult>(event.payload()) else {
			log::error!("Dev socket: failed to parse eval result");
			return;
		};

		let response = if args.ok {
			let value = args
				.value
				.as_deref()
				.and_then(|raw| {
					serde_json::from_str(raw).ok_or_log("Failed to parse dev socket result value")
				})
				.unwrap_or(Value::Null);
			json!({ "id": args.id, "ok": true, "value": value })
		} else {
			json!({ "id": args.id, "ok": false, "error": args.error.unwrap_or_default() })
		};

		if let Some(sender) = remove_pending(&args.id) {
			let _ = sender.send(response.to_string());
		}
	});

	tauri::async_runtime::spawn(async move {
		if let Err(error) = run_listener(window).await {
			log::error!("Dev socket stopped: {error}");
		}
	});
}

async fn run_listener(window: WebviewWindow<tauri::Wry>) -> std::io::Result<()> {
	let port = std::env::var("RAI_PAL_DEV_SOCKET_PORT")
		.ok_or_log("Failed to read RAI_PAL_DEV_SOCKET_PORT")
		.and_then(|value| {
			value
				.parse()
				.ok_or_log("Invalid RAI_PAL_DEV_SOCKET_PORT, using default")
		})
		.unwrap_or(DEFAULT_PORT);

	let listener = TcpListener::bind(("127.0.0.1", port)).await?;

	println!("Dev socket listening on 127.0.0.1:{port}");
	log::info!("Dev socket listening on 127.0.0.1:{port}");

	loop {
		let (stream, _address) = listener.accept().await?;
		let window = window.clone();
		tauri::async_runtime::spawn(async move {
			if let Err(error) = handle_connection(stream, window).await {
				log::debug!("Dev socket connection error: {error}");
			}
		});
	}
}

async fn handle_connection(
	stream: TcpStream,
	window: WebviewWindow<tauri::Wry>,
) -> std::io::Result<()> {
	let (reader, mut writer) = stream.into_split();
	let mut lines = BufReader::new(reader).lines();

	while let Some(line) = lines.next_line().await? {
		let Some(request) = parse_request(&line) else {
			let _ = writer
				.write_all(
					b"{\"id\":null,\"ok\":false,\"error\":\"invalid request, expected JSON: {\\\"eval\\\": \\\"...\\\"}\"}\n",
				)
				.await;
			continue;
		};

		let id = if request.id.is_empty() {
			next_id()
		} else {
			request.id
		};

		let (sender, mut receiver) = mpsc::unbounded_channel();
		insert_pending(&id, sender);

		if let Err(error) = window.eval(wrap_eval(&id, &request.eval)) {
			remove_pending(&id);
			let response =
				json!({ "id": id, "ok": false, "error": format!("failed to eval: {error}") });
			let _ = writer.write_all(format!("{response}\n").as_bytes()).await;
			continue;
		}

		let response = match tokio::time::timeout(EVAL_TIMEOUT, receiver.recv()).await {
			Ok(Some(response)) => response,
			Ok(None) => json!({ "id": id, "ok": false, "error": "result channel closed" }).to_string(),
			Err(_) => json!({ "id": id, "ok": false, "error": format!("eval timed out after {}s", EVAL_TIMEOUT.as_secs()) }).to_string(),
		};

		remove_pending(&id);
		let _ = writer.write_all(response.as_bytes()).await;
		let _ = writer.write_all(b"\n").await;
	}

	Ok(())
}

fn parse_request(line: &str) -> Option<EvalRequest> {
	let line = line.trim();
	if line.is_empty() {
		return None;
	}
	serde_json::from_str(line).ok_or_log("Failed to parse dev socket request")
}

/// Wraps the user's JS in an async IIFE that evaluates it, serializes the
/// result to a JSON string, and reports it back to Rust by emitting an event.
fn wrap_eval(id: &str, js: &str) -> String {
	let js_json = serde_json::to_string(js).unwrap_or_default();
	let id_json = serde_json::to_string(id).unwrap_or_default();
	let event_json = serde_json::to_string(RESULT_EVENT).unwrap_or_default();

	format!(
		r#"(async () => {{
  const __dev_serialize = (value) => {{
    if (value === undefined) return "null";
    try {{
      const json = JSON.stringify(value);
      return json === undefined ? JSON.stringify(String(value)) : json;
    }} catch (error) {{
      return JSON.stringify(String(value));
    }}
  }};
  try {{
    const __dev_value = await eval({js_json});
    await window.__TAURI_INTERNALS__.invoke("plugin:event|emit", {{
      event: {event_json},
      payload: {{ id: {id_json}, ok: true, value: __dev_serialize(__dev_value) }}
    }});
  }} catch (error) {{
    await window.__TAURI_INTERNALS__.invoke("plugin:event|emit", {{
      event: {event_json},
      payload: {{ id: {id_json}, ok: false, error: String((error && error.stack) || error) }}
    }});
  }}
}})();"#
	)
}
