use std::{
	collections::HashMap,
	sync::LazyLock,
	time::{
		SystemTime,
		UNIX_EPOCH,
	},
};

use log;
use rai_pal_proc_macros::serializable_enum;
use serde::Serialize;
use uuid::Uuid;

use crate::http;

const COLLECT_URL: &str = "https://raipal.events.raicuparta.com/collect";

static SESSION_ID: LazyLock<String> = LazyLock::new(|| Uuid::new_v4().hyphenated().to_string());

pub type AnalyticsData = HashMap<String, String>;

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

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AnalyticsPayload {
	client_id: Option<String>,
	session_id: Option<String>,
	timestamp_millis: u128,
	events: Vec<AnalyticsEvent>,
}

#[derive(Debug, Serialize)]
struct AnalyticsEvent {
	id: String,
	#[serde(skip_serializing_if = "Option::is_none")]
	data: Option<serde_json::Value>,
}

pub async fn send_event(event_name: Event, data: Option<AnalyticsData>) {
	let event_data = data.and_then(|d| {
		if d.is_empty() {
			None
		} else {
			Some(serde_json::to_value(d).unwrap_or_default())
		}
	});

	let event_id = serde_json::to_value(event_name)
		.ok()
		.and_then(|v| v.as_str().map(String::from))
		.unwrap_or_default();

	let payload = AnalyticsPayload {
		client_id: None,
		session_id: Some(SESSION_ID.to_string()),
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
