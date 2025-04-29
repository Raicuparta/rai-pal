#![cfg(target_os = "linux")]

use std::{
	fmt::Debug,
	fs::{self, read_to_string},
	io::{self},
	path::Path,
};

use crate::{
	game::{DbGame, InsertGame},
	game_executable::GameExecutable,
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
}

#[derive(Debug, Serialize, Deserialize)]
struct RootInstalled {
	installed: Vec<InstalledGOGGame>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ParsedGame {
	app_name: String,
	title: String,
	is_installed: bool,
	art_cover: Option<String>,
	folder_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Root {
	games: Vec<ParsedGame>,
}

fn get_detected_games() -> Result<Vec<ParsedGame>> {
	let dirs = BaseDirs::new()
		.ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Failed to get user directories"))?;
	let config_dir = dirs.config_dir();
	let file_content =
		read_to_string(Path::new(&config_dir).join("heroic/store_cache/gog_library.json"))?;
	let installed_games = read_installed_games()?;

	Ok(serde_json::from_str::<Root>(file_content.as_str())?
		.games
		.into_iter()
		// gog-redist is not a game but it shows up in the library
		.filter(|game| game.app_name != "gog-redist")
		.map(|mut game| {
			if installed_games.contains(&game.app_name) {
				// is_installed props from the library are not reliable
				game.is_installed = true;
			}
			game
		})
		.collect())
}

fn read_installed_games() -> Result<Vec<String>> {
	let dirs = BaseDirs::new().ok_or_else(Error::AppDataNotFound)?;
	let config_dir = dirs.config_dir();
	let file_content =
		read_to_string(Path::new(&config_dir).join("heroic/gog_store/installed.json"))?;

	Ok(
		serde_json::from_str::<RootInstalled>(file_content.as_str())?
			.installed
			.into_iter()
			.map(|game| game.app_name)
			.collect(),
	)
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

fn read_info_file(path: &Path, app_id: &str) -> Result<GogGame> {
	let file_path = path.join(format!("goggame-{app_id}.info"));

	let json_string = fs::read_to_string(file_path)?;
	Ok(serde_json::from_str::<GogGame>(&json_string)?)
}

#[derive(Clone)]
pub struct HeroicGog {}

impl HeroicGog {
	fn get_executable(entry: &ParsedGame) -> Option<GameExecutable> {
		let dirs = BaseDirs::new()?;
		let home_dir = dirs.home_dir();
		let game_path = Path::new(&home_dir)
			.join("Games/Heroic")
			.join(&entry.folder_name.clone()?);
		let infos = read_info_file(game_path.as_path(), &entry.app_name).ok()?;

		let executable_name = infos.play_tasks.iter().find_map(|task| {
			if task.is_primary.unwrap_or(false) {
				task.path.clone()
			} else {
				None
			}
		})?;

		GameExecutable::new(game_path.join(executable_name).as_path())
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
	async fn insert_games(&self, db: &std::sync::Mutex<rusqlite::Connection>) -> Result {
		let parsed_games = get_detected_games()?;
		for parsed_game in parsed_games {
			let mut game = DbGame::new(
				*Self::ID,
				parsed_game.app_name.clone(),
				parsed_game.title.clone(),
			);
			game.thumbnail_url = parsed_game.art_cover.clone();
			if let Some(executable) = Self::get_executable(&parsed_game) {
				game.set_executable(&executable);
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
