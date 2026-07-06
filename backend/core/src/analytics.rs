use std::{
	sync::LazyLock,
	time::{
		SystemTime,
		UNIX_EPOCH,
	},
};

use log;
use rai_pal_proc_macros::{
	serializable_enum,
	serializable_struct,
};
use serde::Serialize;
use uuid::Uuid;

use crate::http;

const APP_ID: &str = "rai-pal";
const COLLECT_URL: &str = "https://events.raicuparta.com/collect";

static SESSION_ID: LazyLock<String> = LazyLock::new(|| Uuid::new_v4().hyphenated().to_string());

#[serializable_enum]
#[serde(rename_all = "snake_case")]
pub enum Event {
	InstallMod,
	UninstallMod,
	UpdateMod,
	RunMod,
	ProviderCommand,
	StartApp,
	UserSignIn,
	ErrorNotification,
}

#[serializable_struct]
pub struct AnalyticsData {
	param: Option<String>,
	game: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AnalyticsPayload {
	app_id: String,
	client_id: String,
	session_id: String,
	timestamp_millis: u128,
	events: Vec<AnalyticsEvent>,
}

#[derive(Debug, Serialize)]
struct AnalyticsEvent {
	id: String,
	data: Option<serde_json::Value>,
}

pub async fn send_event(event_name: Event, data: Option<AnalyticsData>) {
	let event_data = data.as_ref().and_then(|d| {
		if d.param.is_none() && d.game.is_none() {
			None
		} else {
			Some(serde_json::json!({
				"param": d.param,
				"game": d.game,
			}))
		}
	});

	let event_id = serde_json::to_value(event_name)
		.ok()
		.and_then(|v| v.as_str().map(String::from))
		.unwrap_or_default();

	let payload = AnalyticsPayload {
		app_id: APP_ID.to_string(),
		client_id: SESSION_ID.to_string(),
		session_id: SESSION_ID.to_string(),
		timestamp_millis: SystemTime::now()
			.duration_since(UNIX_EPOCH)
			.unwrap_or_default()
			.as_millis(),
		events: vec![AnalyticsEvent {
			id: event_id,
			data: event_data,
		}],
	};

	log::debug!("Analytics payload {payload:?}");

	let resp = http::CLIENT.post(COLLECT_URL).json(&payload).send().await;

	match resp {
		Ok(resp) => {
			if resp.status().is_success() {
				log::info!("Successfully Sent Analytics Event {event_name}");
			} else {
				log::error!(
					"Couldn't Send Analytics Event {event_name}! {}",
					resp.status()
				);
			}
		}
		Err(err) => {
			log::error!("Couldn't Send Analytics Event {event_name}! {err:?}");
		}
	}
}
