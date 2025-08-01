#![cfg(target_os = "linux")]

use std::{
	fmt::Debug,
	fs::read_to_string,
	path::{Path, PathBuf},
};

use crate::{
	game::DbGame,
	local_database::{DbMutex, GameDatabase},
	paths,
	providers::provider::{ProviderActions, ProviderId, ProviderStatic},
	result::Result,
};

use directories::BaseDirs;
use serde::{Deserialize, Serialize};

use super::provider_command::{ProviderCommand, ProviderCommandAction};

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

fn get_detected_games() -> Result<Option<Vec<ParsedGame>>> {
	let dirs = paths::base_dirs()?;
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
	fn get_exe_path(entry: &ParsedGame) -> Option<PathBuf> {
		let dirs = BaseDirs::new()?;
		let home_dir = dirs.home_dir();
		let game_path = Path::new(&home_dir)
			.join("Games/Heroic")
			.join(&entry.folder_name.clone()?);

		entry
			.install
			.executable
			.as_ref()
			.map(|executable_name| game_path.join(executable_name))
	}
}

impl ProviderStatic for HeroicEpic {
	const ID: &'static ProviderId = &ProviderId::Epic;

	fn new() -> Result<Self>
	where
		Self: Sized,
	{
		Ok(Self {})
	}
}

impl ProviderActions for HeroicEpic {
	async fn insert_games(&self, db: &DbMutex) -> Result {
		if let Some(parsed_games) = get_detected_games()? {
			for parsed_game in parsed_games {
				let mut game = DbGame::new(
					*Self::ID,
					parsed_game.app_name.clone(),
					parsed_game.title.clone(),
				);
				game.thumbnail_url = Some(parsed_game.art_cover.clone());
				if let Some(exe_path) = Self::get_exe_path(&parsed_game) {
					game.set_executable(&exe_path);
					game.add_provider_command(
						ProviderCommandAction::StartViaProvider,
						ProviderCommand::String(format!(
							"heroic://launch/{}",
							parsed_game.app_name
						)),
					);
				}
				db.insert_game(&game);
			}
		}
		Ok(())
	}
}
