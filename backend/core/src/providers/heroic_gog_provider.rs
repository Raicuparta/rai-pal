#![cfg(target_os = "linux")]

use std::{
	collections::HashMap,
	fmt::Debug,
	fs::{self, read_to_string},
	io::{self},
	path::{Path, PathBuf},
};

use log;

use crate::{
	game::DbGame,
	local_database::{DbMutex, GameDatabase},
	providers::provider::{ProviderActions, ProviderId, ProviderStatic},
	result::{Error, Result},
};

use directories::BaseDirs;
use serde::{Deserialize, Serialize};

use super::provider_command::{ProviderCommand, ProviderCommandAction};

#[derive(Debug, Serialize, Deserialize)]
struct InstalledGOGGame {
	executable: String,
	#[serde(rename(deserialize = "appName"))]
	app_name: String,
	install_path: Option<PathBuf>,
}

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
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
		let dirs =
			BaseDirs::new().ok_or_else(|| io::Error::other("Failed to get user directories"))?;
		let config_dir = dirs.config_dir();
		let file_content =
			read_to_string(Path::new(&config_dir).join("heroic/store_cache/gog_library.json"))?;

		Ok(serde_json::from_str::<Root>(file_content.as_str())?
			.games
			.into_iter()
			// gog-redist is not a game but it shows up in the library
			.filter(|game| game.app_name != "gog-redist")
			.collect())
	}

	fn get_installed_games() -> Result<HashMap<String, InstalledGOGGame>> {
		let dirs = BaseDirs::new().ok_or_else(Error::AppDataNotFound)?;
		let config_dir = dirs.config_dir();
		let file_content =
			read_to_string(Path::new(&config_dir).join("heroic/gog_store/installed.json"))?;

		Ok(
			serde_json::from_str::<RootInstalled>(file_content.as_str())?
				.installed
				.into_iter()
				.map(|game| (game.app_name.clone(), game))
				.collect(),
		)
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

impl ProviderStatic for HeroicGog {
	const ID: &'static ProviderId = &ProviderId::Gog;

	fn new() -> Result<Self>
	where
		Self: Sized,
	{
		Ok(Self {})
	}
}

impl ProviderActions for HeroicGog {
	async fn insert_games(&self, db: &DbMutex) -> Result {
		let owned_games = Self::get_owned_games()?;
		let installed_games = Self::get_installed_games()?;

		for parsed_game in owned_games {
			let mut game = DbGame::new(
				*Self::ID,
				parsed_game.app_name.clone(),
				parsed_game.title.clone(),
			);
			game.thumbnail_url.clone_from(&parsed_game.art_icon);
			if let Some(installed_game) = installed_games.get(&parsed_game.app_name) {
				if let Some(exe_path) = Self::get_exe_path(installed_game) {
					game.set_executable(&exe_path);
				}
				game.add_provider_command(
					ProviderCommandAction::StartViaProvider,
					ProviderCommand::String(format!(
						"heroic://launch/gog/{}",
						parsed_game.app_name
					)),
				);
			}
			db.insert_game(&game);
		}

		Ok(())
	}
}
