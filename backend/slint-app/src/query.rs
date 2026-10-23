use std::{sync::Arc, thread, time::Duration};

use rai_pal_core::{
	games_query::{GamesQuery, GamesSortBy},
	local_database::{
		app_database::DbMutex,
		game_database::{GameDatabase, GameIdsResponse},
	},
};

use crate::{
	MainWindow,
	filters::{FilterState, build_games_filter},
	model::make_rows,
	scheduler::FetchScheduler,
};

pub fn load_game_ids(database: &DbMutex, query: GamesQuery) -> GameIdsResponse {
	match database.get_game_ids(Some(query)) {
		Ok(response) => response,
		Err(err) => {
			eprintln!("Failed to load game ids: {err}");
			GameIdsResponse {
				game_ids: vec![],
				total_count: 0,
			}
		}
	}
}

pub fn make_games_query(
	search: &str,
	sort_index: i32,
	filter_state: &FilterState,
) -> GamesQuery {
	let (sort_by, sort_descending) = match sort_index {
		1 => (GamesSortBy::Title, true),
		2 => (GamesSortBy::Engine, false),
		3 => (GamesSortBy::Engine, true),
		4 => (GamesSortBy::ReleaseDate, false),
		5 => (GamesSortBy::ReleaseDate, true),
		_ => (GamesSortBy::Title, false),
	};

	GamesQuery {
		search: search.to_string(),
		sort_by,
		sort_descending,
		filter: build_games_filter(filter_state),
		..Default::default()
	}
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

		let response = load_game_ids(&database, query);
		let ui_scheduler = scheduler.clone();

		let _ = slint::invoke_from_event_loop(move || {
			if ui_scheduler.generation() != query_generation {
				return;
			}

			let Some(window) = window.upgrade() else {
				return;
			};

			let rows = make_rows(&response.game_ids);
			ui_scheduler.begin_query();

			let model = std::rc::Rc::new(slint::VecModel::from(rows));
			window.set_game_rows(model.into());
			window.set_total_count(response.total_count as i32);
		});
	});
}

