use std::{collections::HashMap, rc::Rc};

use rai_pal_backend::{
	game::Game, game_providers::game_provider::GameProviderId,
	local_database::game_database::GameId, operating_system::OperatingSystem, result::LogErrExt,
};

use slint::Model as _;

use crate::{EngineId, GameRowData};

#[derive(Clone, Debug, Default)]
pub struct ReleaseDate {
	pub day: String,
	pub month: String,
	pub year: String,
}

impl ReleaseDate {
	pub fn parse(date: Option<&str>) -> Self {
		let Some(date) = date else {
			return Self::default();
		};

		let date = date.get(0..10).unwrap_or(date);
		let mut parts = date.split('-');
		let year = parts.next().unwrap_or_default();
		let month = parts.next().map_or(0, |month| {
			month
				.parse::<u32>()
				.ok_or_log(&format!("Failed to parse release month {month}"))
				.unwrap_or_default()
		});
		let day = parts.next().unwrap_or_default();

		Self {
			day: day.to_string(),
			month: month_name(month).to_string(),
			year: year.to_string(),
		}
	}

	pub fn display_short(&self) -> String {
		if self.year.is_empty() {
			return String::new();
		}

		let mut parts = Vec::new();

		if !self.month.is_empty() {
			parts.push(self.month.clone());
		}

		if !self.day.is_empty() {
			parts.push(self.day.clone());
		}

		if parts.is_empty() {
			return self.year.clone();
		}

		format!("{}, {}", parts.join(" "), self.year)
	}
}

const MONTHS: [&str; 12] = [
	"January",
	"February",
	"March",
	"April",
	"May",
	"June",
	"July",
	"August",
	"September",
	"October",
	"November",
	"December",
];

fn month_name(month: u32) -> &'static str {
	match usize::try_from(month) {
		Ok(month) if (1..=MONTHS.len()).contains(&month) => MONTHS[month - 1],
		_ => "",
	}
}

pub fn make_row(game_id: &GameId) -> GameRowData {
	let mut row = GameRowData {
		provider: game_id.provider_id.into(),
		game_id: game_id.game_id.clone().into(),
		..GameRowData::default()
	};
	reset_row(&mut row);
	row
}

pub fn make_rows(game_ids: &[GameId]) -> Vec<GameRowData> {
	game_ids.iter().map(make_row).collect()
}

pub fn sync_rows(model: &slint::VecModel<GameRowData>, game_ids: &[GameId]) {
	let mut reusable: HashMap<(GameProviderId, String), GameRowData> = (0..model.row_count())
		.filter_map(|index| model.row_data(index))
		.filter_map(|row| {
			let provider = Option::<GameProviderId>::from(row.provider)?;
			Some(((provider, row.game_id.to_string()), row))
		})
		.collect();

	let rows = game_ids
		.iter()
		.map(|game_id| {
			reusable
				.remove(&(game_id.provider_id, game_id.game_id.clone()))
				.unwrap_or_else(|| make_row(game_id))
		})
		.collect::<Vec<_>>();

	let row_count = model.row_count();

	for (index, row) in rows.iter().enumerate() {
		if index < row_count {
			model.set_row_data(index, row.clone());
		} else {
			model.push(row.clone());
		}
	}

	while model.row_count() > rows.len() {
		model.remove(model.row_count() - 1);
	}
}

pub fn reset_row(row: &mut GameRowData) {
	row.title = "...".into();
	row.thumbnail = slint::Image::default();
	row.thumbnail_url = "".into();
	row.has_thumbnail = false;
	row.installed = false;
	row.tags = slint::ModelRc::default();
	row.loaded = false;
	row.engine = EngineId::None;
	row.engine_name = "".into();
	row.engine_version = "".into();
	row.engine_details = "".into();
	row.release_day = "".into();
	row.release_month = "".into();
	row.release_year = "".into();
}

pub fn apply_game_to_row(row: &mut GameRowData, game: &Game) {
	let mut tags = game
		.tags
		.iter()
		.map(|tag| tag.to_string().to_uppercase())
		.collect::<Vec<_>>();
	tags.sort();

	let release_date = ReleaseDate::parse(game.release_date_rfc3339.as_deref());

	row.title = game.full_title().into();
	row.thumbnail_url = game.thumbnail_url.clone().unwrap_or_default().into();
	row.installed = game.exe_path.is_some();
	row.tags = Rc::new(slint::VecModel::from(
		tags.into_iter()
			.map(slint::SharedString::from)
			.collect::<Vec<_>>(),
	))
	.into();
	row.engine = game.engine_brand.into();
	row.engine_name = game
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
	row.release_day = release_date.day.into();
	row.release_month = release_date.month.into();
	row.release_year = release_date.year.into();
	row.loaded = true;
}

fn get_engine_details(game: &Game) -> String {
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

	details.join(" ").to_lowercase()
}
