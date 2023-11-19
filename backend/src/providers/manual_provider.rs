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
	},
	serializable_struct,
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
			.filter_map(|file_path| {
				let id = file_path.to_string_lossy().to_string();

				Some((
					id.clone(),
					InstalledGame::new(
						&id,
						file_name_without_extension(file_path).ok()?,
						None,
						file_path,
						None,
						None,
						mod_loaders,
					)?,
				))
			})
			.collect())
	}

	async fn get_owned_games(&self) -> Result<Vec<OwnedGame>> {
		Ok(Vec::new())
	}
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

pub fn add_game(path: &Path) -> Result {
	let config_path = games_config_path()?;

	let mut games_config = read_games_config(&config_path)?;
	games_config.paths.push(path.to_path_buf());

	fs::write(config_path, serde_json::to_string_pretty(&games_config)?)?;

	Ok(())
}
