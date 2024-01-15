use std::collections::{
	HashMap,
	HashSet,
};

use crate::{
	game_engines::game_engine::GameEngine,
	game_executable::OperatingSystem,
	game_mode::GameMode,
	providers::{
		provider::ProviderId,
		provider_command::ProviderCommand,
	},
	serializable_struct,
	steam::id_lists::UevrScore,
};

serializable_struct!(OwnedGame {
	pub id: String,
	pub provider: ProviderId,
	pub provider_game_id: String,
	pub name: String,
	pub os_list: HashSet<OperatingSystem>,
	pub engine: Option<GameEngine>,
	pub release_date: i64,
	pub thumbnail_url: String,
	pub game_mode: Option<GameMode>,
	pub uevr_score: Option<UevrScore>,
	pub show_library_command: Option<ProviderCommand>,
	pub open_page_command: Option<ProviderCommand>,
	pub install_command: Option<ProviderCommand>,
});

impl OwnedGame {
	pub fn new(provider_game_id: &str, provider: ProviderId, name: &str) -> Self {
		Self {
			id: format!("{provider}_{provider_game_id}"),
			provider_game_id: provider_game_id.to_string(),
			provider,
			name: name.to_string(),
			os_list: HashSet::default(),
			engine: None,
			release_date: 0,
			thumbnail_url: String::default(),
			game_mode: None,
			uevr_score: None,
			show_library_command: None,
			open_page_command: None,
			install_command: None,
		}
	}

	pub fn set_os_list(&mut self, os_list: HashSet<OperatingSystem>) -> &mut Self {
		self.os_list = os_list;
		self
	}

	pub fn set_engine(&mut self, engine: GameEngine) -> &mut Self {
		self.engine = Some(engine);
		self
	}

	pub fn set_release_date(&mut self, release_date: i64) -> &mut Self {
		self.release_date = release_date;
		self
	}

	pub fn set_thumbnail_url(&mut self, thumbnail_url: &str) -> &mut Self {
		self.thumbnail_url = thumbnail_url.to_string();
		self
	}

	pub fn set_game_mode(&mut self, game_mode: GameMode) -> &mut Self {
		self.game_mode = Some(game_mode);
		self
	}

	pub fn set_uevr_score(&mut self, uevr_score: UevrScore) -> &mut Self {
		self.uevr_score = Some(uevr_score);
		self
	}

	pub fn set_show_library_command(&mut self, show_library_command: ProviderCommand) -> &mut Self {
		self.show_library_command = Some(show_library_command);
		self
	}

	pub fn set_open_page_command(&mut self, open_page_command: ProviderCommand) -> &mut Self {
		self.open_page_command = Some(open_page_command);
		self
	}

	pub fn set_install_command(&mut self, install_command: ProviderCommand) -> &mut Self {
		self.install_command = Some(install_command);
		self
	}
}

pub type Map = HashMap<String, OwnedGame>;
