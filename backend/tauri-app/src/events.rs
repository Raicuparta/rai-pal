use std::fmt::Display;

use rai_pal_proc_macros::serializable_event;
use serde::Serialize;
use tauri_specta::Event;

use rai_pal_core::{
	local_mod, mod_loaders::mod_loader, providers::provider::ProviderId, remote_mod,
};

#[serializable_event]
pub struct FoundInstalledGame();

#[serializable_event]
pub struct FoundGame();

#[serializable_event]
pub struct FoundOwnedGame();

#[serializable_event]
pub struct SyncModLoaders(pub mod_loader::DataMap);

#[serializable_event]
pub struct SyncLocalMods(pub local_mod::Map);

#[serializable_event]
pub struct SyncRemoteMods(pub remote_mod::Map);

#[serializable_event]
pub struct ExecutedProviderCommand;

#[serializable_event]
pub struct SelectInstalledGame(pub ProviderId, pub String);

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
		FoundInstalledGame,
		FoundOwnedGame,
		SyncModLoaders,
		SyncLocalMods,
		SyncRemoteMods,
		ExecutedProviderCommand,
		SelectInstalledGame,
		ErrorRaised,
	]
}
