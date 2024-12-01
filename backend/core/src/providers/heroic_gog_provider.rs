#![cfg(target_os = "linux")]

use std::{
	fmt::Debug,
	fs::{self, read_to_string},
	io::{self},
	path::Path,
};

use crate::{
	installed_game::InstalledGame,
	owned_game::OwnedGame,
	providers::provider::{ProviderActions, ProviderId, ProviderStatic},
	result::Result as GameResult,
};

use directories::BaseDirs;
use serde::{Deserialize, Serialize};

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

fn get_detected_games() -> Result<Vec<ParsedGame>, io::Error> {
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
		.filter(|game|  game.app_name != "gog-redist")
		.map(|mut game| {
			if installed_games.contains(&game.app_name) {
				// is_installed props from the library are not reliable
				game.is_installed = true;
			}
			game
		})
		.collect())
}

fn read_installed_games() -> Result<Vec<String>, io::Error> {
	let dirs = BaseDirs::new()
		.ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Failed to get user directories"))?;
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

fn read_info_file(path: &Path, app_id: &str) -> Result<GogGame, String> {
	let file_path = path.join(format!("goggame-{app_id}.info"));

	match fs::read_to_string(file_path) {
		Ok(json_string) => match serde_json::from_str(&json_string) {
			Ok(json_value) => Ok(json_value),
			Err(err) => Err(format!("Error parsing JSON: {err}")),
		},
		Err(err) => Err(format!("Error reading file: {err}")),
	}
}

#[derive(Clone)]
pub struct HeroicGog {}

impl HeroicGog {
	fn get_installed_game(entry: &ParsedGame) -> Option<InstalledGame> {
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

		let mut game = InstalledGame::new(
			game_path.join(executable_name).as_path(),
			&entry.title,
			Self::ID.to_owned(),
		)?;

		game.set_start_command_string(&get_start_command("gog", &entry.app_name));
		game.set_provider_game_id(&entry.app_name);

		game.set_thumbnail_url(&entry.art_cover.clone()?);

		Some(game)
	}
}

impl ProviderStatic for HeroicGog {
	const ID: &'static ProviderId = &ProviderId::Gog;

	fn new() -> GameResult<Self>
	where
		Self: Sized,
	{
		Ok(Self {})
	}
}

impl ProviderActions for HeroicGog {
	async fn get_games<TInstalledCallback, TOwnedCallback>(
		&self,
		mut installed_callback: TInstalledCallback,
		mut owned_callback: TOwnedCallback,
	) -> GameResult
	where
		TInstalledCallback: FnMut(InstalledGame) + Send + Sync,
		TOwnedCallback: FnMut(OwnedGame) + Send + Sync,
	{
		let games = get_detected_games()?;
		for game in games {
			let mut owned_game = OwnedGame::new(
				&game.app_name,
				*Self::ID,
				&game.title,
			);
			if let Some(thumbnail_url) = game.art_cover.clone() {
				owned_game.set_thumbnail_url(&thumbnail_url);
			}
			owned_callback(owned_game);

			if game.is_installed {
				if let Some(installed_game) = Self::get_installed_game(&game) {
					installed_callback(installed_game);
				}
			}
		}

		Ok(())
	}
}

pub fn get_start_command(source: &str, app_id: &str) -> String {
	format!("heroic://launch/{source}/{app_id}")
}
