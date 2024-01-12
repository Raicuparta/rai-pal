use std::collections::HashSet;

use async_trait::async_trait;

use super::provider::{
	self,
	ProviderId,
};
use crate::{
	game_engines::game_engine::GameEngine,
	installed_game::InstalledGame,
	owned_game::OwnedGame,
	pc_gaming_wiki,
	provider::{
		ProviderActions,
		ProviderStatic,
	},
	Result,
};

pub struct Gog {
	engine_cache: provider::EngineCache,
}

impl ProviderStatic for Gog {
	const ID: &'static ProviderId = &ProviderId::Gog;

	fn new() -> Result<Self>
	where
		Self: Sized,
	{
		let engine_cache = Self::try_get_engine_cache();

		Ok(Self { engine_cache })
	}
}

#[async_trait]
impl ProviderActions for Gog {
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
		let owned_games =
			futures::future::join_all(game_scanner::gog::games().unwrap_or_default().iter().map(
				|game| async {
					OwnedGame {
						// TODO should add a constructor to OwnedGame to avoid ID collisions and stuff.
						id: game.id.clone(),
						provider_id: *Self::ID,
						name: game.name.clone(),
						installed: false, // TODO
						os_list: HashSet::default(),
						engine: get_engine(&game.id, &self.engine_cache).await,
						release_date: 0,                  // TODO
						thumbnail_url: String::default(), // TODO Maybe possible to get from the sqlite db?
						game_mode: None,
						uevr_score: None,
					}
				},
			))
			.await;

		Self::try_save_engine_cache(
			&owned_games
				.clone()
				.into_iter()
				.map(|owned_game| (owned_game.name.clone(), owned_game.engine))
				.collect(),
		);

		Ok(owned_games)
	}
}

async fn get_engine(gog_id: &str, cache: &provider::EngineCache) -> Option<GameEngine> {
	if let Some(cached_engine) = cache.get(gog_id) {
		return cached_engine.clone();
	}

	pc_gaming_wiki::get_engine(&format!("GOGcom_ID%20HOLDS%20%22{gog_id}%22")).await
}
