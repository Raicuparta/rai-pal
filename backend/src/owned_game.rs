use std::collections::{
	HashMap,
	HashSet,
};

use crate::{
	game_executable::OperatingSystem,
	game_mode::GameMode,
	providers::{
		provider::ProviderId,
		provider_command::{
			ProviderCommand,
			ProviderCommandAction,
		},
	},
	serializable_struct,
};

serializable_struct!(OwnedGame {
	pub id: String,
	pub provider: ProviderId,
	pub name: String,
	pub os_list: HashSet<OperatingSystem>,
	pub release_date: Option<i64>,
	pub thumbnail_url: Option<String>,
	pub game_mode: Option<GameMode>,

	// TODO: the keys for this map should be ProviderCommandAction, but tauri-specta doesn't support that.
	pub provider_commands: HashMap<String, ProviderCommand>,
});

impl OwnedGame {
	pub fn new(provider_game_id: &str, provider: ProviderId, name: &str) -> Self {
		Self {
			id: get_id(provider, provider_game_id),
			provider,
			name: name.to_string(),
			os_list: HashSet::default(),
			provider_commands: HashMap::default(),
			release_date: None,
			thumbnail_url: None,
			game_mode: None,
		}
	}

	pub fn set_os_list(&mut self, os_list: HashSet<OperatingSystem>) -> &mut Self {
		self.os_list = os_list;
		self
	}

	pub fn set_release_date(&mut self, release_date: i64) -> &mut Self {
		self.release_date = Some(release_date);
		self
	}

	pub fn set_thumbnail_url(&mut self, thumbnail_url: &str) -> &mut Self {
		self.thumbnail_url = Some(thumbnail_url.to_string());
		self
	}

	pub fn set_game_mode(&mut self, game_mode: GameMode) -> &mut Self {
		self.game_mode = Some(game_mode);
		self
	}

	pub fn add_provider_command(
		&mut self,
		command_action: ProviderCommandAction,
		command: ProviderCommand,
	) -> &mut Self {
		self.provider_commands
			.insert(command_action.to_string(), command);
		self
	}
}

pub fn get_id(provider: ProviderId, provider_game_id: &str) -> String {
	format!("{provider}_{provider_game_id}")
}

pub type Map = HashMap<String, OwnedGame>;
