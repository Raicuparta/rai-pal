use std::{
	cell::RefCell,
	rc::Rc,
	sync::{Arc, Mutex},
	thread,
	time::Duration,
};

use rai_pal_core::{
	app_settings::{AppSettings, ColorScheme, TabId},
	game::DbGame,
	game_providers::{game_provider::GameProviderId, provider_command::ProviderCommandAction},
	games_query::GamesSortBy,
	local_database::{
		app_database::DbMutex,
		game_database::{GameDatabase, try_create},
		mod_database::ModDatabase,
	},
};
use slint::{ComponentHandle, Model};

use crate::{
	GameRowData, MainWindow, actions,
	fetch::spawn_fetchers,
	filters::{
		FilterAction, apply_filter_action, build_groups, load_mod_families, reset_all_filters,
		reset_filter_group,
	},
	i18n,
	model::{self, make_rows},
	mods,
	query::{load_game_ids, start_query},
	scheduler::{FetchScheduler, enqueue_index, schedule_visible_range},
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

	let settings = Rc::new(RefCell::new(AppSettings::read()));
	let detected_locale = i18n::detect_locale();
	let initial_locale = i18n::effective_locale(&settings.borrow(), detected_locale);
	let mod_families = load_mod_families(&database);

	let initial_query = settings.borrow().games_query.clone();
	let initial_tab = tab_index(settings.borrow().selected_tab);
	let initial_groups = build_groups(
		&initial_query.filter,
		&mod_families,
		&settings.borrow().expanded_filters,
	);

	let initial_have_changes = initial_groups.iter().any(|group| group.has_changes);

	let response = load_game_ids(&database, initial_query.clone());
	let rows = make_rows(&response.game_ids);
	let database = Arc::new(database);

	let window = MainWindow::new()?;
	i18n::apply_translation(initial_locale);
	window.set_selected_language(
		settings
			.borrow()
			.override_language
			.map(i18n::locale_code)
			.unwrap_or_default()
			.into(),
	);
	window.set_detected_language_name(
		detected_locale
			.map(i18n::native_name)
			.unwrap_or_default()
			.into(),
	);
	window.set_selected_color_scheme(color_scheme_code(settings.borrow().color_scheme).into());
	window.set_app_version(env!("CARGO_PKG_VERSION").into());
	window.set_search_text(initial_query.search.clone().into());
	window.set_sort_index(sort_index(initial_query.sort_by));
	window.set_sort_descending(initial_query.sort_descending);
	window.set_current_tab(initial_tab);
	window.set_filter_groups(Rc::new(slint::VecModel::from(initial_groups)).into());
	window.set_filters_have_changes(initial_have_changes);

	let renderer_name = match window.window().set_rendering_notifier(|_, _| {}) {
		Ok(()) => "FemtoVG (OpenGL)",
		Err(slint::SetRenderingNotifierError::Unsupported) => "Software",
		Err(_) => "Unknown",
	};
	window.set_renderer_name(renderer_name.into());

	let window_weak = window.as_weak();
	let scheduler = spawn_fetchers(database.clone(), window_weak.clone());
	scheduler.begin_query();

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
		schedule_visible_range(&ui_scheduler, &rows, first.max(0) as usize, last as usize);
	});

	let selected_game: SelectedGameState = Arc::new(Mutex::new(None));

	let ui_settings = settings.clone();
	let ui_window = window_weak.clone();
	let ui_database = database.clone();
	let ui_scheduler = scheduler.clone();
	window.on_search_changed(move |text| {
		{
			let mut settings = ui_settings.borrow_mut();
			settings.games_query.search = text.to_string();
		}
		persist_settings(&ui_settings);

		start_query(
			ui_window.clone(),
			ui_database.clone(),
			ui_scheduler.clone(),
			ui_settings.borrow().games_query.clone(),
			Duration::from_millis(200),
		);
	});

	let ui_settings = settings.clone();
	let ui_window = window_weak.clone();
	let ui_database = database.clone();
	let ui_scheduler = scheduler.clone();
	window.on_sort_changed(move |index| {
		{
			let mut settings = ui_settings.borrow_mut();
			let sort_by = sort_from_index(index);
			if settings.games_query.sort_by != sort_by {
				settings.games_query.sort_descending = false;
			}
			settings.games_query.sort_by = sort_by;
		}
		persist_settings(&ui_settings);

		start_query(
			ui_window.clone(),
			ui_database.clone(),
			ui_scheduler.clone(),
			ui_settings.borrow().games_query.clone(),
			Duration::ZERO,
		);
	});

	let ui_settings = settings.clone();
	let ui_window = window_weak.clone();
	let ui_database = database.clone();
	let ui_scheduler = scheduler.clone();
	window.on_sort_direction_changed(move |descending| {
		{
			let mut settings = ui_settings.borrow_mut();
			settings.games_query.sort_descending = descending;
		}
		persist_settings(&ui_settings);

		start_query(
			ui_window.clone(),
			ui_database.clone(),
			ui_scheduler.clone(),
			ui_settings.borrow().games_query.clone(),
			Duration::ZERO,
		);
	});

	let ui_window = window_weak.clone();
	let ui_database = database.clone();
	let ui_scheduler = scheduler.clone();
	let ui_settings = settings.clone();
	let ui_mod_families = mod_families.clone();
	window.on_filter_toggled(move |group, value, checked| {
		handle_filter_action(
			&ui_window,
			&ui_database,
			&ui_scheduler,
			&ui_settings,
			&ui_mod_families,
			group.as_str(),
			value.as_str(),
			FilterAction::Toggle(checked),
		);
	});

	let ui_window = window_weak.clone();
	let ui_database = database.clone();
	let ui_scheduler = scheduler.clone();
	let ui_settings = settings.clone();
	let ui_mod_families = mod_families.clone();
	window.on_filter_lock_toggled(move |group, value| {
		handle_filter_action(
			&ui_window,
			&ui_database,
			&ui_scheduler,
			&ui_settings,
			&ui_mod_families,
			group.as_str(),
			value.as_str(),
			FilterAction::Lock,
		);
	});

	let ui_window = window_weak.clone();
	let ui_database = database.clone();
	let ui_scheduler = scheduler.clone();
	let ui_settings = settings.clone();
	let ui_mod_families = mod_families.clone();
	window.on_filter_only(move |group, value| {
		handle_filter_action(
			&ui_window,
			&ui_database,
			&ui_scheduler,
			&ui_settings,
			&ui_mod_families,
			group.as_str(),
			value.as_str(),
			FilterAction::Only,
		);
	});

	let ui_window = window_weak.clone();
	let ui_database = database.clone();
	let ui_scheduler = scheduler.clone();
	let ui_settings = settings.clone();
	let ui_mod_families = mod_families.clone();
	window.on_filter_reset(move |group| {
		reset_filter(
			&ui_window,
			&ui_database,
			&ui_scheduler,
			&ui_settings,
			&ui_mod_families,
			group.as_str(),
		);
	});

	let ui_window = window_weak.clone();
	let ui_database = database.clone();
	let ui_scheduler = scheduler.clone();
	let ui_settings = settings.clone();
	let ui_mod_families = mod_families.clone();
	window.on_filters_reset(move || {
		reset_filters(
			&ui_window,
			&ui_database,
			&ui_scheduler,
			&ui_settings,
			&ui_mod_families,
		);
	});

	let ui_window = window_weak.clone();
	let ui_settings = settings.clone();
	let ui_mod_families = mod_families.clone();
	window.on_filter_expanded_changed(move |group, expanded| {
		{
			let mut settings = ui_settings.borrow_mut();
			if expanded {
				settings.expanded_filters.insert(group.to_string());
			} else {
				settings.expanded_filters.remove(group.as_str());
			}
		}
		persist_settings(&ui_settings);

		let groups = {
			let settings = ui_settings.borrow();
			build_groups(
				&settings.games_query.filter,
				&ui_mod_families,
				&settings.expanded_filters,
			)
		};

		if let Some(window) = ui_window.upgrade() {
			window.set_filter_groups(Rc::new(slint::VecModel::from(groups)).into());
		}
	});

	let ui_settings = settings.clone();
	let ui_window = window_weak.clone();
	window.on_language_changed(move |value| {
		let override_language = if value.is_empty() {
			None
		} else {
			match i18n::locale_from_code(value.as_str()) {
				Some(locale) => Some(locale),
				None => {
					eprintln!("Invalid language selection {value}");
					return;
				}
			}
		};

		{
			let mut settings = ui_settings.borrow_mut();
			settings.override_language = override_language;
		}
		persist_settings(&ui_settings);

		let locale = i18n::effective_locale(&ui_settings.borrow(), detected_locale);
		i18n::apply_translation(locale);

		if let Some(window) = ui_window.upgrade() {
			window.set_selected_language(value);
		}
	});

	let ui_settings = settings.clone();
	window.on_color_scheme_changed(move |value| {
		let color_scheme = match value.as_str() {
			"dark" => ColorScheme::Dark,
			"light" => ColorScheme::Light,
			_ => ColorScheme::Auto,
		};

		ui_settings.borrow_mut().color_scheme = color_scheme;
		persist_settings(&ui_settings);
	});

	let ui_settings = settings.clone();
	window.on_tab_changed(move |index| {
		let mut settings = ui_settings.borrow_mut();
		settings.selected_tab = tab_from_index(index);
		drop(settings);
		persist_settings(&ui_settings);
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

		selected::show(&window, &game, row);
		mods::refresh_mods(&window, &ui_database, &game);
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
	window.on_reinstall_mod(move |mod_id| {
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

	let ui_window = window_weak.clone();
	let ui_database = database.clone();
	let ui_selected_game = selected_game.clone();
	window.on_download_mod_config(move |mod_id| {
		let Some((_, game)) = get_selected_game(&ui_selected_game) else {
			return;
		};

		mods::download_mod_config(
			ui_database.clone(),
			ui_window.clone(),
			game,
			mod_id.to_string(),
		);
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
				selected::show(&window, &game, row);
				mods::refresh_mods(&window, &database, &game);
			});
		});
	});

	let model = Rc::new(slint::VecModel::from(rows));
	window.set_game_rows(model.into());
	window.set_total_count(response.total_count as i32);
	window.run()
}

