use std::collections::BTreeMap;

use rai_pal_core::{
	game_providers::game_provider::GameProviderId,
	mods::game_mod::GameMod,
};
use rai_pal_proc_macros::serializable_event;
use serde::Serialize;

#[serializable_event]
pub struct RefreshGame(pub GameProviderId, pub String);

#[serializable_event]
pub struct GamesChanged();

#[serializable_event]
pub struct SyncMods(pub BTreeMap<String, GameMod>);

#[serializable_event]
pub struct ExecutedProviderCommand;

#[serializable_event]
pub struct SelectGame(pub GameProviderId, pub String);

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
		GamesChanged,
		SyncMods,
		ExecutedProviderCommand,
		SelectGame,
	]
}
