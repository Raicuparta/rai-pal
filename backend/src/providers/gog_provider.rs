use std::collections::HashSet;

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

pub struct GogProvider {}

impl ProviderStatic for GogProvider {
	const ID: &'static ProviderId = &ProviderId::Gog;

	fn new() -> Result<Self>
	where
		Self: Sized,
	{
		Ok(Self {})
	}
}

#[async_trait]
impl ProviderActions for GogProvider {
	fn get_installed_games(&self) -> Result<Vec<InstalledGame>> {
		Ok(game_scanner::gog::games()
			.unwrap_or_default()
			.iter()
			.filter_map(|game| {
				InstalledGame::new(
					game.path.as_ref()?,
					&game.name,
					Self::ID.to_owned(),
					None,
					None,
					None,
				)
			})
			.collect())
	}

	async fn get_owned_games(&self) -> Result<Vec<OwnedGame>> {
		Ok(game_scanner::gog::games()
			.unwrap_or_default()
			.iter()
			.map(|game| OwnedGame {
				// TODO should add a constructor to OwnedGame to avoid ID collisions and stuff.
				id: game.id.clone(),
				provider_id: *Self::ID,
				name: game.name.clone(),
				installed: false, // TODO
				os_list: HashSet::default(),
				engine: None,
				release_date: 0,                  // TODO
				thumbnail_url: String::default(), // TODO Maybe possible to get from the sqlite db?
				game_mode: None,
				uevr_score: None,
			})
			.collect())
	}
}
