use std::sync::LazyLock;

use serde::Serialize;
use serde_json::{Map, Value, json};
use uuid::Uuid;

use crate::{http, result::Result};

const COLLECT_URL: &str = "https://events.raicuparta.com/rai-pal/collect";

static SESSION_ID: LazyLock<String> = LazyLock::new(|| Uuid::new_v4().hyphenated().to_string());

pub const MAX_EVENTS_PER_REQUEST: usize = 250;

#[derive(Debug)]
pub enum Event {
	InstallMod {
		mod_id: String,
		game: Option<String>,
	},
	UninstallMod {
		mod_id: String,
		game: Option<String>,
	},
	UpdateMod {
		mod_id: String,
		game: Option<String>,
	},
	RunMod {
		mod_id: String,
		game: Option<String>,
	},
	ProviderCommand {
		action: String,
		game: String,
	},
	StartApp {
		app_version: String,
	},
	UserSignIn,
	ErrorNotification {
		message: String,
	},
}

impl Event {
	fn into_wire(self) -> WireEvent {
		match self {
			Self::InstallMod { mod_id, game } => WireEvent::mod_event("InstallMod", mod_id, game),
			Self::UninstallMod { mod_id, game } => {
				WireEvent::mod_event("UninstallMod", mod_id, game)
			}
			Self::UpdateMod { mod_id, game } => WireEvent::mod_event("UpdateMod", mod_id, game),
			Self::RunMod { mod_id, game } => WireEvent::mod_event("RunMod", mod_id, game),
			Self::ProviderCommand { action, game } => {
				WireEvent::new("ProviderCommand", json!({ "action": action, "game": game }))
			}
			Self::StartApp { app_version } => WireEvent::new(
				"StartApp",
				json!({
					"app_version": app_version,
					"platform": std::env::consts::OS,
					"mode": if cfg!(debug_assertions) { "dev" } else { "release" },
				}),
			),
			Self::UserSignIn => WireEvent {
				id: "UserSignIn",
				data: None,
			},
			Self::ErrorNotification { message } => {
				WireEvent::new("ErrorNotification", json!({ "error_message": message }))
			}
		}
	}
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct Payload {
	client_id: Option<String>,
	session_id: Option<String>,
	events: Vec<WireEvent>,
}

#[derive(Debug, Serialize)]
struct WireEvent {
	id: &'static str,
	data: Option<Value>,
}

impl WireEvent {
	fn new(id: &'static str, data: Value) -> Self {
		Self {
			id,
			data: Some(data),
		}
	}

	fn mod_event(id: &'static str, mod_id: String, game: Option<String>) -> Self {
		let mut data = Map::new();
		data.insert("mod_id".to_string(), mod_id.into());

		if let Some(game) = game {
			data.insert("game".to_string(), game.into());
		}

		Self::new(id, Value::Object(data))
	}
}

pub async fn send_batch(events: Vec<Event>) -> Result<()> {
	let payload = Payload {
		client_id: None,
		session_id: Some(SESSION_ID.clone()),
		events: events
			.into_iter()
			.take(MAX_EVENTS_PER_REQUEST)
			.map(Event::into_wire)
			.collect(),
	};

	log::debug!("Sending analytics payload {payload:?}");

	http::CLIENT
		.post(COLLECT_URL)
		.json(&payload)
		.send()
		.await?
		.error_for_status()?;

	Ok(())
}
