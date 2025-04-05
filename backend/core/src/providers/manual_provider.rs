use std::{
	fs,
	path::{Path, PathBuf},
};

use log::error;
use rai_pal_proc_macros::serializable_struct;

use super::provider::{ProviderActions, ProviderId, ProviderStatic};
use crate::{
	game::{DbGame, GameId},
	paths::{app_data_path, file_name_without_extension},
	result::{Error, Result},
};

#[serializable_struct]
pub struct Manual {}

#[derive(serde::Serialize, serde::Deserialize)]
struct GamesConfig {
	pub paths: Vec<PathBuf>,
}

impl ProviderStatic for Manual {
	const ID: &'static ProviderId = &ProviderId::Manual;

	fn new() -> Result<Self>
	where
		Self: Sized,
	{
		Ok(Self {})
	}
}

impl ProviderActions for Manual {
	async fn insert_games(&self, pool: &sqlx::Pool<sqlx::Sqlite>) -> Result {
		Ok(())
	}

	async fn get_games<TCallback>(&self, mut callback: TCallback) -> Result
	where
		TCallback: FnMut(DbGame) + Send + Sync,
	{
		// for path in read_games_config(&games_config_path()?).paths {
		// 	match get_game_from_path(&path) {
		// 		Ok(game) => callback(game),
		// 		Err(error) => {
		// 			error!(
		// 				"Failed to get game from path '{}'. Will remove this path from the config. Error: {}",
		// 				path.display(),
		// 				error
		// 			);
		// 			remove_game(&path)?;
		// 		}
		// 	}
		// }

		Ok(())
	}
}

fn games_config_path() -> Result<PathBuf> {
	Ok(app_data_path()?.join("games.json"))
}

fn read_games_config(games_config_path: &Path) -> GamesConfig {
	match fs::read_to_string(games_config_path)
		.and_then(|games_config_file| Ok(serde_json::from_str::<GamesConfig>(&games_config_file)?))
	{
		Ok(games_config) => games_config,
		Err(error) => {
			error!("Error reading config: {error}");
			GamesConfig {
				paths: Vec::default(),
			}
		}
	}
}

// fn get_game_from_path(path: &Path) -> Result<DbGame> {
// 	let installed_game =
// 		InstalledGame::new(path).ok_or(Error::FailedToGetGameFromPath(path.to_path_buf()))?;
// 	let mut game = DbGame::new(
// 		GameId {
// 			game_id: installed_game.id.clone(),
// 			provider_id: ProviderId::Manual,
// 		},
// 		file_name_without_extension(path)?,
// 	);
// 	game.installed_game = Some(installed_game);
// 	Ok(game)
// }

pub fn add_game(path: &Path) -> Result<DbGame> {
	// let game = get_game_from_path(path)?;

	// let config_path = games_config_path()?;

	// let mut games_config = read_games_config(&config_path);
	// games_config.paths.push(path.to_path_buf());

	// fs::write(config_path, serde_json::to_string_pretty(&games_config)?)?;

	// Ok(game)
	Err(Error::FailedToGetGameFromPath(path.to_path_buf()))
}

pub fn remove_game(path: &Path) -> Result {
	let config_path = games_config_path()?;
	let mut games_config = read_games_config(&config_path);
	games_config.paths.retain(|p| p != path);

	fs::write(config_path, serde_json::to_string_pretty(&games_config)?)?;

	Ok(())
}
