use std::{
	collections::HashMap,
	sync::LazyLock,
};

use log;
use rai_pal_proc_macros::serializable_enum;
use serde::Serialize;
use uuid::Uuid;

use crate::http;

const COLLECT_URL: &str = "https://events.raicuparta.com/rai-pal/collect";

static SESSION_ID: LazyLock<String> = LazyLock::new(|| Uuid::new_v4().hyphenated().to_string());

pub type AnalyticsData = HashMap<String, String>;

#[serializable_enum]
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
	events: Vec<AnalyticsEvent>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AnalyticsEvent {
	id: String,
	data: Option<serde_json::Value>,
}

pub async fn send_event(event: Event, data: Option<AnalyticsData>) {
	let payload = AnalyticsPayload {
		client_id: None,
		session_id: Some(SESSION_ID.to_string()),
		events: vec![AnalyticsEvent {
			id: event.to_string(),
			data: data.and_then(|d| {
				if d.is_empty() {
					None
				} else {
					Some(serde_json::to_value(d).unwrap_or_default())
				}
			}),
		}],
	};

	log::debug!("Analytics payload {payload:?}");

	let resp = http::CLIENT.post(COLLECT_URL).json(&payload).send().await;

	match resp {
		Ok(resp) => {
			if resp.status().is_success() {
				log::info!("Successfully Sent Analytics Event {event}");
			} else {
				log::error!("Couldn't Send Analytics Event {event}! {}", resp.status());
			}
		}
		Err(err) => {
			log::error!("Couldn't Send Analytics Event {event}! {err:?}");
		}
	}
}
