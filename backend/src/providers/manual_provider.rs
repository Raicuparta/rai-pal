use std::{
	fs,
	path::{
		Path,
		PathBuf,
	},
};

use async_trait::async_trait;

use super::provider::{
	ProviderActions,
	ProviderStatic,
};
use crate::{
	installed_game::{
		self,
		InstalledGame,
	},
	mod_loaders::mod_loader,
	owned_game::OwnedGame,
	paths::{
		app_data_path,
		file_name_without_extension,
		path_to_str,
	},
	serializable_struct,
	Error,
	Result,
};

serializable_struct!(ManualProvider {});

#[derive(serde::Serialize, serde::Deserialize)]
struct GamesConfig {
	pub paths: Vec<PathBuf>,
}

impl ProviderStatic for ManualProvider {
	const ID: &'static str = "manual";

	fn new() -> Result<Self>
	where
		Self: Sized,
	{
		Ok(Self {})
	}
}

#[async_trait]
impl ProviderActions for ManualProvider {
	fn get_installed_games(
		&self,
		mod_loaders: &mod_loader::DataMap,
	) -> Result<installed_game::Map> {
		Ok(read_games_config(&games_config_path()?)?
			.paths
			.iter()
			.filter_map(|path| {
				let game = create_game_from_path(path, mod_loaders)?;

				Some((game.id.clone(), game))
			})
			.collect())
	}

	async fn get_owned_games(&self) -> Result<Vec<OwnedGame>> {
		Ok(Vec::new())
	}
}

fn create_game_from_path(path: &Path, mod_loaders: &mod_loader::DataMap) -> Option<InstalledGame> {
	InstalledGame::new(
		path_to_str(path).ok()?,
		file_name_without_extension(path).ok()?,
		None,
		path,
		None,
		None,
		mod_loaders,
	)
}

fn games_config_path() -> Result<PathBuf> {
	Ok(app_data_path()?.join("games.json"))
}

fn read_games_config(games_config_path: &Path) -> Result<GamesConfig> {
	// TODO: handle missing / empty file gracefully.
	Ok(serde_json::from_str::<GamesConfig>(&fs::read_to_string(
		games_config_path,
	)?)?)
}

pub fn add_game(path: &Path, mod_loaders: &mod_loader::DataMap) -> Result<InstalledGame> {
	let game = create_game_from_path(path, mod_loaders)
		.ok_or(Error::FailedToGetGameFromPath(path.to_path_buf()))?;

	let config_path = games_config_path()?;

	let mut games_config = read_games_config(&config_path)?;
	// TODO: normalize and prevent adding repeated entries.
	games_config.paths.push(path.to_path_buf());

	fs::write(config_path, serde_json::to_string_pretty(&games_config)?)?;

	Ok(game)
}
