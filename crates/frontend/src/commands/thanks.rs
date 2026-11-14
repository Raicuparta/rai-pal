use slint::ComponentHandle as _;
use std::{rc::Rc, thread};

use rai_pal_backend::{http, result::LogErrExt};
use serde_json::Value;

use crate::{
	MainWindow, PatronData,
	fetch::{self, FetchedImage},
	runtime, text,
};

use super::CommandHandler;

const PATRONS_URL: &str = "https://raicuparta.github.io/patron-fetcher/patrons.json";
const AVATAR_SIZE: u32 = 64;

struct FetchedPatron {
	ranking: i32,
	name: String,
	initials: String,
	avatar: Option<FetchedImage>,
}

impl CommandHandler {
	pub fn load_patrons(&self) {
		load(self.window());
	}
}

fn load(window: slint::Weak<MainWindow>) {
	thread::spawn(move || {
		let patrons = runtime::run(fetch_patrons());

		crate::runtime::with_ui_window(&window, move |window| {
			let patrons = patrons
				.into_iter()
				.map(|patron| {
					let avatar = patron.avatar.map(FetchedImage::into_slint_image);

					PatronData {
						ranking: patron.ranking,
						name: patron.name.into(),
						initials: patron.initials.into(),
						has_avatar: avatar.is_some(),
						avatar: avatar.unwrap_or_default(),
					}
				})
				.collect::<Vec<_>>();

			window
				.global::<crate::ThanksState>()
				.set_patrons(Rc::new(slint::VecModel::from(patrons)).into());
		});
	});
}

async fn fetch_patrons() -> Vec<FetchedPatron> {
	let Some(active_patrons) = fetch_active_patrons().await else {
		return Vec::new();
	};

	let mut patrons = Vec::new();

	for value in &active_patrons {
		if let Some(patron) = patron_from_json(value).await {
			patrons.push(patron);
		}
	}

	patrons
}

async fn fetch_active_patrons() -> Option<Vec<Value>> {
	let database = http::CLIENT
		.get(PATRONS_URL)
		.send()
		.await
		.ok_or_log("Failed to request patrons")?
		.error_for_status()
		.ok_or_log("Patrons request failed")?
		.json::<Value>()
		.await
		.ok_or_log("Failed to parse patrons response")?;
	let patrons = database.get("activePatrons")?.as_array()?;

	Some(patrons.clone())
}

async fn patron_from_json(value: &Value) -> Option<FetchedPatron> {
	let name = value.get("name")?.as_str()?.to_string();

	if name.is_empty() {
		return None;
	}

	let ranking = i32::try_from(value.get("ranking")?.as_i64()?)
		.ok_or_log("Patron ranking does not fit in i32")?;
	let avatar = match value.get("imageUrl").and_then(Value::as_str) {
		Some(url) => fetch::fetch_image(url, AVATAR_SIZE, AVATAR_SIZE).await,
		None => None,
	};

	Some(FetchedPatron {
		ranking,
		initials: text::initials(&name).unwrap_or_default(),
		name,
		avatar,
	})
}
