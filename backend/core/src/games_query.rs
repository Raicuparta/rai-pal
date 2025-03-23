use std::{cmp::Ordering, collections::HashSet};

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

		if !filter.providers.contains(&Some(game.id.provider_id)) {
			return false;
		}

		// Tags filtering is a bit different, because games can have multiple tags.
		// Since filters are only checkboxes, we have to make a decision on what the checks mean.
		// Since players are more likely to want to hide games with specific tags, we make it so
		// if a game includes any tag that's unchecked, that game gets filtered out.
		// This means that currently there's no way to find for instance games that have both
		// tag A and tag B.
		if !filter.tags.is_empty() {
			if game.tags.is_empty() {
				if !filter.tags.contains(&None) {
					return false;
				}
			} else {
				let enabled_tags: HashSet<_> = filter.tags.iter().filter_map(|tag| *tag).collect();
				if !game.tags.iter().all(|tag| enabled_tags.contains(tag)) {
					return false;
				}
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
			candidates.push(game.external_id.as_str());

			// if let Some(installed_game) = game.installed_game.as_ref() {
			// 	candidates.push(&installed_game.executable.name);
			// }

			if !any_contains(&candidates, &self.search) {
				return false;
			}
		}

		true
	}

	pub fn sort(&self, game_a: &Game, game_b: &Game) -> Ordering {
		let ordering = match self.sort_by {
			GamesSortBy::Title => game_a
				.title
				.display
				.to_lowercase()
				.cmp(&game_b.title.display.to_lowercase()),
			GamesSortBy::Engine => game_a.get_engine().cmp(&game_b.get_engine()),
			GamesSortBy::ReleaseDate => game_a.release_date.cmp(&game_b.release_date),
		};

		if self.sort_descending {
			ordering.reverse()
		} else {
			ordering
		}
	}
}
