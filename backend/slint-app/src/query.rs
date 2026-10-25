use std::{sync::Arc, thread, time::Duration};

use rai_pal_core::{
	games_query::GamesQuery,
	local_database::{
		app_database::DbMutex,
		game_database::{GameDatabase, GameIdsResponse},
	},
};

use crate::{MainWindow, model::make_rows, notifications, scheduler::FetchScheduler};

pub fn load_game_ids(database: &DbMutex, query: GamesQuery) -> Result<GameIdsResponse, String> {
	database
		.get_game_ids(Some(query))
		.map_err(|err| err.to_string())
}

pub fn start_query(
	window: slint::Weak<MainWindow>,
	database: Arc<DbMutex>,
	scheduler: Arc<FetchScheduler>,
	query: GamesQuery,
	delay: Duration,
) {
	let query_generation = scheduler.begin_query();

	thread::spawn(move || {
		if !delay.is_zero() {
			thread::sleep(delay);
		}

		if scheduler.generation() != query_generation {
			return;
		}

		let result = load_game_ids(&database, query);
		let ui_scheduler = scheduler.clone();

		let _ = slint::invoke_from_event_loop(move || {
			if ui_scheduler.generation() != query_generation {
				return;
			}

			let Some(window) = window.upgrade() else {
				return;
			};

			let response = match result {
				Ok(response) => response,
				Err(err) => {
					eprintln!("Failed to get app data: {err}");
					notifications::error(&window, format!("Failed to get app data: {err}"));
					return;
				}
			};

			let rows = make_rows(&response.game_ids);
			ui_scheduler.begin_query();

			window.set_games_total_count(i32::try_from(response.total_count).unwrap_or(i32::MAX));
			let model = std::rc::Rc::new(slint::VecModel::from(rows));
			window.set_game_rows(model.into());
		});
	});
}
