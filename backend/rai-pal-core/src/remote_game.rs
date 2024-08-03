use std::collections::HashMap;

use rai_pal_proc_macros::serializable_struct;

use crate::{game_engines::game_engine::GameEngine, owned_game, providers::provider::ProviderId};

#[serializable_struct]
pub struct RemoteGame {
	pub id: String,
	pub engine: Option<GameEngine>,
}

pub type Map = HashMap<String, RemoteGame>;

impl RemoteGame {
	pub fn new(provider_id: ProviderId, provider_game_id: &str) -> Self {
		Self {
			id: owned_game::get_id(provider_id, provider_game_id),
			engine: None,
		}
	}

	pub fn set_engine(&mut self, engine: GameEngine) -> &mut Self {
		self.engine = Some(engine);
		self
	}
}
