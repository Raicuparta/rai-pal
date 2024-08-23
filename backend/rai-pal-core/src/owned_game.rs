use std::collections::{HashMap, HashSet};

use rai_pal_proc_macros::serializable_struct;

use crate::{
	game_tag::GameTag,
	game_title::GameTitle,
	providers::{
		provider::ProviderId,
		provider_command::{ProviderCommand, ProviderCommandAction},
	},
};

#[serializable_struct]
pub struct OwnedGame {
	pub global_id: String,
	pub provider_game_id: String,
	pub provider: ProviderId,
	pub title: GameTitle,
	pub release_date: Option<i64>,
	pub thumbnail_url: Option<String>,
	pub tags: HashSet<GameTag>,
	pub provider_commands: HashMap<ProviderCommandAction, ProviderCommand>,
}

impl OwnedGame {
	pub fn new(provider_game_id: &str, provider: ProviderId, name: &str) -> Self {
		let title = GameTitle::new(name);
		let mut tags = HashSet::default();
		if title.is_probably_demo() {
			tags.insert(GameTag::Demo);
		}
		Self {
			global_id: get_global_id(provider, provider_game_id),
			provider_game_id: provider_game_id.to_string(),
			provider,
			tags,
			title,
			provider_commands: HashMap::default(),
			release_date: None,
			thumbnail_url: None,
		}
	}

	pub fn set_release_date(&mut self, release_date: i64) -> &mut Self {
		self.release_date = Some(release_date);
		self
	}

	pub fn set_thumbnail_url(&mut self, thumbnail_url: &str) -> &mut Self {
		self.thumbnail_url = Some(thumbnail_url.to_string());
		self
	}

	pub fn add_tag(&mut self, tag: GameTag) -> &mut Self {
		self.tags.insert(tag);
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

pub fn get_global_id(provider: ProviderId, provider_game_id: &str) -> String {
	format!("{provider}_{provider_game_id}")
}
