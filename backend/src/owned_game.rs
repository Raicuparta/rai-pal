use std::collections::{
	HashMap,
	HashSet,
};

use crate::{
	game_engines::game_engine::GameEngine,
	game_executable::OperatingSystem,
	game_mode::GameMode,
	providers::provider::ProviderId,
	serializable_struct,
	steam::id_lists::UevrScore,
};

serializable_struct!(OwnedGame {
	pub id: String,
	pub provider_id: ProviderId,
	pub name: String,
	pub installed: bool,
	pub os_list: HashSet<OperatingSystem>,
	pub engine: Option<GameEngine>,
	pub release_date: i64,
	pub thumbnail_url: String,
	pub game_mode: Option<GameMode>,
	pub uevr_score: Option<UevrScore>,
});

pub type Map = HashMap<String, OwnedGame>;
