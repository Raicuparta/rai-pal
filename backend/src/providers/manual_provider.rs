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
	ProviderId,
	ProviderStatic,
};
use crate::{
	installed_game::InstalledGame,
	mod_loaders::mod_loader,
	owned_game::OwnedGame,
	paths::{
		app_data_path,
		file_name_without_extension,
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
	const ID: &'static ProviderId = &ProviderId::Manual;

	fn new() -> Result<Self>
	where
		Self: Sized,
	{
		Ok(Self {})
	}
}

#[async_trait]
impl ProviderActions for ManualProvider {
	fn get_installed_games(&self, mod_loaders: &mod_loader::DataMap) -> Result<Vec<InstalledGame>> {
		Ok(read_games_config(&games_config_path()?)
			.paths
			.iter()
			.filter_map(|path| create_game_from_path(path, mod_loaders))
			.collect())
	}

	async fn get_owned_games(&self) -> Result<Vec<OwnedGame>> {
		Ok(Vec::new())
	}
}

fn create_game_from_path(path: &Path, mod_loaders: &mod_loader::DataMap) -> Option<InstalledGame> {
	InstalledGame::new(
		path,
		file_name_without_extension(path).ok()?,
		*ManualProvider::ID,
		None,
		None,
		None,
		mod_loaders,
	)
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
			eprintln!("Error reading config: {error}");
			GamesConfig {
				paths: Vec::default(),
			}
		}
	}
}

pub fn add_game(path: &Path, mod_loaders: &mod_loader::DataMap) -> Result<InstalledGame> {
	let game = create_game_from_path(path, mod_loaders)
		.ok_or(Error::FailedToGetGameFromPath(path.to_path_buf()))?;

	let config_path = games_config_path()?;

	let mut games_config = read_games_config(&config_path);
	games_config.paths.push(path.to_path_buf());

	fs::write(config_path, serde_json::to_string_pretty(&games_config)?)?;

	Ok(game)
}
