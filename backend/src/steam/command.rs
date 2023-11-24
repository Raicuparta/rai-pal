use crate::{
	events::{
		AppEvent,
		EventEmitter,
	},
	result::Result,
};

pub fn run(command: &str, handle: &tauri::AppHandle) -> Result {
	open::that_detached(format!("steam://{command}"))?;
	handle.emit_event(AppEvent::ExecutedSteamCommand, ());
	Ok(())
}
