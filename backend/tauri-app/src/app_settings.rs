use std::{collections::HashSet, fs, path::PathBuf};

use rai_pal_core::{app_paths, games_query::GamesQuery, path_extensions::PathExt};
use rai_pal_proc_macros::{serializable_enum, serializable_struct};
use serde::Deserialize;

use crate::result::Result;

#[serializable_enum]
pub enum AppLocale {
	EnUs,
	EsEs,
	FrFr,
	DeDe,
	PtPt,
	ZhCn,
	JaJp,
	KoKr,
	WaWa,
}

#[serializable_enum]
#[derive(Default)]
pub enum TabId {
	#[default]
	Games,
	Mods,
	Thanks,
}

#[serializable_struct]
#[derive(Default)]
#[serde(default)]
pub struct AppSettings {
	pub hide_game_thumbnails: bool,
	pub override_language: Option<AppLocale>,
	#[serde(deserialize_with = "deserialize_games_query")]
	pub games_query: GamesQuery,
	pub selected_tab: TabId,
	pub skip_confirm_dialogs: HashSet<String>,
}

// Settings written before this field became non-optional may contain `null`.
fn deserialize_games_query<'de, D>(deserializer: D) -> std::result::Result<GamesQuery, D::Error>
where
	D: serde::Deserializer<'de>,
{
	Ok(Option::<GamesQuery>::deserialize(deserializer)?.unwrap_or_default())
}

// If the settings schema changes, update this so it gets recreated.
const SETTINGS_VERSION: u32 = 1u32;

impl AppSettings {
	fn try_read() -> Result<Self> {
		let path = Self::get_path()?;
		if !path.is_file() {
			return Ok(Self::default());
		}

		let data = fs::read_to_string(&path)?;

		Ok(serde_json::from_str(&data)?)
	}

	pub fn read() -> Self {
		Self::try_read().unwrap_or_else(|err| {
			log::error!("Failed to read app settings, resetting to default. Error: {err}");
			Self::default()
		})
	}

	pub fn try_write(&self) -> Result {
		let path = Self::get_path()?;
		fs::create_dir_all(path.try_parent()?)?;

		// Write to a temp file first so a crash mid-write can't corrupt settings.
		let temp_path = path.with_extension("json.tmp");
		fs::write(&temp_path, serde_json::to_string(self)?)?;
		fs::rename(&temp_path, &path)?;

		Ok(())
	}

	fn get_path() -> Result<PathBuf> {
		Ok(app_paths::app_data_file(&format!(
			"settings-{SETTINGS_VERSION}.json"
		))?)
	}
}

#[cfg(test)]
mod tests {
	#![allow(clippy::unwrap_used)]

	use super::AppSettings;

	#[test]
	fn null_games_query_defaults_to_all_filters_enabled() {
		let settings: AppSettings = serde_json::from_str(r#"{"gamesQuery":null}"#).unwrap();

		assert!(settings.games_query.filter.providers.known.is_empty());
		assert!(settings.games_query.filter.tags.known.is_empty());
	}

	#[test]
	fn missing_fields_fall_back_to_defaults() {
		let settings: AppSettings = serde_json::from_str("{}").unwrap();

		assert!(!settings.hide_game_thumbnails);
		assert!(settings.games_query.filter.providers.known.is_empty());
	}
}
