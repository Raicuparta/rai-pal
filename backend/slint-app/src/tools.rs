use std::{path::PathBuf, sync::Arc, thread, time::Duration};

use rai_pal_core::{
	app_paths,
	game_providers::{
		game_provider::GameProviderId,
		steam::{steam_provider::Steam, steam_shortcut},
	},
	games_query::GamesQuery,
	local_database::{app_database::DbMutex, mod_database::ModDatabase},
	result::Result,
};
use slint::ComponentHandle;

use crate::{
	MainWindow, query::start_query, refresh, scheduler::FetchScheduler, selected::SelectedGameState,
};

pub fn reset_steam_cache(
	window: slint::Weak<MainWindow>,
	database: Arc<DbMutex>,
	scheduler: Arc<FetchScheduler>,
	query: GamesQuery,
	selected_game: SelectedGameState,
) {
	if let Some(window) = window.upgrade() {
		window.set_steam_cache_running(true);
		window.set_steam_cache_success(false);
		window.set_steam_cache_error("".into());
	}

	thread::spawn(move || {
		let result = Steam::delete_cache()
			.and_then(|()| GameProviderId::Steam.insert_games(&database))
			.and_then(|()| database.refresh_installed_mods());

		let _ = slint::invoke_from_event_loop(move || {
			let Some(window) = window.upgrade() else {
				return;
			};

			window.set_steam_cache_running(false);

			match result {
				Ok(()) => {
					window.set_steam_cache_success(true);
					start_query(
						window.as_weak(),
						database.clone(),
						scheduler,
						query,
						Duration::ZERO,
					);
					refresh::refresh_selected_game(&window, &database, &selected_game);
				}
				Err(err) => {
					window.set_steam_cache_error(err.to_string().into());
				}
			}
		});
	});
}

pub fn add_steam_shortcut(window: slint::Weak<MainWindow>) {
	if let Some(window) = window.upgrade() {
		window.set_steam_shortcut_running(true);
		window.set_steam_shortcut_success(false);
		window.set_steam_shortcut_error("".into());
	}

	thread::spawn(move || {
		let result = add_shortcut();

		let _ = slint::invoke_from_event_loop(move || {
			let Some(window) = window.upgrade() else {
				return;
			};

			window.set_steam_shortcut_running(false);

			match result {
				Ok(()) => window.set_steam_shortcut_success(true),
				Err(err) => window.set_steam_shortcut_error(err.to_string().into()),
			}
		});
	});
}

const STEAM_ARTWORK: &[(&str, &[u8])] = &[
	(
		"grid.png",
		include_bytes!("../../tauri-app/resources/steam-artwork/grid.png"),
	),
	(
		"header.png",
		include_bytes!("../../tauri-app/resources/steam-artwork/header.png"),
	),
	(
		"hero.png",
		include_bytes!("../../tauri-app/resources/steam-artwork/hero.png"),
	),
	(
		"icon.png",
		include_bytes!("../../tauri-app/resources/steam-artwork/icon.png"),
	),
	(
		"logo.png",
		include_bytes!("../../tauri-app/resources/steam-artwork/logo.png"),
	),
	(
		"portrait.png",
		include_bytes!("../../tauri-app/resources/steam-artwork/portrait.png"),
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
		std::fs::write(artwork_dir.join(file_name), bytes)?;
	}

	Ok(artwork_dir)
}
