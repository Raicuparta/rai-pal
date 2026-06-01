#![cfg(target_os = "linux")]

use std::{
	fmt::Debug,
	path::PathBuf,
};

use serde::{
	Deserialize,
	Serialize,
};

use super::provider_command::ProviderCommandAction;
use crate::{
	game::DbGame,
	game_providers::{
		game_provider::{
			GameProviderId,
			ProviderActions,
			WineProviderActions,
		},
		heroic_provider,
	},
	local_database::game_database::{
		DbMutex,
		GameDatabase,
	},
	result::Result,
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
	Ok(
		heroic_provider::read_heroic_json::<Root>("store_cache/legendary_library.json")?
			.and_then(|root| root.library),
	)
}

#[derive(Clone)]
pub struct HeroicEpic;

impl HeroicEpic {
	fn get_exe_path(entry: &ParsedGame) -> Option<PathBuf> {
		let install = entry.install.as_ref()?;
		let game_path = install.install_path.as_ref()?;
		Some(game_path.join(install.executable.as_ref()?))
	}
}

impl ProviderActions for HeroicEpic {
	fn insert_games(&self, db: &DbMutex) -> Result {
		if let Some(parsed_games) = get_detected_games()? {
			for parsed_game in parsed_games {
				let mut game = DbGame::new(
					GameProviderId::Epic,
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
						heroic_provider::launch_command(&parsed_game.app_name, None),
					);
				}
				db.insert_game(&game);
			}
		}
		Ok(())
	}
}

impl WineProviderActions for HeroicEpic {
	fn set_wine_dll_overrides(&self, game: &DbGame, dll_overrides: &[String]) -> Result {
		heroic_provider::set_wine_dll_overrides(&game.external_id, dll_overrides)
	}
}