fn game_from_row(
	database: &Arc<rai_pal_core::local_database::app_database::DbMutex>,
	row: &GameRowData,
) -> Option<DbGame> {
	if let Ok(game) = serde_json::from_str::<DbGame>(row.game_json.as_str()) {
		return Some(game);
	}

	let provider_id = row.provider_id.parse::<GameProviderId>().ok()?;
	database.get_game(&provider_id, row.game_id.as_str()).ok()
}

fn get_selected_game(state: &SelectedGameState) -> Option<(usize, DbGame)> {
	state
		.lock()
		.ok()
		.and_then(|selected_game| selected_game.clone())
}

fn persist_settings(settings: &Rc<RefCell<AppSettings>>) {
	if let Err(err) = settings.borrow().try_write() {
		eprintln!("Failed to save app settings: {err}");
	}
}

fn refresh_filter_groups(
	window: &slint::Weak<MainWindow>,
	settings: &Rc<RefCell<AppSettings>>,
	mod_families: &[String],
) {
	let groups = {
		let settings = settings.borrow();
		build_groups(
			&settings.games_query.filter,
			mod_families,
			&settings.expanded_filters,
		)
	};

	if let Some(window) = window.upgrade() {
		window.set_filters_have_changes(groups.iter().any(|group| group.has_changes));
		window.set_filter_groups(Rc::new(slint::VecModel::from(groups)).into());
	}
}

