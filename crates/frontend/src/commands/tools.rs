use std::{path::PathBuf, sync::Arc, thread, time::Duration};

use rai_pal_backend::{
	app_paths,
	game_providers::{
		game_provider::GameProviderId,
		steam::{steam_provider::Steam, steam_shortcut},
	},
	local_database::mod_database::ModDatabase,
	result::Result,
};
use slint::ComponentHandle;

use crate::{MessageContext, notifications};

use super::{CommandHandler, on_command};

impl CommandHandler {
	pub fn reset_steam_cache(&self) {
		let Some(ui_window) = self.ui() else {
			return;
		};

		if ui_window.global::<crate::SteamCacheState>().get_state() == crate::ActionState::Loading {
			return;
		}

		let window = self.window();
		let database = Arc::clone(&self.database);

		if let Some(window) = window.upgrade() {
			window
				.global::<crate::SteamCacheState>()
				.set_state(crate::ActionState::Loading);
			window
				.global::<crate::SteamCacheState>()
				.set_error(Default::default());
		}

		thread::spawn(move || {
			let result = Steam::delete_cache()
				.and_then(|()| GameProviderId::Steam.insert_games(&database))
				.and_then(|()| database.refresh_installed_mods());

			crate::runtime::invoke_on_ui(move || {
				let Some(window) = window.upgrade() else {
					return;
				};

				match result {
					Ok(()) => {
						window
							.global::<crate::SteamCacheState>()
							.set_state(crate::ActionState::Success);

						if let Some(handler) = Self::current() {
							handler.start_query(Duration::ZERO);
							handler.reload_selected_game();
						}
					}
					Err(err) => {
						window
							.global::<crate::SteamCacheState>()
							.set_error(notifications::data(MessageContext::ResetSteamCache, err));
						window
							.global::<crate::SteamCacheState>()
							.set_state(crate::ActionState::Error);
					}
				}
			});
		});
	}

	pub fn add_steam_shortcut(&self) {
		let Some(ui_window) = self.ui() else {
			return;
		};

		if ui_window.global::<crate::SteamShortcutState>().get_state()
			== crate::ActionState::Loading
		{
			return;
		}

		let window = self.window();

		if let Some(window) = window.upgrade() {
			window
				.global::<crate::SteamShortcutState>()
				.set_state(crate::ActionState::Loading);
			window
				.global::<crate::SteamShortcutState>()
				.set_error(Default::default());
		}

		thread::spawn(move || {
			let result = add_shortcut();

			crate::runtime::invoke_on_ui(move || {
				let Some(window) = window.upgrade() else {
					return;
				};

				match result {
					Ok(()) => window
						.global::<crate::SteamShortcutState>()
						.set_state(crate::ActionState::Success),
					Err(err) => {
						window
							.global::<crate::SteamShortcutState>()
							.set_error(notifications::data(MessageContext::AddSteamShortcut, err));
						window
							.global::<crate::SteamShortcutState>()
							.set_state(crate::ActionState::Error);
					}
				}
			});
		});
	}
}

const STEAM_ARTWORK: &[(&str, &[u8])] = &[
	(
		"grid.png",
		include_bytes!("../../resources/steam-artwork/grid.png"),
	),
	(
		"header.png",
		include_bytes!("../../resources/steam-artwork/header.png"),
	),
	(
		"hero.png",
		include_bytes!("../../resources/steam-artwork/hero.png"),
	),
	(
		"icon.png",
		include_bytes!("../../resources/steam-artwork/icon.png"),
	),
	(
		"logo.png",
		include_bytes!("../../resources/steam-artwork/logo.png"),
	),
	(
		"portrait.png",
		include_bytes!("../../resources/steam-artwork/portrait.png"),
	),
];

fn add_shortcut() -> Result {
	let executable = std::env::current_exe()?;
	let artwork_dir = write_steam_artwork()?;

	steam_shortcut::add_current_executable_to_steam_shortcuts(&executable, &artwork_dir)
}

fn write_steam_artwork() -> Result<PathBuf> {
	let artwork_dir = app_paths::temp_dir("steam-artwork")?;

	for (file_name, bytes) in STEAM_ARTWORK {
		fs_err::write(artwork_dir.join(file_name), bytes)?;
	}

	Ok(artwork_dir)
}

pub fn register(window: &crate::MainWindow) {
	on_command!(window, DialogActions, on_reset_steam_cache, || handler => {
		handler.reset_steam_cache();
	});
	on_command!(window, DialogActions, on_add_steam_shortcut, || handler => {
		handler.add_steam_shortcut();
	});
}
