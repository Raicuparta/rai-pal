use std::collections::HashSet;

use rai_pal_proc_macros::{serializable_enum, serializable_struct};
use strum::IntoEnumIterator;

use crate::{
	game_engines::{game_engine::EngineBrand, unity::UnityBackend},
	game_executable::Architecture,
	game_tag::GameTag,
	providers::provider::ProviderId,
};

#[serializable_enum]
pub enum InstallState {
	Installed,
	NotInstalled,
}

#[serializable_struct]
pub struct GamesFilter {
	pub providers: HashSet<Option<ProviderId>>,
	pub tags: HashSet<Option<GameTag>>,
	pub architectures: HashSet<Option<Architecture>>,
	pub unity_backends: HashSet<Option<UnityBackend>>,
	pub engines: HashSet<Option<EngineBrand>>,
	pub installed: HashSet<Option<InstallState>>,
}

#[serializable_enum]
pub enum GamesSortBy {
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

impl Default for GamesSortBy {
	fn default() -> Self {
		Self::Title
	}
}

impl Default for GamesFilter {
	fn default() -> Self {
		Self {
			architectures: Architecture::iter().map(Some).collect(),
			engines: EngineBrand::iter().map(Some).collect(),
			providers: ProviderId::iter().map(Some).collect(),
			tags: GameTag::iter().map(Some).collect(),
			unity_backends: UnityBackend::iter().map(Some).collect(),
			installed: InstallState::iter().map(Some).collect(),
		}
	}
}
