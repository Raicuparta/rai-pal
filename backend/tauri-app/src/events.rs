use rai_pal_core::{
	game::DbGame,
	game_providers::game_provider::GameProviderId,
	local_database::mod_database::GameModInfo,
};
use rai_pal_proc_macros::{
	serializable_event,
	serializable_struct,
};
use serde::Serialize;

#[serializable_struct]
pub struct SelectedGameData {
	pub game: DbGame,
	pub mod_infos: Vec<GameModInfo>,
}

#[serializable_event]
pub struct RefreshGame(pub GameProviderId, pub String);

#[serializable_event]
pub struct AppDatabaseChanged();

#[serializable_event]
pub struct ExecutedProviderCommand;

#[serializable_event]
pub struct SelectGame(pub Option<SelectedGameData>);

#[serializable_event]
pub struct AddModSource(pub String);

pub trait EventEmitter {
	fn emit_safe<TEvent: tauri_specta::Event + Serialize + Clone>(&self, event: TEvent);
}

impl EventEmitter for tauri::AppHandle {
	fn emit_safe<TEvent: tauri_specta::Event + serde::Serialize + Clone>(&self, event: TEvent) {
		event
			.emit(self)
			.unwrap_or_else(|err| log::error!("Failed to emit event: {err}"));
	}
}

pub fn collect_events() -> tauri_specta::Events {
	tauri_specta::collect_events![
		RefreshGame,
		AppDatabaseChanged,
		ExecutedProviderCommand,
		SelectGame,
		AddModSource,
	]
}
