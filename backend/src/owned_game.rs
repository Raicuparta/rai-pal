use std::collections::{
	HashMap,
	HashSet,
};

use crate::{
	game_engines::game_engine::GameEngineBrand,
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
	pub engine: Option<GameEngineBrand>,
	pub release_date: i32,
	pub thumbnail_url: String,
	pub game_mode: GameMode,
	pub uevr_score: Option<UevrScore>,
});

pub type Map = HashMap<String, OwnedGame>;
