use std::{cmp::Ordering, collections::HashSet, string};

use rai_pal_proc_macros::{serializable_enum, serializable_struct};

use crate::{
	game::Game,
	game_engines::{game_engine::EngineBrand, unity::UnityScriptingBackend},
	game_executable::Architecture,
	game_tag::GameTag,
	providers::provider::ProviderId,
	string_includes::any_contains,
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
	pub unity_scripting_backends: HashSet<Option<UnityScriptingBackend>>,
	pub engines: HashSet<Option<EngineBrand>>,
	pub installed: HashSet<Option<InstallState>>,
}

#[serializable_enum]
pub enum GamesSortBy {
	Title,
	Tags,
	Provider,
	Architecture,
	ScriptingBackend,
	Engine,
	Installed,
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
			architectures: Architecture::variants().into_iter().map(Some).collect(),
			engines: EngineBrand::variants().into_iter().map(Some).collect(),
			providers: ProviderId::variants().into_iter().map(Some).collect(),
			tags: GameTag::variants().into_iter().map(Some).collect(),
			unity_scripting_backends: UnityScriptingBackend::variants()
				.into_iter()
				.map(Some)
				.collect(),
			installed: InstallState::variants().into_iter().map(Some).collect(),
		}
	}
}

impl GamesQuery {
	pub fn matches(&self, game: &Game) -> bool {
		let filter = &self.filter;

		if !filter.providers.contains(&Some(game.provider_id)) {
			return false;
		}

		// filter by tags. If the game has any of the tags, it's a match.
		// If the game has no tags and the None tag is in the filter, that's a match.
		// If the game has any tag that's in the filter tags, that's a match.
		if !filter.tags.is_empty() {
			if game.tags.is_empty() {
				if !filter.tags.contains(&None) {
					return false;
				}
			} else if !game
				.tags
				.iter()
				.any(|tag| filter.tags.contains(&Some(*tag)))
			{
				return false;
			}
		}

		if !filter.architectures.contains(
			&game
				.installed_game
				.as_ref()
				.and_then(|installed_game| installed_game.executable.architecture),
		) {
			return false;
		}

		if !filter
			.engines
			.contains(&game.get_engine().map(|engine| engine.brand))
		{
			return false;
		}

		if !filter.unity_scripting_backends.contains(
			&game
				.installed_game
				.as_ref()
				.and_then(|installed_game| installed_game.executable.scripting_backend),
		) {
			return false;
		}

		if !filter.installed.contains(if game.installed_game.is_some() {
			&Some(InstallState::Installed)
		} else {
			&Some(InstallState::NotInstalled)
		}) {
			return false;
		}

		if !self.search.is_empty() {
			// We'll try to match the search term to a bunch of different strings related to this game.
			let mut candidates: Vec<&str> = vec![&game.title.display];
			candidates.extend(game.title.normalized.iter().map(String::as_str));

			if let Some(installed_game) = game.installed_game.as_ref() {
				candidates.push(&installed_game.executable.name);
			}

			if !any_contains(&candidates, &self.search) {
				return false;
			}
		}

		true
	}

	pub fn sort(&self, game_a: &Game, game_b: &Game) -> Ordering {
		let ordering = match self.sort_by {
			GamesSortBy::Title => game_a.title.display.cmp(&game_b.title.display),
			GamesSortBy::Tags => {
				// TODO: reduce repetition in those whole sort function.
				// TODO: also, should probably sort the tags themselves so this is more stable.
				let string_a = format!(
					"{}{}",
					// We start with the tag count to make games with more tags show up first.
					game_a.tags.len(),
					game_a
						.tags
						.iter()
						.map(string::ToString::to_string)
						.collect::<String>()
				);

				let string_b = format!(
					"{}{}",
					game_b.tags.len(),
					game_b
						.tags
						.iter()
						.map(string::ToString::to_string)
						.collect::<String>()
				);

				string_a.cmp(&string_b)
			}
			GamesSortBy::Provider => game_a.provider_id.cmp(&game_b.provider_id),
			GamesSortBy::Architecture => {
				let architecture_a = game_a
					.installed_game
					.as_ref()
					.and_then(|installed_game_a| installed_game_a.executable.architecture);

				let architecture_b = game_b
					.installed_game
					.as_ref()
					.and_then(|installed_game_b| installed_game_b.executable.architecture);

				architecture_a.cmp(&architecture_b)
			}
			GamesSortBy::ScriptingBackend => {
				let unity_backend_a = game_a
					.installed_game
					.as_ref()
					.and_then(|installed_game_a| installed_game_a.executable.scripting_backend);

				let unity_backend_b = game_b
					.installed_game
					.as_ref()
					.and_then(|installed_game_b| installed_game_b.executable.scripting_backend);

				unity_backend_a.cmp(&unity_backend_b)
			}
			GamesSortBy::Engine => {
				// TODO: sort by engine version too.

				let engine_a = game_a.get_engine().map(|engine_a| engine_a.brand);

				let engine_b = game_b.get_engine().map(|engine_b| engine_b.brand);

				engine_a.cmp(&engine_b)
			}
			GamesSortBy::Installed => game_a
				.installed_game
				.is_some()
				.cmp(&game_b.installed_game.is_some()),
		};

		if self.sort_descending {
			ordering.reverse()
		} else {
			ordering
		}
	}
}
