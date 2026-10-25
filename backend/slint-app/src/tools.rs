use std::{sync::Arc, thread, time::Duration};

use rai_pal_core::{
	game_providers::{game_provider::GameProviderId, steam::steam_provider::Steam},
	games_query::GamesQuery,
	local_database::{app_database::DbMutex, mod_database::ModDatabase},
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
