#![cfg(target_os = "linux")]

use super::provider_command::{ProviderCommand, ProviderCommandAction};
use crate::{
	game::DbGame,
	local_database::{DbMutex, GameDatabase},
	paths,
	providers::provider::{ProviderActions, ProviderId, ProviderStatic},
	result::Result,
};
use serde::{Deserialize, Serialize};
use std::{
	fmt::Debug,
	fs::read_to_string,
	path::{Path, PathBuf},
};

#[derive(Debug, Serialize, Deserialize)]
struct ParsedGame {
	app_name: String,
	title: String,
	install: Option<Install>,
	art_cover: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Install {
	executable: Option<String>,
	install_path: Option<PathBuf>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Root {
	library: Option<Vec<ParsedGame>>,
}

fn get_detected_games() -> Result<Option<Vec<ParsedGame>>> {
	let dirs = paths::base_dirs()?;
	let config_dir = dirs.config_dir();
	let path = Path::new(&config_dir).join("heroic/store_cache/legendary_library.json");
	if !path.try_exists()? {
		return Ok(None);
	}

	let file_content = read_to_string(path)?;

	Ok(serde_json::from_str::<Root>(file_content.as_str())?.library)
}

#[derive(Clone)]
pub struct HeroicEpic {}

impl HeroicEpic {
	fn get_exe_path(entry: &ParsedGame) -> Option<PathBuf> {
		let install = entry.install.as_ref()?;
		let game_path = install.install_path.as_ref()?;
		Some(game_path.join(install.executable.as_ref()?))
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
				if let Some(art_cover) = &parsed_game.art_cover {
					game.thumbnail_url = Some(format!("{art_cover}?h=100&resize=1"));
				}
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
