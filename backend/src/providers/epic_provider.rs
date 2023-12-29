use async_trait::async_trait;

use super::provider::ProviderId;
use crate::{
	installed_game::InstalledGame,
	owned_game::OwnedGame,
	provider::{
		ProviderActions,
		ProviderStatic,
	},
	Result,
};

pub struct EpicProvider {}

impl ProviderStatic for EpicProvider {
	const ID: &'static ProviderId = &ProviderId::Epic;

	fn new() -> Result<Self>
	where
		Self: Sized,
	{
		Ok(Self {})
	}
}

#[async_trait]
impl ProviderActions for EpicProvider {
	fn get_installed_games(&self) -> Result<Vec<InstalledGame>> {
		Ok(game_scanner::epicgames::games()
			.unwrap_or_default()
			.iter()
			.filter_map(|game| {
				if let Some(path) = game.path.clone().map(|path| {
					// TODO this is just presuming the exe name is the same as the folder,
					// which is a dumb thing to guess. Need to either find an exe inside this folder,
					// or see if epic keeps a reference to the full exe path somewhere.
					path.join(format!(
						"{}.exe",
						path.file_name().unwrap_or_default().to_string_lossy()
					))
				}) {
					return InstalledGame::new(
						&path,
						&game.name,
						Self::ID.to_owned(),
						None,
						None,
						None,
					);
				}
				None
			})
			.collect())
	}

	async fn get_owned_games(&self) -> Result<Vec<OwnedGame>> {
		Ok(Vec::default())
	}
}
