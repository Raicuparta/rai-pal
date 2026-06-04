use std::{
	sync::LazyLock,
	time::{
		SystemTime,
		UNIX_EPOCH,
	},
};

use log::{
	error,
	info,
};
use rai_pal_proc_macros::{
	serializable_enum,
	serializable_struct,
};
use serde::Serialize;
use uuid::Uuid;

use crate::http;

const MEASUREMENT_ID: &str = "G-KTJZNR0ZET";
const API_KEY: Option<&str> = option_env!("ANALYTICS_API_KEY");

static ANALYTICS_ID: LazyLock<String> = LazyLock::new(|| Uuid::new_v4().hyphenated().to_string());

#[serializable_enum]
#[serde(rename_all = "snake_case")]
pub enum Event {
	InstallMod,
	RunMod,
	ProviderCommand,
	StartApp,
	ManuallyAddGame,
}

#[derive(Debug, Serialize)]
struct AnalyticsEventParams {
	mod_id: String,
	game: String,
	app_version: String,
}

#[serializable_struct]
pub struct AnalyticsData {
	mod_id: Option<String>,
	game: Option<String>,
}

#[derive(Debug, Serialize)]
struct AnalyticsEvent {
	name: Event,
	params: AnalyticsEventParams,
}

#[derive(Debug, Serialize)]
struct AnalyticsPayload {
	client_id: String,
	timestamp_micros: u128,
	non_personalized_ads: bool,
	events: Vec<AnalyticsEvent>,
}

impl AnalyticsPayload {
	pub fn new(event_name: Event, data: &AnalyticsData) -> Self {
		Self {
			client_id: ANALYTICS_ID.to_string(),
			timestamp_micros: SystemTime::now()
				.duration_since(UNIX_EPOCH)
				.unwrap_or_default()
				.as_micros(),
			non_personalized_ads: true,
			events: vec![AnalyticsEvent {
				name: event_name,
				params: AnalyticsEventParams {
					mod_id: data.mod_id.clone().unwrap_or_default(),
					game: data.game.clone().unwrap_or_default(),
					app_version: env!("CARGO_PKG_VERSION").to_string(),
				},
			}],
		}
	}
}

pub async fn send_event(event_name: Event, data: Option<AnalyticsData>) {
	if let Some(api_key) = API_KEY {
		let url = format!(
			"https://www.google-analytics.com/mp/collect?measurement_id={MEASUREMENT_ID}&api_secret={api_key}"
		);
		let payload = AnalyticsPayload::new(
			event_name,
			&data.unwrap_or(AnalyticsData {
				mod_id: None,
				game: None,
			}),
		);
		info!("Sending {payload:?}");
		let resp = http::CLIENT.post(url).json(&payload).send().await;
		match resp {
			Ok(resp) => {
				if resp.status().is_success() {
					info!("Successfully Sent Analytics Event {event_name:?}");
				} else {
					error!(
						"Couldn't Send Analytics Event {event_name}! {}",
						resp.status()
					);
				}
			}
			Err(err) => {
				error!(
					"{}",
					format!("Couldn't Send Analytics Event {event_name}! {err:?}")
						.replace(api_key, "***")
				);
			}
		}
	} else {
		info!("Skipping Analytics As The ANALYTICS_API_KEY Is Null ({event_name:?})");
	}
}
