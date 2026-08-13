use std::collections::HashMap;

use rai_pal_proc_macros::{
	serializable_enum,
	serializable_struct,
};

use crate::{
	architecture::Architecture,
	game_engines::{
		game_engine::EngineBrand,
		unity::UnityBackend,
	},
	game_providers::game_provider::GameProviderId,
	game_tag::GameTag,
	operating_system::OperatingSystem,
};

#[serializable_enum]
pub enum InstallState {
	Installed,
	NotInstalled,
}

#[serializable_struct]
pub struct FilterItem {
	pub enabled: bool,
	pub locked: bool,
}

#[derive(serde::Serialize, serde::Deserialize, specta::Type, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FilterGroup<T: std::hash::Hash + std::cmp::Eq> {
	pub known: HashMap<T, FilterItem>,
	pub unknown: Option<FilterItem>,
}

impl<T: std::hash::Hash + std::cmp::Eq> Default for FilterGroup<T> {
	fn default() -> Self {
		Self {
			known: HashMap::new(),
			unknown: None,
		}
	}
}

#[serializable_struct]
#[derive(Default)]
pub struct GamesFilter {
	pub providers: FilterGroup<GameProviderId>,
	pub tags: FilterGroup<GameTag>,
	pub architectures: FilterGroup<Architecture>,
	pub unity_backends: FilterGroup<UnityBackend>,
	pub engines: FilterGroup<EngineBrand>,
	pub os: FilterGroup<OperatingSystem>,
	pub installed: FilterGroup<InstallState>,
	pub mod_families: FilterGroup<String>,
}

#[serializable_enum]
#[derive(Default)]
pub enum GamesSortBy {
	#[default]
	Title,
	Engine,
	ReleaseDate,
}

#[serializable_struct]
#[derive(Default)]
pub struct GamesQuery {
	pub filter: GamesFilter,
	pub search: String,
	pub sort_by: GamesSortBy,
	pub sort_descending: bool,
}
