use std::{path::PathBuf, sync::Arc, thread, time::Duration};

use rai_pal_backend::{
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
		window
			.global::<crate::SteamCacheState>()
			.set_state(crate::ActionState::Loading);
		window
			.global::<crate::SteamCacheState>()
			.set_error("".into());
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
					start_query(
						window.as_weak(),
						Arc::clone(&database),
						scheduler,
						query,
						Duration::ZERO,
					);
					refresh::refresh_selected_game(&window, &database, &selected_game);
				}
				Err(err) => {
					window
						.global::<crate::SteamCacheState>()
						.set_error(err.to_string().into());
					window
						.global::<crate::SteamCacheState>()
						.set_state(crate::ActionState::Error);
				}
			}
		});
	});
}

pub fn add_steam_shortcut(window: slint::Weak<MainWindow>) {
	if let Some(window) = window.upgrade() {
		window
			.global::<crate::SteamShortcutState>()
			.set_state(crate::ActionState::Loading);
		window
			.global::<crate::SteamShortcutState>()
			.set_error("".into());
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
						.set_error(err.to_string().into());
					window
						.global::<crate::SteamShortcutState>()
						.set_state(crate::ActionState::Error);
				}
			}
		});
	});
}

const STEAM_ARTWORK: &[(&str, &[u8])] = &[
	(
		"grid.png",
		include_bytes!("../resources/steam-artwork/grid.png"),
	),
	(
		"header.png",
		include_bytes!("../resources/steam-artwork/header.png"),
	),
	(
		"hero.png",
		include_bytes!("../resources/steam-artwork/hero.png"),
	),
	(
		"icon.png",
		include_bytes!("../resources/steam-artwork/icon.png"),
	),
	(
		"logo.png",
		include_bytes!("../resources/steam-artwork/logo.png"),
	),
	(
		"portrait.png",
		include_bytes!("../resources/steam-artwork/portrait.png"),
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
