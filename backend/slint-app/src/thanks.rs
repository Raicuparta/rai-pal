use std::{rc::Rc, thread};

use serde_json::Value;

use crate::{
	MainWindow, PatronData,
	fetch::{self, FetchedImage},
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

		let _ = slint::invoke_from_event_loop(move || {
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

			window.set_patrons(Rc::new(slint::VecModel::from(patrons)).into());
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
		.ok()?
		.error_for_status()
		.ok()?;

	let database: Value = response.json().ok()?;
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
		initials: initials(&name),
		name,
		avatar,
	})
}

fn initials(name: &str) -> String {
	name.split_whitespace()
		.filter_map(|word| word.chars().next())
		.take(2)
		.collect::<String>()
		.to_uppercase()
}
