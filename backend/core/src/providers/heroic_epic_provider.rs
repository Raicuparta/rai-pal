#![cfg(target_os = "linux")]

use std::{
	fmt::Debug,
	fs::read_to_string,
	io::{self},
	path::Path,
};

use crate::{
	game::Game,
	installed_game::InstalledGame,
	providers::provider::{ProviderActions, ProviderId, ProviderStatic},
	result::Result as GameResult,
};

use directories::BaseDirs;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct InstalledEpicGame {
	executable: String,
	#[serde(rename(deserialize = "appName"))]
	app_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct RootInstalled {
	installed: Vec<InstalledEpicGame>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ParsedGame {
	app_name: String,
	title: String,
	install: Install,
	is_installed: bool,
	art_cover: String,
	folder_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Install {
	executable: Option<String>,
	install_path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Root {
	library: Option<Vec<ParsedGame>>,
}

fn get_detected_games() -> Result<Option<Vec<ParsedGame>>, io::Error> {
	let dirs = BaseDirs::new()
		.ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Failed to get user directories"))?;
	let config_dir = dirs.config_dir();
	let file_content =
		read_to_string(Path::new(&config_dir).join("heroic/store_cache/legendary_library.json"))?;

	Ok(serde_json::from_str::<Root>(file_content.as_str())?.library)
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
struct EpicGame {
	game_id: String,
	name: String,
	play_tasks: Vec<PlayTask>,
}

#[derive(Clone)]
pub struct HeroicEpic {}

impl HeroicEpic {
	fn get_installed_game(entry: &ParsedGame) -> Option<InstalledGame> {
		let dirs = BaseDirs::new()?;
		let home_dir = dirs.home_dir();
		let game_path = Path::new(&home_dir)
			.join("Games/Heroic")
			.join(&entry.folder_name.clone()?);

		if let Some(executable_name) = &entry.install.executable {
			let mut game = InstalledGame::new(game_path.join(executable_name).as_path())?;

			game.set_start_command_string(&get_start_command(&entry.app_name));

			return Some(game);
		}

		None
	}
}

impl ProviderStatic for HeroicEpic {
	const ID: &'static ProviderId = &ProviderId::Epic;

	fn new() -> GameResult<Self>
	where
		Self: Sized,
	{
		Ok(Self {})
	}
}

impl ProviderActions for HeroicEpic {
	async fn get_games<TCallback>(&self, mut callback: TCallback) -> GameResult
	where
		TCallback: FnMut(Game) + Send + Sync,
	{
		if let Some(parsed_games) = get_detected_games()? {
			for parsed_game in parsed_games {
				let mut game = Game::new(&parsed_game.app_name, *Self::ID, &parsed_game.title);
				game.set_thumbnail_url(&parsed_game.art_cover);
				game.installed_game = Self::get_installed_game(&parsed_game);
				callback(game);
			}
		}
		Ok(())
	}
}

pub fn get_start_command(app_id: &str) -> String {
	format!("heroic://launch/{app_id}")
}
