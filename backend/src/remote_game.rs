use std::collections::HashMap;

use crate::{
	game_engines::game_engine::GameEngine,
	serializable_struct,
	steam::id_lists::UevrScore,
};

serializable_struct!(RemoteGame {
	pub id: String,
	pub engine: Option<GameEngine>,
	pub uevr_score: Option<UevrScore>
});

pub type Map = HashMap<String, RemoteGame>;
