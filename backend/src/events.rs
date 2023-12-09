use std::fmt::Display;

use serde::Serialize;
use tauri::Manager;

use crate::serializable_enum;

serializable_enum!(AppEvent {
	SyncInstalledGames,
	SyncOwnedGames,
	SyncMods,
	ExecutedSteamCommand,
	GameAdded,
	GameRemoved,
	Error,
});

pub trait EventEmitter {
	fn emit_event<TPayload: Serialize + Clone>(&self, event: AppEvent, payload: TPayload);
	fn emit_error<TPayload: Serialize + Clone + Display>(&self, payload: TPayload);
}

impl EventEmitter for tauri::AppHandle {
	fn emit_event<TPayload: Serialize + Clone>(&self, event: AppEvent, payload: TPayload) {
		self.emit_all(&event.to_string(), payload)
			.unwrap_or_else(|err| eprintln!("Failed to emit event: {err}"));
	}

	fn emit_error<TPayload: Serialize + Clone + Display>(&self, payload: TPayload) {
		eprintln!("Error: {payload}");
		self.emit_all(&AppEvent::Error.to_string(), payload)
			.unwrap_or_else(|err| eprintln!("Failed to emit error: {err}."));
	}
}
