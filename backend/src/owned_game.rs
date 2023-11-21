use std::collections::HashSet;

use crate::{
	game_engines::game_engine::GameEngineBrand,
	game_executable::OperatingSystem,
	providers::provider::ProviderId,
	serializable_struct,
};

serializable_struct!(OwnedGame {
	pub id: String,
	pub provider_id: ProviderId,
	pub name: String,
	pub installed: bool,
	pub os_list: HashSet<OperatingSystem>,
	pub engine: GameEngineBrand,
	pub release_date: i32,
	pub thumbnail_url: String,
});
