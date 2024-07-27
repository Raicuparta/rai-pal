use std::fmt::Display;

use rai_pal_proc_macros::serializable_event;
use serde::Serialize;
use tauri_specta::Event;

use rai_pal_core::{
	installed_game, local_mod, mod_loaders::mod_loader, owned_game, remote_game, remote_mod,
};

#[serializable_event]
pub struct FoundInstalledGame(pub installed_game::InstalledGame);

#[serializable_event]
pub struct FoundOwnedGame(pub owned_game::OwnedGame);

#[serializable_event]
pub struct FoundRemoteGame(pub remote_game::RemoteGame);

#[serializable_event]
pub struct SyncModLoaders(pub mod_loader::DataMap);

#[serializable_event]
pub struct SyncLocalMods(pub local_mod::Map);

#[serializable_event]
pub struct SyncRemoteMods(pub remote_mod::Map);

#[serializable_event]
pub struct ExecutedProviderCommand;

#[serializable_event]
pub struct GameAdded(pub String);

#[serializable_event]
pub struct GameRemoved(pub String);

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

pub fn collect_events() -> (
	tauri_specta::EventCollection,
	std::vec::Vec<tauri_specta::EventDataType>,
	specta::TypeMap,
) {
	tauri_specta::collect_events![
		FoundInstalledGame,
		FoundOwnedGame,
		FoundRemoteGame,
		SyncModLoaders,
		SyncLocalMods,
		SyncRemoteMods,
		ExecutedProviderCommand,
		GameAdded,
		GameRemoved,
		ErrorRaised,
	]
}
