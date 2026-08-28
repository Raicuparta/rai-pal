//! Dev-only commands served through the user socket.
//!
//! Debug builds of Rai Pal register a `DevCommandHandler` with the user socket
//! (see `rai_pal_core::user::user_socket`) that evaluates arbitrary JavaScript
//! inside the app's webview and returns the JSON-serialized result. This lets
//! agents (or humans) read the DOM and drive the UI, since the webview's
//! `console.*` output and DOM aren't visible from the terminal.
//!
//! The handler is only registered in debug builds (`#[cfg(debug_assertions)]`),
//! so the endpoints it serves (e.g. `/dev/eval`) never exist in release builds.
//!
//! Protocol (`GET /dev/eval?code=<url-encoded JS>`):
//!
//! ```text
//! → GET /dev/eval?code=document.body.innerText
//! ← {"ok": true, "value": "..."}
//! ← {"ok": false, "error": "..."}
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

use rai_pal_core::{
	result::LogErrExt,
	user::user_socket::{
		DevCommandHandler,
		DevHttpResponse,
		set_dev_command_handler,
	},
};
use serde::Deserialize;
use serde_json::{
	Value,
	json,
};
use tauri::{
	Listener,
	WebviewWindow,
};
use tokio::sync::mpsc;

const EVAL_TIMEOUT: Duration = Duration::from_mins(1);
const RESULT_EVENT: &str = "__dev_eval_result";

type ResultSender = mpsc::UnboundedSender<String>;

/// In-flight evals, keyed by id, each sending its result back to the HTTP
/// request that started it.
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
struct EvalResult {
	id: String,
	ok: bool,
	#[serde(default)]
	value: Option<String>,
	#[serde(default)]
	error: Option<String>,
}

/// Registers the dev-mode command handler so the user socket can serve
/// `/dev/eval` in debug builds.
pub fn register(window: WebviewWindow<tauri::Wry>) {
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
			json!({ "ok": true, "value": value })
		} else {
			json!({ "ok": false, "error": args.error.unwrap_or_default() })
		};

		if let Some(sender) = remove_pending(&args.id) {
			let _ = sender.send(response.to_string());
		}
	});

	let handler: DevCommandHandler = Box::new(move |request_target| {
		let window = window.clone();
		Box::pin(async move { handle_request(&window, &request_target).await })
	});

	set_dev_command_handler(handler);
}

async fn handle_request(
	window: &WebviewWindow<tauri::Wry>,
	request_target: &str,
) -> Option<DevHttpResponse> {
	let (path, query) = request_target.split_once('?').unwrap_or((request_target, ""));

	if path != "/dev/eval" {
		return None;
	}

	let js = parse_query_param(query, "code");
	let Some(js) = js else {
		return Some(DevHttpResponse {
			status_code: 400,
			status_text: "Bad Request".to_string(),
			body: json!({ "ok": false, "error": "missing \"code\" query parameter" }).to_string(),
		});
	};

	let id = next_id();

	let (sender, mut receiver) = mpsc::unbounded_channel();
	insert_pending(&id, sender);

	if let Err(error) = window.eval(wrap_eval(&id, &js)) {
		remove_pending(&id);
		return Some(DevHttpResponse {
			status_code: 400,
			status_text: "Bad Request".to_string(),
			body: json!({ "ok": false, "error": format!("failed to eval: {error}") }).to_string(),
		});
	}

	let body = match tokio::time::timeout(EVAL_TIMEOUT, receiver.recv()).await {
		Ok(Some(response)) => response,
		Ok(None) => json!({ "ok": false, "error": "result channel closed" }).to_string(),
		Err(_) => json!({
			"ok": false,
			"error": format!("eval timed out after {}s", EVAL_TIMEOUT.as_secs())
		})
		.to_string(),
	};

	remove_pending(&id);

	Some(DevHttpResponse {
		status_code: 200,
		status_text: "OK".to_string(),
		body,
	})
}

fn parse_query_param(query: &str, key: &str) -> Option<String> {
	query
		.split('&')
		.find_map(|pair| {
			let (name, value) = pair.split_once('=')?;
			(name == key).then(|| value.to_string())
		})
		.and_then(|encoded| urlencoding::decode(&encoded).ok().map(|decoded| decoded.into_owned()))
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