fn handle_filter_action(
	window: &slint::Weak<MainWindow>,
	database: &Arc<DbMutex>,
	scheduler: &Arc<FetchScheduler>,
	settings: &Rc<RefCell<AppSettings>>,
	mod_families: &[String],
	group: &str,
	value: &str,
	action: FilterAction,
) {
	let changed = {
		let mut settings = settings.borrow_mut();
		apply_filter_action(
			&mut settings.games_query.filter,
			mod_families,
			group,
			value,
			action,
		)
	};

	if !changed {
		return;
	}

	persist_settings(settings);
	refresh_filter_groups(window, settings, mod_families);

	let query = settings.borrow().games_query.clone();
	start_query(
		window.clone(),
		database.clone(),
		scheduler.clone(),
		query,
		Duration::ZERO,
	);
}

fn reset_filter(
	window: &slint::Weak<MainWindow>,
	database: &Arc<DbMutex>,
	scheduler: &Arc<FetchScheduler>,
	settings: &Rc<RefCell<AppSettings>>,
	mod_families: &[String],
	group: &str,
) {
	let changed = {
		let mut settings = settings.borrow_mut();
		reset_filter_group(&mut settings.games_query.filter, group)
	};

	if !changed {
		return;
	}

	persist_settings(settings);
	refresh_filter_groups(window, settings, mod_families);

	let query = settings.borrow().games_query.clone();
	start_query(
		window.clone(),
		database.clone(),
		scheduler.clone(),
		query,
		Duration::ZERO,
	);
}

fn reset_filters(
	window: &slint::Weak<MainWindow>,
	database: &Arc<DbMutex>,
	scheduler: &Arc<FetchScheduler>,
	settings: &Rc<RefCell<AppSettings>>,
	mod_families: &[String],
) {
	{
		let mut settings = settings.borrow_mut();
		reset_all_filters(&mut settings.games_query.filter);
	}

	persist_settings(settings);
	refresh_filter_groups(window, settings, mod_families);

	let query = settings.borrow().games_query.clone();
	start_query(
		window.clone(),
		database.clone(),
		scheduler.clone(),
		query,
		Duration::ZERO,
	);
}

fn color_scheme_code(color_scheme: ColorScheme) -> &'static str {
	match color_scheme {
		ColorScheme::Dark => "dark",
		ColorScheme::Light => "light",
		ColorScheme::Auto => "auto",
	}
}

fn sort_index(sort_by: GamesSortBy) -> i32 {
	match sort_by {
		GamesSortBy::Title => 0,
		GamesSortBy::Engine => 1,
		GamesSortBy::ReleaseDate => 2,
	}
}

fn sort_from_index(index: i32) -> GamesSortBy {
	match index {
		1 => GamesSortBy::Engine,
		2 => GamesSortBy::ReleaseDate,
		_ => GamesSortBy::Title,
	}
}

fn tab_index(tab: TabId) -> i32 {
	match tab {
		TabId::Games => 0,
		TabId::Mods => 1,
		TabId::Thanks => 2,
	}
}

fn tab_from_index(index: i32) -> TabId {
	match index {
		1 => TabId::Mods,
		2 => TabId::Thanks,
		_ => TabId::Games,
	}
}
