use std::collections::{HashMap, HashSet};

use rai_pal_proc_macros::serializable_struct;

use crate::{
	game_engines::game_engine::GameEngine,
	game_subscription::GameSubscription,
	game_tag::GameTag,
	game_title::GameTitle,
	installed_game::InstalledGame,
	providers::{
		provider::ProviderId,
		provider_command::{ProviderCommand, ProviderCommandAction},
	},
	remote_game::RemoteGame,
};

#[serializable_struct]
pub struct Game {
	// ID used to find this game in provider APIs and stuff.
	pub external_id: String,

	// ID used to uniquely identify this game in Rai Pal, among all games of the same provider.
	pub unique_id: String,

	pub provider_id: ProviderId,
	pub tags: HashSet<GameTag>,
	pub installed_game: Option<InstalledGame>,
	pub remote_game: Option<RemoteGame>,
	pub title: GameTitle,
	pub thumbnail_url: Option<String>,
	pub release_date: Option<i64>,
	pub provider_commands: HashMap<ProviderCommandAction, ProviderCommand>,
	pub from_subscriptions: HashSet<GameSubscription>,
}

impl Game {
	pub fn new(external_id: &str, provider_id: ProviderId, title: &str) -> Self {
		let title = GameTitle::new(title);
		let mut tags = HashSet::default();
		if title.is_probably_demo() {
			tags.insert(GameTag::Demo);
		}

		Self {
			external_id: external_id.to_string(),
			unique_id: external_id.to_string(),
			provider_id,
			tags,
			installed_game: None,
			remote_game: None,
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

	pub fn get_engine(&self) -> Option<&GameEngine> {
		if let Some(installed_game) = &self.installed_game {
			if let Some(engine) = installed_game.executable.engine.as_ref() {
				return Some(engine);
			}
		}

		if let Some(remote_game) = &self.remote_game {
			if let Some(engine) = remote_game
				.engines
				.as_ref()
				.and_then(|engines| engines.first())
			{
				return Some(engine);
			}
		}

		None
	}
}
