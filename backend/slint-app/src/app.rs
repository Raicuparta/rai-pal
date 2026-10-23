use std::{
	cell::RefCell,
	rc::Rc,
	sync::{Arc, Mutex},
	thread,
	time::Duration,
};

use rai_pal_core::{
	game::DbGame,
	game_providers::{
		game_provider::GameProviderId,
		provider_command::ProviderCommandAction,
	},
	games_query::GamesQuery,
	local_database::{
		game_database::{GameDatabase, try_create},
		mod_database::ModDatabase,
	},
};
use slint::{ComponentHandle, Model};

use crate::{
	GameRowData, MainWindow,
	actions,
	fetch::spawn_fetchers,
	filters::FilterState,
	model::{self, make_rows},
	mods,
	query::{load_game_ids, make_games_query, start_query},
	scheduler::{enqueue_index, schedule_visible_range},
	selected,
};

type SelectedGameState = Arc<Mutex<Option<(usize, DbGame)>>>;

pub fn run() -> Result<(), slint::PlatformError> {
	let database = match try_create() {
		Ok(database) => database,
		Err(err) => {
			eprintln!("Failed to open local database: {err}");
			return Ok(());
		}
	};

	if let Err(err) = database.setup_mod_tables() {
		eprintln!("Failed to set up mod tables: {err}");
	}

	let response = load_game_ids(&database, GamesQuery::default());
	let rows = make_rows(&response.game_ids);
	let database = Arc::new(database);

	let window = MainWindow::new()?;
	let window_weak = window.as_weak();
	let scheduler = spawn_fetchers(database.clone(), window_weak.clone());
	scheduler.begin_query();

	let filter_state = Rc::new(RefCell::new(FilterState::default()));
	let selected_game: SelectedGameState = Arc::new(Mutex::new(None));

	let ui_window = window_weak.clone();
	let ui_scheduler = scheduler.clone();
	window.on_request_game(move |index| {
		let Some(window) = ui_window.upgrade() else {
			return;
		};

		let rows = window.get_game_rows();
		enqueue_index(&ui_scheduler, &rows, index.max(0) as usize);
	});

	let ui_window = window_weak.clone();
	let ui_scheduler = scheduler.clone();
	window.on_visible_range_changed(move |first, last| {
		if last < 0 {
			return;
		}

		let Some(window) = ui_window.upgrade() else {
			return;
		};

		let rows = window.get_game_rows();
		schedule_visible_range(
			&ui_scheduler,
			&rows,
			first.max(0) as usize,
			last as usize,
		);
	});

	let ui_window = window_weak.clone();
	let ui_database = database.clone();
	let ui_scheduler = scheduler.clone();
	let ui_filter_state = filter_state.clone();
	window.on_search_changed(move |text| {
		let Some(window) = ui_window.upgrade() else {
			return;
		};

		let query = {
			let filters = ui_filter_state.borrow();
			make_games_query(text.as_str(), window.get_sort_index(), &filters)
		};

		start_query(
			ui_window.clone(),
			ui_database.clone(),
			ui_scheduler.clone(),
			query,
			Duration::from_millis(200),
		);
	});

	let ui_window = window_weak.clone();
	let ui_database = database.clone();
	let ui_scheduler = scheduler.clone();
	let ui_filter_state = filter_state.clone();
	window.on_sort_changed(move |index| {
		let Some(window) = ui_window.upgrade() else {
			return;
		};

		let query = {
			let filters = ui_filter_state.borrow();
			make_games_query(window.get_search_text().as_str(), index, &filters)
		};

		start_query(
			ui_window.clone(),
			ui_database.clone(),
			ui_scheduler.clone(),
			query,
			Duration::ZERO,
		);
	});

	let ui_window = window_weak.clone();
	let ui_database = database.clone();
	let ui_scheduler = scheduler.clone();
	let ui_filter_state = filter_state.clone();
	window.on_filter_changed(move |group, value, checked| {
		let Some(window) = ui_window.upgrade() else {
			return;
		};

		let query = {
			let mut filters = ui_filter_state.borrow_mut();
			filters.set(group.as_str(), value.as_str(), checked);

			make_games_query(
				window.get_search_text().as_str(),
				window.get_sort_index(),
				&filters,
			)
		};

		start_query(
			ui_window.clone(),
			ui_database.clone(),
			ui_scheduler.clone(),
			query,
			Duration::ZERO,
		);
	});

	let ui_window = window_weak.clone();
	let ui_database = database.clone();
	let ui_selected_game = selected_game.clone();
	window.on_select_game(move |index| {
		let index = index.max(0) as usize;

		let Some(window) = ui_window.upgrade() else {
			return;
		};

		let rows = window.get_game_rows();
		let Some(row) = rows.row_data(index) else {
			return;
		};

		let Some(game) = game_from_row(&ui_database, &row) else {
			return;
		};

		if let Ok(mut selected_game) = ui_selected_game.lock() {
			*selected_game = Some((index, game.clone()));
		}

		mods::refresh_mods(&window, &ui_database, &game);
		selected::show(&window, &game, row);
	});

	let ui_window = window_weak.clone();
	let ui_selected_game = selected_game.clone();
	window.on_back(move || {
		if let Ok(mut selected_game) = ui_selected_game.lock() {
			*selected_game = None;
		}

		if let Some(window) = ui_window.upgrade() {
			selected::hide(&window);
		}
	});

	let ui_database = database.clone();
	let ui_selected_game = selected_game.clone();
	window.on_run_game(move || {
		let Some((_, game)) = get_selected_game(&ui_selected_game) else {
			return;
		};

		if let Some(action) = actions::primary_start(&game) {
			actions::run_provider_action(ui_database.clone(), game, action);
		}
	});

	let ui_database = database.clone();
	let ui_selected_game = selected_game.clone();
	window.on_run_provider_action(move |value| {
		let Ok(action) = value.parse::<ProviderCommandAction>() else {
			return;
		};

		let Some((_, game)) = get_selected_game(&ui_selected_game) else {
			return;
		};

		actions::run_provider_action(ui_database.clone(), game, action);
	});

	let ui_selected_game = selected_game.clone();
	window.on_open_game_folder(move || {
		if let Some((_, game)) = get_selected_game(&ui_selected_game) {
			actions::open_game_folder(game);
		}
	});

	let ui_selected_game = selected_game.clone();
	window.on_open_game_mods_folder(move || {
		if let Some((_, game)) = get_selected_game(&ui_selected_game) {
			actions::open_game_mods_folder(game);
		}
	});

	let ui_selected_game = selected_game.clone();
	window.on_open_game_data_folder(move || {
		if let Some((_, game)) = get_selected_game(&ui_selected_game) {
			actions::open_game_data_folder(game);
		}
	});

	let ui_selected_game = selected_game.clone();
	window.on_open_wine_prefix_folder(move || {
		if let Some((_, game)) = get_selected_game(&ui_selected_game) {
			actions::open_wine_prefix_folder(game);
		}
	});

	let ui_selected_game = selected_game.clone();
	window.on_open_wine_binary_folder(move || {
		if let Some((_, game)) = get_selected_game(&ui_selected_game) {
			actions::open_wine_binary_folder(game);
		}
	});

	let ui_window = window_weak.clone();
	let ui_database = database.clone();
	let ui_selected_game = selected_game.clone();
	window.on_install_mod(move |mod_id| {
		let Some((_, game)) = get_selected_game(&ui_selected_game) else {
			return;
		};

		mods::install_mod(
			ui_database.clone(),
			ui_window.clone(),
			game,
			mod_id.to_string(),
		);
	});

	let ui_window = window_weak.clone();
	let ui_database = database.clone();
	let ui_selected_game = selected_game.clone();
	window.on_update_mod(move |mod_id| {
		let Some((_, game)) = get_selected_game(&ui_selected_game) else {
			return;
		};

		mods::install_mod(
			ui_database.clone(),
			ui_window.clone(),
			game,
			mod_id.to_string(),
		);
	});

	let ui_window = window_weak.clone();
	let ui_database = database.clone();
	let ui_selected_game = selected_game.clone();
	window.on_uninstall_mod(move |mod_id| {
		let Some((_, game)) = get_selected_game(&ui_selected_game) else {
			return;
		};

		mods::uninstall_mod(
			ui_database.clone(),
			ui_window.clone(),
			game,
			mod_id.to_string(),
		);
	});

	let ui_database = database.clone();
	let ui_selected_game = selected_game.clone();
	window.on_run_mod(move |mod_id| {
		let Some((_, game)) = get_selected_game(&ui_selected_game) else {
			return;
		};

		mods::run_mod(ui_database.clone(), game, mod_id.to_string());
	});

	let ui_database = database.clone();
	let ui_selected_game = selected_game.clone();
	window.on_open_mod_folder(move |mod_id| {
		let Some((_, game)) = get_selected_game(&ui_selected_game) else {
			return;
		};

		mods::open_mod_folder(ui_database.clone(), game, mod_id.to_string());
	});

	let ui_database = database.clone();
	let ui_selected_game = selected_game.clone();
	window.on_configure_mod(move |mod_id| {
		let Some((_, game)) = get_selected_game(&ui_selected_game) else {
			return;
		};

		mods::configure_mod(ui_database.clone(), game, mod_id.to_string(), false);
	});

	let ui_database = database.clone();
	let ui_selected_game = selected_game.clone();
	window.on_configure_mod_folder(move |mod_id| {
		let Some((_, game)) = get_selected_game(&ui_selected_game) else {
			return;
		};

		mods::configure_mod(ui_database.clone(), game, mod_id.to_string(), true);
	});

	let ui_window = window_weak.clone();
	let ui_database = database.clone();
	let ui_selected_game = selected_game.clone();
	window.on_uninstall_all_mods(move || {
		let Some((_, game)) = get_selected_game(&ui_selected_game) else {
			return;
		};

		mods::uninstall_all_mods(ui_database.clone(), ui_window.clone(), game);
	});

	let ui_window = window_weak.clone();
	let ui_database = database.clone();
	let ui_selected_game = selected_game.clone();
	window.on_refresh_game(move || {
		let Some((index, game)) = get_selected_game(&ui_selected_game) else {
			return;
		};

		let database = ui_database.clone();
		let window = ui_window.clone();
		let ui_selected_game = ui_selected_game.clone();

		thread::spawn(move || {
			let mut game = game;

			if let Err(err) = game.refresh_executable() {
				eprintln!("Failed to refresh game: {err}");
				return;
			}

			database.insert_game(&game);

			if let Err(err) = database.refresh_installed_mods() {
				eprintln!("Failed to refresh installed mods: {err}");
			}

			let Ok(game) = database.get_game(&game.provider_id, &game.game_id) else {
				return;
			};

			let _ = slint::invoke_from_event_loop(move || {
				let Some(window) = window.upgrade() else {
					return;
				};

				if let Ok(mut selected_game) = ui_selected_game.lock() {
					*selected_game = Some((index, game.clone()));
				}

				let rows = window.get_game_rows();
				let mut row = rows.row_data(index).unwrap_or_default();
				model::apply_game_to_row(&mut row, &game);
				rows.set_row_data(index, row.clone());
				mods::refresh_mods(&window, &database, &game);
				selected::show(&window, &game, row);
			});
		});
	});

	let model = Rc::new(slint::VecModel::from(rows));
	window.set_game_rows(model.into());
	window.set_total_count(response.total_count as i32);
	window.run()
}

fn game_from_row(database: &Arc<rai_pal_core::local_database::app_database::DbMutex>, row: &GameRowData) -> Option<DbGame> {
	if let Ok(game) = serde_json::from_str::<DbGame>(row.game_json.as_str()) {
		return Some(game);
	}

	let provider_id = row.provider_id.parse::<GameProviderId>().ok()?;
	database.get_game(&provider_id, row.game_id.as_str()).ok()
}

fn get_selected_game(state: &SelectedGameState) -> Option<(usize, DbGame)> {
	state.lock().ok().and_then(|selected_game| selected_game.clone())
}
