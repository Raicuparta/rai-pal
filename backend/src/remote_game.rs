use std::collections::HashMap;

use crate::{
	game_engines::game_engine::GameEngine,
	owned_game,
	providers::provider::ProviderId,
	serializable_struct,
	steam::id_lists::UevrScore,
};

serializable_struct!(RemoteGame {
	pub id: String,
	pub engine: Option<GameEngine>,
	pub uevr_score: Option<UevrScore>
});

pub type Map = HashMap<String, RemoteGame>;

impl RemoteGame {
	pub fn new(provider_id: ProviderId, provider_game_id: &str) -> Self {
		Self {
			id: owned_game::get_id(provider_id, provider_game_id),
			engine: None,
			uevr_score: None,
		}
	}

	pub fn set_engine(&mut self, engine: GameEngine) -> &mut Self {
		self.engine = Some(engine);
		self
	}

	pub fn set_uevr_score(&mut self, uevr_score: UevrScore) -> &mut Self {
		self.uevr_score = Some(uevr_score);
		self
	}
}
