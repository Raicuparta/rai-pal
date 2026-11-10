use rai_pal_backend::local_database::mod_database::ModDatabase;
use slint::ComponentHandle as _;

use crate::{MainWindow, notifications};

use super::{CommandHandler, on_command};

impl CommandHandler {
	pub fn show_game_debug_data(&self) {
		let Some(window) = self.ui() else {
			return;
		};
		let Some((_, game)) = self.selected_game() else {
			return;
		};

		match serde_json::to_string_pretty(&game) {
			Ok(json) => window
				.global::<crate::DebugState>()
				.set_details_json(json.into()),
			Err(err) => notifications::error(&window, format!("Failed to serialize game: {err}")),
		}
	}

	pub fn show_mod_debug_data(&self, mod_id: &str) {
		let Some(window) = self.ui() else {
			return;
		};

		match self.database.get_mod(mod_id) {
			Ok(game_mod) => match serde_json::to_string_pretty(&game_mod) {
				Ok(json) => window
					.global::<crate::DebugState>()
					.set_details_json(json.into()),
				Err(err) => {
					notifications::error(&window, format!("Failed to serialize mod: {err}"));
				}
			},
			Err(err) => notifications::error(&window, format!("Failed to load mod: {err}")),
		}
	}
}

pub fn register(window: &MainWindow) {
	on_command!(window, GamesActions, on_request_debug_data, || handler => {
		handler.show_game_debug_data();
	});
	on_command!(window, GamesActions, on_request_mod_debug_data, |mod_id| handler => {
		handler.show_mod_debug_data(mod_id.as_str());
	});
}
