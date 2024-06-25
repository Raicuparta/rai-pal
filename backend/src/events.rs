use std::fmt::Display;

use log::error;
use rai_pal_proc_macros::derive_event;
use serde::{Deserialize, Serialize};
use tauri::{EventTarget, Manager};

use crate::serializable_event;

#[serializable_event]
pub struct SyncInstalledGames;

#[serializable_event]
pub struct SyncOwnedGames;

#[serializable_event]
pub struct SyncRemoteGames;

#[serializable_event]
pub struct SyncModLoaders;

#[serializable_event]
pub struct SyncLocalMods;

#[serializable_event]
pub struct SyncRemoteMods;

#[serializable_event]
pub struct ExecutedProviderCommand;

#[serializable_event]
pub struct GameAdded;

#[serializable_event]
pub struct GameRemoved;

#[serializable_event]
pub struct Error(String);

// pub trait EventEmitter {
// 	fn emit_event<TPayload: Serialize + Clone>(&self, event: AppEvent, payload: TPayload);
// 	fn emit_error<TPayload: Serialize + Clone + Display>(&self, payload: TPayload);
// }

// impl EventEmitter for tauri::AppHandle {
// 	fn emit_event<TPayload: Serialize + Clone>(&self, event: AppEvent, payload: TPayload) {
// 		self.emit_to(EventTarget::any(), &event.to_string(), payload)
// 			.unwrap_or_else(|err| error!("Failed to emit event: {err}"));
// 	}

// 	fn emit_error<TPayload: Serialize + Clone + Display>(&self, payload: TPayload) {
// 		error!("Error: {payload}");
// 		self.emit_to(EventTarget::any(), &AppEvent::Error.to_string(), payload)
// 			.unwrap_or_else(|err| error!("Failed to emit error: {err}."));
// 	}
// }

// impl core::fmt::Display for AppEvent {
// 	fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
// 		write!(f, "{:?}", self)
// 	}
// }

pub fn collect_events() -> (
	tauri_specta::EventCollection,
	std::vec::Vec<tauri_specta::EventDataType>,
	specta::TypeMap,
) {
	tauri_specta::collect_events![
		SyncInstalledGames,
		SyncOwnedGames,
		SyncRemoteGames,
		SyncModLoaders,
		SyncLocalMods,
		SyncRemoteMods,
		ExecutedProviderCommand,
		GameAdded,
		GameRemoved,
		Error,
	]
}
