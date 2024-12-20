use std::{
	cmp::Ordering,
	collections::{HashMap, HashSet},
};

use rai_pal_proc_macros::serializable_struct;

use crate::{
	game_subscription::GameSubscription,
	game_tag::GameTag,
	game_title::GameTitle,
	installed_game::{GamesFilter, GamesSortBy, InstalledGame},
	providers::{
		provider::ProviderId,
		provider_command::{ProviderCommand, ProviderCommandAction},
	},
};

#[serializable_struct]
pub struct Game {
	pub id: String,
	pub provider_id: ProviderId,
	pub tags: HashSet<GameTag>,
	pub installed_game: Option<InstalledGame>,
	pub title: GameTitle,
	pub thumbnail_url: Option<String>,
	pub release_date: Option<i64>,
	pub provider_commands: HashMap<ProviderCommandAction, ProviderCommand>,
	pub from_subscriptions: HashSet<GameSubscription>,
}

impl Game {
	pub fn new(id: &str, provider_id: ProviderId, title: &str) -> Self {
		let title = GameTitle::new(title);
		let mut tags = HashSet::default();
		if title.is_probably_demo() {
			tags.insert(GameTag::Demo);
		}

		Self {
			id: id.to_string(),
			provider_id,
			tags,
			installed_game: None,
			title,
			thumbnail_url: None,
			release_date: None,
			provider_commands: HashMap::default(),
			from_subscriptions: HashSet::default(),
		}
	}

	pub fn add_tag(&mut self, tag: GameTag) -> &mut Self {
		self.tags.insert(tag);
		self
	}

	pub fn set_thumbnail_url(&mut self, thumbnail_url: &str) -> &mut Self {
		self.thumbnail_url = Some(thumbnail_url.to_string());
		self
	}

	pub fn set_release_date(&mut self, release_date: i64) -> &mut Self {
		self.release_date = Some(release_date);
		self
	}

	pub fn add_subscription(&mut self, subscription: GameSubscription) -> &mut Self {
		self.from_subscriptions.insert(subscription);
		self
	}

	pub fn add_provider_command(
		&mut self,
		command_action: ProviderCommandAction,
		command: ProviderCommand,
	) -> &mut Self {
		self.provider_commands.insert(command_action, command);
		self
	}
}

#[serializable_struct]
#[derive(Default)]
pub struct DataQuery {
	pub toggles: GamesFilter,
	pub search: String,
	pub sort_by: GamesSortBy,
	pub sort_descending: bool,
}

impl DataQuery {
	pub fn matches(&self, game: &Game) -> bool {
		let toggles = &self.toggles;
		if !toggles.providers.get(&game.provider_id).unwrap_or(&true) {
			return false;
		}

		// TODO: tags need to be merged from owned games.
		// if !toggles.tags.iter().any(|(tag, enabled)| {
		// 	*enabled && game.title.tags.contains(tag)
		// }) {
		// 	matches = false;
		// }

		let mut architectures = toggles.architectures.iter();
		if architectures.any(|(_, enabled)| !enabled)
			&& !toggles.architectures.iter().any(|(architecture, enabled)| {
				*enabled
					&& game.installed_game.as_ref().is_some_and(|installed_game| {
						installed_game
							.executable
							.architecture
							.is_some_and(|a| a == *architecture)
					})
			}) {
			return false;
		}

		let mut engines = toggles.engines.iter();
		if engines.any(|(_, enabled)| !enabled)
			&& !engines.any(|(engine, enabled)| {
				*enabled
					&& game.installed_game.as_ref().is_some_and(|installed_game| {
						installed_game
							.executable
							.engine
							.as_ref()
							.is_some_and(|e| e.brand == *engine)
					})
			}) {
			return false;
		}

		// let mut scripting_backends = toggles.unity_scripting_backends.iter();
		// if scripting_backends.any(|(_, enabled)| !enabled)
		// 	&& !scripting_backends.any(|(backend, enabled)| {
		// 		*enabled
		// 			&& game
		// 				.executable
		// 				.scripting_backend
		// 				.is_some_and(|b| b == *backend)
		// 	}) {
		// 	return false;
		// }

		// if !self.search.is_empty() {
		// 	// We'll try to match the search term to a bunch of different strings related to this game.
		// 	let mut candidates: Vec<&str> = vec![&game.title.display, &game.executable.name];
		// 	candidates.extend(game.title.normalized.iter().map(String::as_str));
		// 	if !any_contains(&candidates, &self.search) {
		// 		return false;
		// 	}
		// }

		true
	}

	pub fn sort(&self, game_a: &Game, game_b: &Game) -> Ordering {
		if game_a.installed_game.is_none() {
			return Ordering::Equal;
		}
		if game_b.installed_game.is_none() {
			return Ordering::Equal;
		}

		let a = game_a.installed_game.as_ref().unwrap();
		let b = game_b.installed_game.as_ref().unwrap();

		let ordering = match self.sort_by {
			GamesSortBy::Title => game_a.title.display.cmp(&game_a.title.display),
			GamesSortBy::Tags => Ordering::Equal,
			GamesSortBy::Provider => game_a
				.provider_id
				.to_string()
				.cmp(&game_a.provider_id.to_string()),
			GamesSortBy::Architecture => a
				.executable
				.architecture
				.and_then(|architecture_a| {
					b.executable.architecture.map(|architecture_b| {
						architecture_a.to_string().cmp(&architecture_b.to_string())
					})
				})
				.unwrap_or(Ordering::Equal),
			GamesSortBy::ScriptingBackend => a
				.executable
				.scripting_backend
				.and_then(|scripting_backend_a| {
					b.executable.scripting_backend.map(|scripting_backend_b| {
						scripting_backend_a
							.to_string()
							.cmp(&scripting_backend_b.to_string())
					})
				})
				.unwrap_or(Ordering::Equal),
			GamesSortBy::Engine => {
				a.executable
					.engine
					.as_ref()
					.and_then(|engine_a| {
						b.executable.engine.as_ref().map(|engine_b| {
							engine_a.brand.to_string().cmp(&engine_b.brand.to_string())
						})
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
