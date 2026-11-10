use slint::ComponentHandle as _;
use std::{rc::Rc, thread};

use rai_pal_backend::result::LogErrExt;
use serde_json::Value;

use crate::{
	MainWindow, PatronData,
	fetch::{self, FetchedImage},
	text,
};

const PATRONS_URL: &str = "https://raicuparta.github.io/patron-fetcher/patrons.json";
const AVATAR_SIZE: u32 = 64;

struct FetchedPatron {
	ranking: i32,
	name: String,
	initials: String,
	avatar: Option<FetchedImage>,
}

pub fn load(window: slint::Weak<MainWindow>) {
	thread::spawn(move || {
		let patrons = fetch_patrons();

		crate::runtime::invoke_on_ui(move || {
			let Some(window) = window.upgrade() else {
				return;
			};

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

fn fetch_patrons() -> Vec<FetchedPatron> {
	let Some(active_patrons) = fetch_active_patrons() else {
		return Vec::new();
	};

	active_patrons.iter().filter_map(patron_from_json).collect()
}

fn fetch_active_patrons() -> Option<Vec<Value>> {
	let response = fetch::http_client()
		.get(PATRONS_URL)
		.send()
		.ok_or_log("Failed to request patrons")?
		.error_for_status()
		.ok_or_log("Patrons request failed")?;

	let database: Value = response
		.json()
		.ok_or_log("Failed to parse patrons response")?;
	let patrons = database.get("activePatrons")?.as_array()?;

	Some(patrons.clone())
}

fn patron_from_json(value: &Value) -> Option<FetchedPatron> {
	let name = value.get("name")?.as_str()?.to_string();

	if name.is_empty() {
		return None;
	}

	let ranking = value.get("ranking")?.as_i64()? as i32;
	let avatar = value
		.get("imageUrl")
		.and_then(Value::as_str)
		.and_then(|url| fetch::fetch_image(url, AVATAR_SIZE, AVATAR_SIZE));

	Some(FetchedPatron {
		ranking,
		initials: text::initials(&name).unwrap_or_default(),
		name,
		avatar,
	})
}
