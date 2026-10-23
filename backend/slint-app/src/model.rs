use rai_pal_core::{
	game::DbGame, game_providers::game_provider::GameProviderId, operating_system::OperatingSystem,
};

use crate::GameRowData;

pub fn make_rows(game_ids: &[(GameProviderId, String)]) -> Vec<GameRowData> {
	game_ids
		.iter()
		.map(|(provider, id)| GameRowData {
			provider_id: format!("{provider}").into(),
			game_id: id.clone().into(),
			title: "...".into(),
			thumbnail: slint::Image::default(),
			has_thumbnail: false,
			loaded: false,
			engine_brand: "".into(),
			engine_version: "".into(),
			engine_details: "".into(),
			release_day: "".into(),
			release_month: "".into(),
			release_year: "".into(),
			game_json: "".into(),
		})
		.collect()
}

pub fn reset_row(row: &mut GameRowData) {
	row.title = "...".into();
	row.thumbnail = slint::Image::default();
	row.has_thumbnail = false;
	row.loaded = false;
	row.engine_brand = "".into();
	row.engine_version = "".into();
	row.engine_details = "".into();
	row.release_day = "".into();
	row.release_month = "".into();
	row.release_year = "".into();
	row.game_json = "".into();
}

pub fn apply_game_to_row(row: &mut GameRowData, game: &DbGame) {
	let (release_day, release_month, release_year) =
		format_release_date(game.release_date_rfc3339.as_deref());

	row.title = game.display_title.clone().into();
	row.engine_brand = game
		.engine_brand
		.map(|brand| brand.to_string())
		.unwrap_or_default()
		.into();
	row.engine_version = game
		.engine_version_display
		.clone()
		.unwrap_or_default()
		.into();
	row.engine_details = get_engine_details(game).into();
	row.release_day = release_day.into();
	row.release_month = release_month.into();
	row.release_year = release_year.into();
	row.game_json = serde_json::to_string_pretty(game)
		.unwrap_or_default()
		.into();
	row.loaded = true;
}

pub fn format_release_date(date: Option<&str>) -> (String, String, String) {
	let Some(date) = date else {
		return (String::new(), String::new(), String::new());
	};

	let date = date.get(0..10).unwrap_or(date);
	let mut parts = date.split('-');
	let year = parts.next().unwrap_or_default();
	let month_number = parts
		.next()
		.and_then(|month| month.parse::<u32>().ok())
		.unwrap_or_default();
	let day = parts.next().unwrap_or_default();

	let month = match month_number {
		1 => "January",
		2 => "February",
		3 => "March",
		4 => "April",
		5 => "May",
		6 => "June",
		7 => "July",
		8 => "August",
		9 => "September",
		10 => "October",
		11 => "November",
		12 => "December",
		_ => "",
	};

	(day.to_string(), month.to_string(), year.to_string())
}

pub fn get_engine_details(game: &DbGame) -> String {
	let mut details = Vec::new();

	if let Some(os) = game.executable_os {
		details.push(
			match os {
				OperatingSystem::Windows => "win",
				OperatingSystem::Linux => "linux",
			}
			.to_string(),
		);
	}

	if let Some(unity_backend) = game.unity_backend {
		details.push(unity_backend.to_string());
	}

	if let Some(architecture) = game.architecture {
		details.push(architecture.to_string());
	}

	details.join(" ")
}
