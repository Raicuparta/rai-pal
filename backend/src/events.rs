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
});

pub trait EventEmitter {
	fn emit_event(&self, event: AppEvent) -> Result;
}

impl EventEmitter for tauri::AppHandle {
	fn emit_event(&self, event: AppEvent) -> Result {
		Ok(self.emit_all(&event.to_string(), ())?)
	}
}
