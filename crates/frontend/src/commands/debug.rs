use rai_pal_backend::local_database::mod_database::ModDatabase;
use serde::Serialize;
use slint::ComponentHandle as _;
use std::thread;

use crate::{DebugDataState, MainWindow, MessageContext, notifications};

use super::{CommandHandler, on_command};

impl CommandHandler {
	pub fn show_game_debug_data(&self) {
		let Some(game) = self.selected_game() else {
			return;
		};

		self.show_debug_data(&game);
	}

	pub fn show_mod_debug_data(&self, mod_id: &str) {
		match self.database.get_mod(mod_id) {
			Ok(game_mod) => self.show_debug_data(&game_mod),
			Err(err) => {
				let Some(window) = self.ui() else {
					return;
				};
				notifications::error(
					&window,
					notifications::data(MessageContext::DebugData, &err),
				);
			}
		}
	}

	fn show_debug_data(&self, value: &impl Serialize) {
		let Some(window) = self.ui() else {
			return;
		};

		match serde_json::to_string_pretty(value) {
			Ok(json) => {
				let state = window.global::<DebugDataState>();
				state.set_text(json.as_str().into());
				state.set_reset_scroll_revision(state.get_reset_scroll_revision() + 1);
				state.set_open(true);
			}
			Err(err) => notifications::error(
				&window,
				notifications::data(MessageContext::DebugData, &err),
			),
		}
	}

	pub fn show_logs(&self) {
		let window = self.window();

		thread::spawn(move || {
			let result = crate::logging::latest_log_file().and_then(|path| {
				let text = fs_err::read_to_string(&path)?;
				Ok((path, text))
			});

			crate::runtime::with_ui_window(&window, move |window| {
				let Some(handler) = Self::current() else {
					return;
				};

				match result {
					Ok((path, text)) => {
						let title = path
							.file_name()
							.map_or_default(|name| name.to_string_lossy().into_owned());

						handler.view_text_file(path, text, title);
					}
					Err(err) => notifications::error(
						&window,
						notifications::data(MessageContext::None, &err),
					),
				}
			});
		});
	}
}

pub fn register(window: &MainWindow) {
	on_command!(window, GamesActions, on_request_debug_data, || handler => {
		handler.show_game_debug_data();
	});
	on_command!(window, AppActions, on_show_logs, || handler => {
		handler.show_logs();
	});
}
