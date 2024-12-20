use std::{cmp::Ordering, collections::HashMap};

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
	pub providers: HashMap<ProviderId, bool>,
	pub tags: HashMap<GameTag, bool>,
	pub architectures: HashMap<Architecture, bool>,
	pub unity_scripting_backends: HashMap<UnityScriptingBackend, bool>,
	pub engines: HashMap<EngineBrand, bool>,
	pub installed: HashMap<InstallState, bool>,
}

#[serializable_enum]
pub enum GamesSortBy {
	Title,
	Tags,
	Provider,
	Architecture,
	ScriptingBackend,
	Engine,
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
			architectures: Architecture::variants()
				.into_iter()
				.map(|variant| (variant, true))
				.collect(),
			engines: EngineBrand::variants()
				.into_iter()
				.map(|variant| (variant, true))
				.collect(),
			providers: ProviderId::variants()
				.into_iter()
				.map(|variant| (variant, true))
				.collect(),
			tags: GameTag::variants()
				.into_iter()
				.map(|variant| (variant, true))
				.collect(),
			unity_scripting_backends: UnityScriptingBackend::variants()
				.into_iter()
				.map(|variant| (variant, true))
				.collect(),
			installed: InstallState::variants()
				.into_iter()
				.map(|variant| (variant, true))
				.collect(),
		}
	}
}

impl GamesQuery {
	pub fn matches(&self, game: &Game) -> bool {
		let filter = &self.filter;

		if !filter.providers.get(&game.provider_id).unwrap_or(&true) {
			return false;
		}

		if filter
			.tags
			.iter()
			.any(|(tag, enabled)| !enabled && game.tags.contains(tag))
		{
			return false;
		}

		if filter.architectures.iter().any(|(filter_arch, enabled)| {
			!enabled
				&& game
					.installed_game
					.as_ref()
					.and_then(|installed_game| installed_game.executable.architecture.as_ref())
					.is_some_and(|game_arch| game_arch == filter_arch)
		}) {
			return false;
		}

		if filter.engines.iter().any(|(engine_brand, enabled)| {
			!enabled
				&& game
					.installed_game
					.as_ref()
					.and_then(|installed_game| installed_game.executable.engine.as_ref())
					.is_some_and(|engine| engine.brand == *engine_brand)
		}) {
			return false;
		}

		if filter
			.unity_scripting_backends
			.iter()
			.any(|(filter_backend, enabled)| {
				!enabled
					&& game
						.installed_game
						.as_ref()
						.and_then(|installed_game| {
							installed_game.executable.scripting_backend.as_ref()
						})
						.is_some_and(|game_backend| game_backend == filter_backend)
			}) {
			return false;
		}

		let game_installed_state = if game.installed_game.is_some() {
			InstallState::Installed
		} else {
			InstallState::NotInstalled
		};
		if filter.installed.get(&game_installed_state).unwrap_or(&true) == &false {
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
			GamesSortBy::Title => game_a.title.display.cmp(&game_a.title.display),
			GamesSortBy::Tags => Ordering::Equal,
			GamesSortBy::Provider => game_a
				.provider_id
				.to_string()
				.cmp(&game_a.provider_id.to_string()),
			GamesSortBy::Architecture => {
				let a = game_a
					.installed_game
					.as_ref()
					.and_then(|installed_game_a| installed_game_a.executable.architecture);
				let b = game_b
					.installed_game
					.as_ref()
					.and_then(|installed_game_b| installed_game_b.executable.architecture);

				a.and_then(|architecture_a| {
					b.map(|architecture_b| {
						architecture_a.to_string().cmp(&architecture_b.to_string())
					})
				})
				.unwrap_or(Ordering::Equal)
			}
			GamesSortBy::ScriptingBackend => {
				let a = game_a
					.installed_game
					.as_ref()
					.and_then(|installed_game_a| installed_game_a.executable.scripting_backend);
				let b = game_b
					.installed_game
					.as_ref()
					.and_then(|installed_game_b| installed_game_b.executable.scripting_backend);

				a.and_then(|scripting_backend_a| {
					b.map(|scripting_backend_b| {
						scripting_backend_a
							.to_string()
							.cmp(&scripting_backend_b.to_string())
					})
				})
				.unwrap_or(Ordering::Equal)
			}
			GamesSortBy::Engine => {
				let a = game_a
					.installed_game
					.as_ref()
					.and_then(|installed_game_a| installed_game_a.executable.engine.as_ref());
				let b = game_b
					.installed_game
					.as_ref()
					.and_then(|installed_game_b| installed_game_b.executable.engine.as_ref());

				a.and_then(|engine_a| {
					b.map(|engine_b| engine_a.brand.to_string().cmp(&engine_b.brand.to_string()))
				})
				.unwrap_or(Ordering::Equal)
			}
		};

		if self.sort_descending {
			ordering.reverse()
		} else {
			ordering
		}
	}
}
