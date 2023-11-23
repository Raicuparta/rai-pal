use serde::Serialize;
use tauri::Manager;

use crate::{
	result::Result,
	serializable_enum,
};

serializable_enum!(AppEvent {
	SyncInstalledGames,
	SyncOwnedGames,
	SyncDiscoverGames,
	SyncMods,
	ExecutedSteamCommand,
	GameAdded,
});

pub trait EventEmitter {
	fn emit_event<TPayload: Serialize + Clone>(&self, event: AppEvent, payload: TPayload)
		-> Result;
}

impl EventEmitter for tauri::AppHandle {
	fn emit_event<TPayload: Serialize + Clone>(
		&self,
		event: AppEvent,
		payload: TPayload,
	) -> Result {
		Ok(self.emit_all(&event.to_string(), payload)?)
	}
}
