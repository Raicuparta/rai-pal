use std::{
	collections::HashMap,
	fs,
	path::PathBuf,
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

#[async_trait]
impl ProviderActions for ManualProvider {
	fn get_installed_games(
		&self,
		mod_loaders: &mod_loader::DataMap,
	) -> Result<installed_game::Map> {
		let games_json = app_data_path()?.join("games.json");

		// TODO: handle missing / empty file gracefully.
		let games_config = serde_json::from_str::<GamesConfig>(&fs::read_to_string(games_json)?)?;

		Ok(games_config
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

impl ProviderStatic for ManualProvider {
	const ID: &'static str = "manual";

	fn new() -> Result<Self>
	where
		Self: Sized,
	{
		Ok(Self {})
	}
}
