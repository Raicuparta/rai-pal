#![cfg(target_os = "linux")]

use std::{
	collections::HashMap,
	fmt::Debug,
	fs,
	path::{Path, PathBuf},
};

use log;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::provider_command::ProviderCommandAction;
use crate::{
	game::DbGame,
	game_providers::{
		game_provider::{GameProviderId, ProviderActions, WineProviderActions},
		heroic_provider,
	},
	local_database::{app_database::DbMutex, game_database::GameDatabase},
	result::Result,
};

#[derive(Debug, Serialize, Deserialize)]
struct InstalledGOGGame {
	#[serde(rename(deserialize = "appName"))]
	app_name: String,
	install_path: Option<PathBuf>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct RootInstalled {
	installed: Vec<InstalledGOGGame>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ParsedGame {
	app_name: String,
	title: String,
	art_cover: Option<String>,
	art_square: Option<String>,
	art_icon: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GogApiInfo {
	first_release_date: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct Root {
	games: Vec<ParsedGame>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct PlayTask {
	is_primary: Option<bool>,
	name: String,
	path: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct GogGame {
	game_id: String,
	name: String,
	play_tasks: Vec<PlayTask>,
}

#[derive(Clone)]
pub struct HeroicGog {}

impl HeroicGog {
	fn get_owned_games() -> Result<Vec<ParsedGame>> {
		Ok(
			heroic_provider::read_heroic_json::<Root>("store_cache/gog_library.json")?
				.unwrap_or_default()
				.games
				.into_iter()
				// gog-redist is not a game but it shows up in the library
				.filter(|game| game.app_name != "gog-redist")
				.collect(),
		)
	}

	fn get_installed_games() -> Result<HashMap<String, InstalledGOGGame>> {
		Ok(
			heroic_provider::read_heroic_json::<RootInstalled>("gog_store/installed.json")?
				.unwrap_or_default()
				.installed
				.into_iter()
				.map(|game| (game.app_name.clone(), game))
				.collect(),
		)
	}

	// Heroic caches the GOG store API response here, and it's the only local file
	// with release dates (`gog_library.json` only has library metadata).
	fn get_release_dates() -> Result<HashMap<String, String>> {
		let Some(api_info) = heroic_provider::read_heroic_json::<HashMap<String, Value>>(
			"store_cache/gog_api_info.json",
		)?
		else {
			return Ok(HashMap::new());
		};

		Ok(api_info
			.into_iter()
			.filter_map(|(key, value)| {
				let app_name = key.strip_prefix("gog_")?.to_string();
				let release_date = serde_json::from_value::<GogApiInfo>(value)
					.ok()?
					.first_release_date?;
				heroic_provider::parse_release_date(&release_date).map(|date| (app_name, date))
			})
			.collect())
	}

	fn read_info_file(path: &Path, app_id: &str) -> Result<GogGame> {
		let json_string = fs::read_to_string(path.join(format!("goggame-{app_id}.info")))?;
		Ok(serde_json::from_str::<GogGame>(&json_string)?)
	}

	fn get_exe_path(entry: &InstalledGOGGame) -> Option<PathBuf> {
		let game_path = entry.install_path.as_ref()?;
		let infos = match Self::read_info_file(game_path, &entry.app_name) {
			Ok(infos) => infos,
			Err(err) => {
				log::warn!(
					"Failed to read info file for GOG game '{}': {}",
					entry.app_name,
					err
				);
				return None;
			}
		};

		let executable_name = infos.play_tasks.iter().find_map(|task| {
			if task.is_primary.unwrap_or(false) {
				task.path.clone()
			} else {
				None
			}
		})?;

		Some(game_path.join(executable_name))
	}
}

impl ProviderActions for HeroicGog {
	fn insert_games(&self, db: &DbMutex) -> Result {
		let owned_games = Self::get_owned_games()?;
		let installed_games = Self::get_installed_games()?;
		let release_dates = Self::get_release_dates()?;

		for parsed_game in owned_games {
			let mut game = DbGame::new(
				GameProviderId::Gog,
				parsed_game.app_name.clone(),
				parsed_game.title.clone(),
			);
			game.thumbnail_url.clone_from(&parsed_game.art_icon);
			game.release_date_rfc3339 = release_dates.get(&parsed_game.app_name).cloned();
			if let Some(installed_game) = installed_games.get(&parsed_game.app_name) {
				if let Some(exe_path) = Self::get_exe_path(installed_game) {
					game.set_executable(&exe_path);
				}
				game.add_provider_command(
					ProviderCommandAction::StartViaProvider,
					heroic_provider::launch_command(&parsed_game.app_name, Some("gog")),
				);
			}
			db.insert_game(&game);
		}

		Ok(())
	}
}

impl WineProviderActions for HeroicGog {
	fn set_wine_dll_overrides(&self, game: &DbGame, dll_overrides: &[String]) -> Result {
		heroic_provider::set_wine_dll_overrides(&game.external_id, dll_overrides)
	}
}
