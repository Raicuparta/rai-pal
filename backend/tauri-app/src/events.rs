use std::{
	collections::HashMap,
	fmt::Display,
};

use rai_pal_core::{
	game_mods::game_mod::GameMod,
	providers::provider::ProviderId,
};
use rai_pal_proc_macros::serializable_event;
use serde::Serialize;
use tauri_specta::Event;

#[serializable_event]
pub struct RefreshGame(pub ProviderId, pub String);

#[serializable_event]
pub struct GamesChanged();

#[serializable_event]
pub struct SyncMods(pub HashMap<String, GameMod>);

#[serializable_event]
pub struct ExecutedProviderCommand;

#[serializable_event]
pub struct SelectGame(pub ProviderId, pub String);

#[serializable_event]
pub struct ErrorRaised(pub String);

pub trait EventEmitter {
	fn emit_safe<TEvent: tauri_specta::Event + Serialize + Clone>(&self, event: TEvent);
	fn emit_error<TError: Serialize + Clone + Display>(&self, error: TError);
}

impl EventEmitter for tauri::AppHandle {
	fn emit_safe<TEvent: tauri_specta::Event + serde::Serialize + Clone>(&self, event: TEvent) {
		event
			.emit(self)
			.unwrap_or_else(|err| log::error!("Failed to emit event: {err}"));
	}

	fn emit_error<TError: Serialize + Clone + Display>(&self, error: TError) {
		log::error!("Error: {error}");

		ErrorRaised(error.to_string())
			.emit(self)
			.unwrap_or_else(|err| log::error!("Failed to emit error event: {err}"));
	}
}

pub fn collect_events() -> tauri_specta::Events {
	tauri_specta::collect_events![
		RefreshGame,
		GamesChanged,
		SyncMods,
		ExecutedProviderCommand,
		SelectGame,
		ErrorRaised,
	]
}
