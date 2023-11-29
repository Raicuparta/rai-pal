use std::time::{
	SystemTime,
	UNIX_EPOCH,
};

use lazy_static::lazy_static;
use reqwest::Client;
use serde::Serialize;

const MEASUREMENT_ID: &str = "G-KTJZNR0ZET";
const API_KEY: Option<&str> = option_env!("ANALYTICS_API_KEY");

lazy_static! {
	static ref ANALYTICS_ID: String = uuid::Uuid::new_v4().hyphenated().to_string();
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum Event {
	InstallOrRunMod,
	StartApp,
	ManuallyAddGame,
}

#[derive(Debug, Serialize)]
struct AnalyticsEventParams {
	data: String,
	app_version: String,
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
	pub fn new(event_name: &Event, data: &str) -> Self {
		Self {
			client_id: ANALYTICS_ID.to_string(),
			timestamp_micros: SystemTime::now()
				.duration_since(UNIX_EPOCH)
				.unwrap_or_default()
				.as_micros(),
			non_personalized_ads: true,
			events: vec![AnalyticsEvent {
				name: event_name.to_owned(),
				params: AnalyticsEventParams {
					data: data.to_string(),
					app_version: env!("CARGO_PKG_VERSION").to_string(),
				},
			}],
		}
	}
}

pub async fn send_event(event_name: Event, data: &str) {
	if let Some(api_key) = API_KEY {
		let url = format!("https://www.google-analytics.com/mp/collect?measurement_id={MEASUREMENT_ID}&api_secret={api_key}");
		let client = Client::new();
		let payload = AnalyticsPayload::new(&event_name, data);
		println!("Sending {payload:?}");
		let resp = client.post(url).json(&payload).send().await;
		match resp {
			Ok(resp) => {
				if resp.status().is_success() {
					println!("Successfully Sent Analytics Event {event_name:?} for {data}");
				} else {
					eprintln!(
						"Couldn't Send Analytics Event For {}! {}",
						data,
						resp.status()
					);
				}
			}
			Err(err) => {
				eprintln!(
					"{}",
					format!("Couldn't Send Analytics Event For {data}! {err:?}")
						.replace(api_key, "***")
				);
			}
		}
	} else {
		println!("Skipping Analytics As The ANALYTICS_API_KEY Is Null ({event_name:?})");
	}
}
