use rai_pal_core::{
	game_engines::game_engine::EngineBrand,
	game_providers::game_provider::GameProviderId,
	games_query::{FilterItem, GamesFilter, InstallState},
};

pub struct FilterState {
	installed_installed: bool,
	installed_not_installed: bool,
	engine_unity: bool,
	engine_unreal: bool,
	engine_godot: bool,
	engine_gamemaker: bool,
	engine_unknown: bool,
	provider_steam: bool,
	provider_epic: bool,
	provider_gog: bool,
	provider_itch: bool,
	provider_xbox: bool,
	provider_manual: bool,
}

impl Default for FilterState {
	fn default() -> Self {
		Self {
			installed_installed: true,
			installed_not_installed: true,
			engine_unity: true,
			engine_unreal: true,
			engine_godot: true,
			engine_gamemaker: true,
			engine_unknown: true,
			provider_steam: true,
			provider_epic: true,
			provider_gog: true,
			provider_itch: true,
			provider_xbox: true,
			provider_manual: true,
		}
	}
}


impl FilterState {
	pub fn set(&mut self, group: &str, value: &str, checked: bool) {
		match (group, value) {
			("installed", "Installed") => self.installed_installed = checked,
			("installed", "NotInstalled") => self.installed_not_installed = checked,
			("engines", "Unity") => self.engine_unity = checked,
			("engines", "Unreal") => self.engine_unreal = checked,
			("engines", "Godot") => self.engine_godot = checked,
			("engines", "GameMaker") => self.engine_gamemaker = checked,
			("engines", "Unknown") => self.engine_unknown = checked,
			("providers", "Steam") => self.provider_steam = checked,
			("providers", "Epic") => self.provider_epic = checked,
			("providers", "Gog") => self.provider_gog = checked,
			("providers", "Itch") => self.provider_itch = checked,
			("providers", "Xbox") => self.provider_xbox = checked,
			("providers", "Manual") => self.provider_manual = checked,
			_ => {}
		}
	}
}
fn filter_item(enabled: bool) -> FilterItem {
	FilterItem {
		enabled,
		locked: false,
	}
}

pub fn build_games_filter(state: &FilterState) -> GamesFilter {
	let mut filter = GamesFilter::default();

	filter.installed.known.insert(
		InstallState::Installed,
		filter_item(state.installed_installed),
	);
	filter.installed.known.insert(
		InstallState::NotInstalled,
		filter_item(state.installed_not_installed),
	);

	filter.engines.known.insert(
		EngineBrand::Unity,
		filter_item(state.engine_unity),
	);
	filter.engines.known.insert(
		EngineBrand::Unreal,
		filter_item(state.engine_unreal),
	);
	filter.engines.known.insert(
		EngineBrand::Godot,
		filter_item(state.engine_godot),
	);
	filter.engines.known.insert(
		EngineBrand::GameMaker,
		filter_item(state.engine_gamemaker),
	);
	filter.engines.unknown = Some(filter_item(state.engine_unknown));

	filter.providers.known.insert(
		GameProviderId::Steam,
		filter_item(state.provider_steam),
	);
	filter.providers.known.insert(
		GameProviderId::Epic,
		filter_item(state.provider_epic),
	);
	filter.providers.known.insert(
		GameProviderId::Gog,
		filter_item(state.provider_gog),
	);
	filter.providers.known.insert(
		GameProviderId::Itch,
		filter_item(state.provider_itch),
	);
	filter.providers.known.insert(
		GameProviderId::Xbox,
		filter_item(state.provider_xbox),
	);
	filter.providers.known.insert(
		GameProviderId::Manual,
		filter_item(state.provider_manual),
	);

	filter
}

