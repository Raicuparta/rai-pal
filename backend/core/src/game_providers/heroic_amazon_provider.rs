#![cfg(target_os = "linux")]

use fs_err as fs;
use std::{
	fmt::Debug,
	path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

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
struct ParsedGame {
	app_name: String,
	title: String,
	art_cover: Option<String>,
	art_square: Option<String>,
	install: Option<Install>,
	#[serde(default)]
	is_installed: bool,
	extra: Option<Extra>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Install {
	install_path: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Fuel {
	main: FuelMain,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct FuelMain {
	command: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Extra {
	release_date: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Root {
	library: Option<Vec<ParsedGame>>,
}

fn get_detected_games() -> Result<Option<Vec<ParsedGame>>> {
	Ok(
		heroic_provider::read_heroic_json::<Root>("store_cache/nile_library.json")?
			.and_then(|root| root.library),
	)
}

#[derive(Clone)]
pub struct HeroicAmazon;

impl HeroicAmazon {
	// Amazon writes the launch command into `fuel.json` when installing, so the
	// executable comes from the game's own metadata instead of guessing.
	fn get_exe_path(install_path: &Path) -> Option<PathBuf> {
		let fuel_content = fs::read_to_string(install_path.join("fuel.json")).ok()?;
		let fuel = serde_json::from_str::<Fuel>(&fuel_content).ok()?;
		Some(install_path.join(fuel.main.command.replace('\\', "/")))
	}
}

impl ProviderActions for HeroicAmazon {
	fn insert_games(&self, db: &DbMutex) -> Result {
		if let Some(parsed_games) = get_detected_games()? {
			for parsed_game in parsed_games {
				let mut game = DbGame::new(
					GameProviderId::Amazon,
					parsed_game.app_name.clone(),
					parsed_game.title.clone(),
				);

				game.thumbnail_url = parsed_game
					.art_cover
					.clone()
					.or_else(|| parsed_game.art_square.clone());

				game.release_date_rfc3339 = parsed_game
					.extra
					.as_ref()
					.and_then(|extra| extra.release_date.as_deref())
					.and_then(heroic_provider::parse_release_date);

				if parsed_game.is_installed {
					game.add_provider_command(
						ProviderCommandAction::StartViaProvider,
						heroic_provider::launch_command(&parsed_game.app_name, Some("nile")),
					);
				}

				if let Some(exe_path) = parsed_game
					.install
					.as_ref()
					.and_then(|install| install.install_path.as_ref())
					.and_then(|install_path| Self::get_exe_path(install_path))
				{
					game.set_executable(&exe_path);
				}

				db.insert_game(&game);
			}
		}

		Ok(())
	}
}

impl WineProviderActions for HeroicAmazon {
	fn get_wine_prefix_path(&self, game: &DbGame) -> Result<PathBuf> {
		heroic_provider::get_wine_prefix_path(&game.external_id)
	}

	fn set_wine_dll_overrides(&self, game: &DbGame, dll_overrides: &[String]) -> Result {
		let prefix_path = self.get_wine_prefix_path(game)?;
		crate::wine::set_wine_dll_overrides_in_reg(&prefix_path, dll_overrides)
	}
}
