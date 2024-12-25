use std::{
	fs,
	path::{Path, PathBuf},
};

use log::error;
use rai_pal_proc_macros::serializable_struct;

use super::provider::{ProviderActions, ProviderId, ProviderStatic};
use crate::{
	game::Game,
	installed_game::InstalledGame,
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
	// async fn get_games<TInstalledCallback, TOwnedCallback>(
	// 	&self,
	// 	mut installed_callback: TInstalledCallback,
	// 	mut _owned_callback: TOwnedCallback,
	// ) -> Result
	// where
	// 	TInstalledCallback: FnMut(InstalledGame) + Send + Sync,
	// 	TOwnedCallback: FnMut(OwnedGame) + Send + Sync,
	// {
	// 	for path in read_games_config(&games_config_path()?).paths {
	// 		if let Some(installed_game) = create_game_from_path(&path) {
	// 			installed_callback(installed_game);
	// 		}
	// 	}

	// 	Ok(())
	// }

	async fn get_games_new<TCallback>(&self, callback: TCallback) -> Result
	where
		TCallback: FnMut(Game) + Send + Sync,
	{
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

pub fn add_game(path: &Path) -> Result<InstalledGame> {
	let game =
		InstalledGame::new(path).ok_or(Error::FailedToGetGameFromPath(path.to_path_buf()))?;

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
