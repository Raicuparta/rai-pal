use std::{
	fs,
	path::{Path, PathBuf},
};

use log::error;
use rai_pal_proc_macros::serializable_struct;

use super::provider::{ProviderActions, ProviderId, ProviderStatic};
use crate::{
	local_database::{DbMutex, InsertGame},
	game::DbGame,
	paths::{self, app_data_path, file_name_without_extension},
	result::Result,
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
	async fn insert_games(&self, db: &DbMutex) -> Result {
		for path in read_games_config(&games_config_path()?).paths {
			match get_game_from_path(&path) {
				Ok(game) => {
					db.insert_game(&game);
				}
				Err(error) => {
					error!(
						"Failed to get game from path '{}'. Will remove this path from the config. Error: {}",
						path.display(),
						error
					);
					remove_game(&path)?;
				}
			}
		}

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

fn get_game_from_path(exe_path: &Path) -> Result<DbGame> {
	let mut game = DbGame::new(
		ProviderId::Manual,
		paths::hash_path(exe_path),
		file_name_without_extension(exe_path)?.to_string(),
	);
	game.set_executable(exe_path);
	Ok(game)
}

pub fn add_game(path: &Path) -> Result<DbGame> {
	let game = get_game_from_path(path)?;

	let config_path = games_config_path()?;

	let mut games_config = read_games_config(&config_path);
	games_config.paths.push(path.to_path_buf());

	fs::write(config_path, serde_json::to_string_pretty(&games_config)?)?;

	Ok(game)
}

pub fn remove_game(path: &Path) -> Result {
	let config_path = games_config_path()?;
	let mut games_config = read_games_config(&config_path);
	games_config.paths.retain(|p| p != path);

	fs::write(config_path, serde_json::to_string_pretty(&games_config)?)?;

	Ok(())
}
