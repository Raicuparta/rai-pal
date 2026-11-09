use std::{
	cell::RefCell,
	rc::Rc,
	sync::{Arc, Mutex},
	thread,
	time::Duration,
};

use rai_pal_core::{
	app_settings::{AppSettings, ColorScheme, TabId, WindowState},
	game::Game,
	game_providers::{game_provider::GameProviderId, provider_command::ProviderCommandAction},
	games_query::{FilterAction, GamesSortBy},
	local_database::{
		app_database::DbMutex,
		game_database::{GameDatabase, try_create},
		mod_database::ModDatabase,
	},
	result::LogErrExt,
};
use slint::{
	ComponentHandle, Model,
	winit_030::{EventResult, WinitWindowAccessor, winit},
};

use crate::{
	ActionStates, AppColorScheme, AppTab, Fonts, MainWindow, ModActions, action_state, actions,
	analytics, confirmation, deep_links, downloads,
	fetch::spawn_fetchers,
	filters::{
		apply_filter_action, build_groups, load_mod_families, reset_all_filters, reset_filter_group,
	},
	i18n, logging, manual, mod_sources,
	model::{self, make_rows},
	mods, mods_tab, notifications,
	query::{load_game_ids, start_query},
	refresh,
	scheduler::{FetchScheduler, enqueue_index, schedule_visible_range},
	selected, single_instance, thanks, tools, updater, user,
};

pub fn run() -> Result<(), slint::PlatformError> {
	fix_gamescope_scale_factor();
	logging::init();

	let Some(_single_instance) = single_instance::acquire() else {
		return Ok(());
	};

	analytics::send(
		rai_pal_core::analytics::Event::StartApp,
		&[
			("app_version", env!("CARGO_PKG_VERSION").to_string()),
			("platform", std::env::consts::OS.to_string()),
			(
				"mode",
				if cfg!(debug_assertions) {
					"dev"
				} else {
					"release"
				}
				.to_string(),
			),
		],
	);

	let database = match try_create() {
		Ok(database) => database,
		Err(err) => {
			log::error!("Failed to open local database: {err}");
			return Ok(());
		}
	};

	if let Err(err) = database.setup_mod_tables() {
		log::error!("Failed to set up mod tables: {err}");
	}

	let settings = Rc::new(RefCell::new(AppSettings::read()));
	let detected_locale = i18n::detect_locale();
	let initial_locale = i18n::effective_locale(&settings.borrow(), detected_locale);
	let mod_families = load_mod_families(&database);

	let initial_query = settings.borrow().games_query.clone();
	let initial_tab = app_tab(settings.borrow().selected_tab);
	let initial_groups = build_groups(
		&initial_query.filter,
		&mod_families,
		&settings.borrow().expanded_filters,
	);

	let initial_have_changes = initial_groups.iter().any(|group| group.has_changes);

	let response = load_game_ids(&database, initial_query.clone()).unwrap_or_else(|err| {
		log::error!("Failed to load initial games: {err}");
		rai_pal_core::local_database::game_database::GameIdsResponse {
			game_ids: Vec::new(),
			total_count: 0,
		}
	});
	let rows = make_rows(&response.game_ids);
	let games_total_count = response.total_count;
	let database = Arc::new(database);

	let window = MainWindow::new()?;
	window.set_window_title(if cfg!(debug_assertions) {
		"Rai Pal DEV".into()
	} else {
		"Rai Pal".into()
	});
	apply_window_state(&window, settings.borrow().window_state);
	window
		.global::<ActionStates>()
		.on_lookup(|id| action_state::lookup(id.as_str()));
	i18n::apply_translation(initial_locale);
	window.global::<crate::AppState>().set_selected_language(
		settings
			.borrow()
			.override_language
			.map(i18n::locale_code)
			.unwrap_or_default()
			.into(),
	);
	window
		.global::<crate::AppState>()
		.set_detected_language_name(
			detected_locale
				.map(i18n::native_name)
				.unwrap_or_default()
				.into(),
		);
	window
		.global::<crate::AppState>()
		.set_selected_color_scheme(app_color_scheme(settings.borrow().color_scheme));
	window
		.global::<crate::AppState>()
		.set_hide_game_thumbnails(settings.borrow().hide_game_thumbnails);
	mods_tab::load(&window, &database);
	window
		.global::<crate::AppState>()
		.set_app_version(env!("CARGO_PKG_VERSION").into());
	user::load(window.as_weak());
	if initial_tab == AppTab::Thanks {
		thanks::load(window.as_weak());
	}
	window
		.global::<crate::GamesState>()
		.set_search_text(initial_query.search.clone().into());
	window.global::<crate::GamesState>().set_total_count(
		i32::try_from(games_total_count).unwrap_or_else(|err| {
			log::warn!("Games count exceeds i32: {err}");
			i32::MAX
		}),
	);
	window
		.global::<crate::GamesState>()
		.set_sort_index(sort_option_index(
			initial_query.sort_by,
			initial_query.sort_descending,
		));
	window
		.global::<crate::AppState>()
		.set_current_tab(initial_tab);
	window
		.global::<crate::GamesState>()
		.set_filters_have_expanded(initial_groups.iter().any(|group| group.expanded));
	window
		.global::<crate::GamesState>()
		.set_filter_groups(Rc::new(slint::VecModel::from(initial_groups)).into());
	window
		.global::<crate::GamesState>()
		.set_filters_have_changes(initial_have_changes);

	let renderer_name = match window.window().set_rendering_notifier(|_, _| {}) {
		Ok(()) => "FemtoVG (OpenGL)",
		Err(slint::SetRenderingNotifierError::Unsupported) => "Software",
		Err(err) => {
			log::warn!("Failed to determine renderer name: {err}");
			"Unknown"
		}
	};
	window
		.global::<crate::AppState>()
		.set_renderer_name(renderer_name.into());
	window
		.global::<crate::AppState>()
		.set_platform_name(platform_name().into());
	window
		.global::<Fonts>()
		.set_monospace_family(monospace_font_family().into());

	let window_weak = window.as_weak();
	let scheduler = spawn_fetchers(database.clone(), window_weak.clone());
	scheduler.set_thumbnails_enabled(!settings.borrow().hide_game_thumbnails);
	scheduler.begin_query();

	let ui_window = window_weak.clone();
	let ui_scheduler = scheduler.clone();
	window
		.global::<crate::GamesActions>()
		.on_request_game(move |index| {
			let Some(window) = ui_window.upgrade() else {
				return;
			};

			let rows = window.global::<crate::GamesState>().get_rows();
			enqueue_index(&ui_scheduler, &rows, index.max(0) as usize);
		});

	let ui_window = window_weak.clone();
	let ui_scheduler = scheduler.clone();
	window
		.global::<crate::GamesActions>()
		.on_visible_range_changed(move |first, last| {
			if last < 0 {
				return;
			}

			let Some(window) = ui_window.upgrade() else {
				return;
			};

			let rows = window.global::<crate::GamesState>().get_rows();
			schedule_visible_range(&ui_scheduler, &rows, first.max(0) as usize, last as usize);
		});

	let selected_game: selected::SelectedGameState = Arc::new(Mutex::new(None));
	let pending_confirmation = Rc::new(confirmation::PendingState::default());

	let ui_settings = settings.clone();
	let ui_window = window_weak.clone();
	let ui_database = database.clone();
	let ui_scheduler = scheduler.clone();
	let ui_selected_game = selected_game.clone();
	window.global::<crate::Refresh>().on_refresh_all(move || {
		let Some(window) = ui_window.upgrade() else {
			return;
		};

		begin_refresh_all(
			&window,
			ui_database.clone(),
			ui_scheduler.clone(),
			&ui_settings,
			&ui_selected_game,
		);
	});

	let ui_window = window_weak.clone();
	window
		.global::<crate::DialogActions>()
		.on_request_url_mod_sources(move || {
			if let Some(window) = ui_window.upgrade() {
				mod_sources::show(&window);
			}
		});

	let ui_window = window_weak.clone();
	let ui_database = database.clone();
	let ui_selected_game = selected_game.clone();
	window
		.global::<crate::DialogActions>()
		.on_set_url_mod_source_enabled(move |url, enabled| {
			let Some(window) = ui_window.upgrade() else {
				return;
			};

			mod_sources::set_enabled(
				&window,
				ui_database.clone(),
				ui_selected_game.clone(),
				url.to_string(),
				enabled,
			);
		});

	let ui_window = window_weak.clone();
	let ui_database = database.clone();
	let ui_selected_game = selected_game.clone();
	window
		.global::<crate::DialogActions>()
		.on_remove_url_mod_source(move |url| {
			let Some(window) = ui_window.upgrade() else {
				return;
			};

			mod_sources::remove(
				&window,
				ui_database.clone(),
				ui_selected_game.clone(),
				url.to_string(),
			);
		});

	let ui_window = window_weak.clone();
	window
		.global::<crate::DialogActions>()
		.on_add_url_mod_source(move |url| {
			if let Some(window) = ui_window.upgrade() {
				mod_sources::request_add(&window, url.to_string());
			}
		});

	let ui_window = window_weak.clone();
	let ui_database = database.clone();
	let ui_selected_game = selected_game.clone();
	window
		.global::<crate::DialogActions>()
		.on_confirm_url_mod_source(move || {
			let Some(window) = ui_window.upgrade() else {
				return;
			};

			mod_sources::confirm_add(&window, ui_database.clone(), ui_selected_game.clone());
		});

	let ui_window = window_weak.clone();
	window
		.global::<crate::DialogActions>()
		.on_cancel_url_mod_source(move || {
			if let Some(window) = ui_window.upgrade() {
				window
					.global::<crate::ModSourcesState>()
					.set_confirm_loading(false);
				window
					.global::<crate::ModSourcesState>()
					.set_confirm_error("".into());
				window
					.global::<crate::ModSourcesState>()
					.set_confirm_mods(Default::default());
				window
					.global::<crate::ModSourcesState>()
					.set_confirm_saving(false);
			}
		});

	let ui_window = window_weak.clone();
	let ui_database = database.clone();
	let ui_scheduler = scheduler.clone();
	let ui_settings = settings.clone();
	let ui_selected_game = selected_game.clone();
	window
		.global::<crate::DialogActions>()
		.on_reset_steam_cache(move || {
			if let Some(window) = ui_window.upgrade()
				&& !window
					.global::<crate::SteamToolsState>()
					.get_cache_running()
			{
				tools::reset_steam_cache(
					ui_window.clone(),
					ui_database.clone(),
					ui_scheduler.clone(),
					ui_settings.borrow().games_query.clone(),
					ui_selected_game.clone(),
				);
			}
		});

	let ui_window = window_weak.clone();
	window
		.global::<crate::DialogActions>()
		.on_add_steam_shortcut(move || {
			if let Some(window) = ui_window.upgrade()
				&& !window
					.global::<crate::SteamToolsState>()
					.get_shortcut_running()
			{
				tools::add_steam_shortcut(ui_window.clone());
			}
		});

	let ui_window = window_weak.clone();
	window
		.global::<crate::AppActions>()
		.on_open_logs_folder(move || {
			actions::open_logs_folder(ui_window.clone(), "open-logs-folder".to_string());
		});

	let ui_window = window_weak.clone();
	let ui_database = database.clone();
	let ui_scheduler = scheduler.clone();
	let ui_settings = settings.clone();
	let ui_mod_families = mod_families.clone();
	window
		.global::<crate::AppActions>()
		.on_reset_settings(move || {
			let Some(window) = ui_window.upgrade() else {
				return;
			};

			let window_state = ui_settings.borrow().window_state;
			let mut settings = AppSettings::default();
			settings.window_state = window_state;
			*ui_settings.borrow_mut() = settings;
			persist_settings(&ui_settings);

			let locale = i18n::effective_locale(&ui_settings.borrow(), detected_locale);
			i18n::apply_translation(locale);

			window
				.global::<crate::AppState>()
				.set_selected_language("".into());
			window
				.global::<crate::AppState>()
				.set_selected_color_scheme(AppColorScheme::Auto);
			window
				.global::<crate::AppState>()
				.set_hide_game_thumbnails(false);
			window
				.global::<crate::GamesState>()
				.set_search_text("".into());
			window.global::<crate::GamesState>().set_sort_index(0);
			window
				.global::<crate::AppState>()
				.set_current_tab(AppTab::Games);
			ui_scheduler.set_thumbnails_enabled(true);

			let groups = build_groups(
				&ui_settings.borrow().games_query.filter,
				&ui_mod_families,
				&ui_settings.borrow().expanded_filters,
			);
			window
				.global::<crate::GamesState>()
				.set_filters_have_changes(groups.iter().any(|group| group.has_changes));
			window
				.global::<crate::GamesState>()
				.set_filters_have_expanded(groups.iter().any(|group| group.expanded));
			window
				.global::<crate::GamesState>()
				.set_filter_groups(Rc::new(slint::VecModel::from(groups)).into());

			start_query(
				ui_window.clone(),
				ui_database.clone(),
				ui_scheduler.clone(),
				ui_settings.borrow().games_query.clone(),
				Duration::ZERO,
			);
		});

	let ui_window = window_weak.clone();
	window
		.global::<crate::DialogActions>()
		.on_request_add_game(move || {
			let Some(window) = ui_window.upgrade() else {
				return;
			};

			window
				.global::<crate::AppState>()
				.set_current_tab(AppTab::Games);
			window
				.global::<crate::AppActions>()
				.invoke_tab_changed(AppTab::Games);
			manual::show(&window);
		});

	let ui_window = window_weak.clone();
	window
		.global::<crate::DialogActions>()
		.on_add_game_closed(move || {
			if let Some(window) = ui_window.upgrade() {
				manual::cancel_scan(&window);
			}
		});

	let ui_window = window_weak.clone();
	let ui_database = database.clone();
	let ui_scheduler = scheduler.clone();
	let ui_settings = settings.clone();
	window
		.global::<crate::DialogActions>()
		.on_select_game_executable(move || {
			let Some(window) = ui_window.upgrade() else {
				return;
			};

			manual::select_executable(
				&window,
				ui_database.clone(),
				ui_scheduler.clone(),
				ui_settings.borrow().games_query.clone(),
			);
		});

	let ui_window = window_weak.clone();
	window
		.global::<crate::DialogActions>()
		.on_select_game_directory(move || {
			let Some(window) = ui_window.upgrade() else {
				return;
			};

			manual::select_directory(&window);
		});

	let ui_window = window_weak.clone();
	window
		.global::<crate::DialogActions>()
		.on_cancel_game_scan(move || {
			if let Some(window) = ui_window.upgrade() {
				manual::cancel_scan(&window);
			}
		});

	let ui_window = window_weak.clone();
	let ui_database = database.clone();
	let ui_scheduler = scheduler.clone();
	let ui_settings = settings.clone();
	window
		.global::<crate::DialogActions>()
		.on_confirm_game_directory(move || {
			let Some(window) = ui_window.upgrade() else {
				return;
			};

			manual::confirm_add_directory(
				&window,
				ui_database.clone(),
				ui_scheduler.clone(),
				ui_settings.borrow().games_query.clone(),
			);
		});

	let ui_window = window_weak.clone();
	let ui_database = database.clone();
	let ui_scheduler = scheduler.clone();
	let ui_settings = settings.clone();
	window
		.global::<crate::DialogActions>()
		.on_remove_game_directory(move |path| {
			let Some(window) = ui_window.upgrade() else {
				return;
			};

			manual::remove_directory(
				&window,
				ui_database.clone(),
				ui_scheduler.clone(),
				ui_settings.borrow().games_query.clone(),
				path.to_string(),
			);
		});

	let ui_window = window_weak.clone();
	let ui_database = database.clone();
	let ui_scheduler = scheduler.clone();
	let ui_settings = settings.clone();
	let ui_pending_confirmation = pending_confirmation.clone();
	let ui_selected_game = selected_game.clone();
	window.global::<crate::GamesActions>().on_remove_game(
		move |confirmation_text, confirm_label| {
			let Some((_, game)) = get_selected_game(&ui_selected_game) else {
				return;
			};

			let Some(window) = ui_window.upgrade() else {
				return;
			};

			confirmation::request_remove_game(
				&window,
				&ui_settings,
				&ui_pending_confirmation,
				ui_database.clone(),
				ui_scheduler.clone(),
				ui_settings.borrow().games_query.clone(),
				ui_selected_game.clone(),
				game,
				confirmation_text.to_string(),
				confirm_label.to_string(),
			);
		},
	);

	let ui_window = window_weak.clone();
	window.global::<crate::AppActions>().on_log_in(move || {
		user::log_in(ui_window.clone());
	});

	let ui_window = window_weak.clone();
	window.global::<crate::AppActions>().on_log_out(move || {
		user::log_out(ui_window.clone());
	});

	let ui_window = window.as_weak();
	window
		.global::<crate::AppActions>()
		.on_dismiss_notification(move |id| {
			if let Some(window) = ui_window.upgrade() {
				notifications::dismiss(&window, id);
			}
		});

	let ui_window = window_weak.clone();
	let ui_selected_game = selected_game.clone();
	window
		.global::<crate::GamesActions>()
		.on_request_debug_data(move || {
			let Some(window) = ui_window.upgrade() else {
				return;
			};
			let Some((_, game)) = get_selected_game(&ui_selected_game) else {
				return;
			};

			match serde_json::to_string_pretty(&game) {
				Ok(json) => window
					.global::<crate::DebugState>()
					.set_details_json(json.into()),
				Err(err) => {
					notifications::error(&window, format!("Failed to serialize game: {err}"))
				}
			}
		});

	let ui_window = window_weak.clone();
	let ui_database = database.clone();
	window
		.global::<crate::GamesActions>()
		.on_request_mod_debug_data(move |mod_id| {
			let Some(window) = ui_window.upgrade() else {
				return;
			};

			match ui_database.get_mod(mod_id.as_str()) {
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
		});

	let ui_window = window_weak.clone();
	let ui_settings = settings.clone();
	let ui_pending_confirmation = pending_confirmation.clone();
	let ui_database = database.clone();
	window
		.global::<crate::DialogActions>()
		.on_confirmation_response(move |confirmed| {
			if let Some(window) = ui_window.upgrade() {
				confirmation::respond(
					&window,
					&ui_settings,
					&ui_pending_confirmation,
					ui_database.clone(),
					confirmed,
				);
			}
		});

	let ui_settings = settings.clone();
	let ui_window = window_weak.clone();
	let ui_database = database.clone();
	let ui_scheduler = scheduler.clone();
	window
		.global::<crate::GamesActions>()
		.on_search_changed(move |text| {
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
	window
		.global::<crate::GamesActions>()
		.on_sort_changed(move |index| {
			{
				let mut settings = ui_settings.borrow_mut();
				let (sort_by, sort_descending) = sort_option_from_index(index);
				settings.games_query.sort_by = sort_by;
				settings.games_query.sort_descending = sort_descending;
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
	window
		.global::<crate::GamesActions>()
		.on_filter_toggled(move |group, value, checked| {
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
	window
		.global::<crate::GamesActions>()
		.on_filter_lock_toggled(move |group, value| {
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
	window
		.global::<crate::GamesActions>()
		.on_filter_only(move |group, value| {
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
	window
		.global::<crate::GamesActions>()
		.on_filter_reset(move |group| {
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
	window
		.global::<crate::GamesActions>()
		.on_filters_reset(move || {
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
	window
		.global::<crate::GamesActions>()
		.on_filters_collapse_all(move || {
			{
				let mut settings = ui_settings.borrow_mut();
				settings.expanded_filters.clear();
			}
			persist_settings(&ui_settings);
			refresh_filter_groups(&ui_window, &ui_settings, &ui_mod_families);
		});

	let ui_window = window_weak.clone();
	let ui_settings = settings.clone();
	let ui_mod_families = mod_families.clone();
	window
		.global::<crate::GamesActions>()
		.on_filter_expanded_changed(move |group, expanded| {
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
				window
					.global::<crate::GamesState>()
					.set_filters_have_expanded(groups.iter().any(|group| group.expanded));
				window
					.global::<crate::GamesState>()
					.set_filter_groups(Rc::new(slint::VecModel::from(groups)).into());
			}
		});

	let ui_settings = settings.clone();
	let ui_window = window_weak.clone();
	window
		.global::<crate::AppActions>()
		.on_language_changed(move |value| {
			let override_language = if value.is_empty() {
				None
			} else {
				match i18n::locale_from_code(value.as_str()) {
					Some(locale) => Some(locale),
					None => {
						log::error!("Invalid language selection {value}");
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
				window
					.global::<crate::AppState>()
					.set_selected_language(value);
			}
		});

	let ui_settings = settings.clone();
	window
		.global::<crate::AppActions>()
		.on_color_scheme_changed(move |value| {
			ui_settings.borrow_mut().color_scheme = match value {
				AppColorScheme::Dark => ColorScheme::Dark,
				AppColorScheme::Light => ColorScheme::Light,
				AppColorScheme::Auto => ColorScheme::Auto,
			};
			persist_settings(&ui_settings);
		});

	let ui_settings = settings.clone();
	let ui_window = window_weak.clone();
	let ui_database = database.clone();
	let ui_scheduler = scheduler.clone();
	window
		.global::<crate::AppActions>()
		.on_hide_game_thumbnails_changed(move |hide| {
			ui_settings.borrow_mut().hide_game_thumbnails = hide;
			persist_settings(&ui_settings);
			ui_scheduler.set_thumbnails_enabled(!hide);

			if !hide {
				start_query(
					ui_window.clone(),
					ui_database.clone(),
					ui_scheduler.clone(),
					ui_settings.borrow().games_query.clone(),
					Duration::ZERO,
				);
			}
		});

	let ui_settings = settings.clone();
	let ui_window = window_weak.clone();
	let ui_database = database.clone();
	window
		.global::<crate::AppActions>()
		.on_tab_changed(move |tab| {
			{
				let mut settings = ui_settings.borrow_mut();
				settings.selected_tab = core_tab(tab);
			}
			persist_settings(&ui_settings);

			if let Some(window) = ui_window.upgrade() {
				if tab == AppTab::Mods {
					mods_tab::load(&window, &ui_database);
				} else if tab == AppTab::Thanks
					&& window
						.global::<crate::ThanksState>()
						.get_patrons()
						.row_count() == 0
				{
					thanks::load(window.as_weak());
				}
			}
		});

	let ui_window = window_weak.clone();
	let ui_database = database.clone();
	let ui_selected_game = selected_game.clone();
	window
		.global::<crate::GamesActions>()
		.on_select_game(move |index| {
			let index = index.max(0) as usize;

			let Some(window) = ui_window.upgrade() else {
				return;
			};

			let rows = window.global::<crate::GamesState>().get_rows();
			let Some(row) = rows.row_data(index) else {
				return;
			};

			let Some(provider_id) = row
				.provider_id
				.parse::<GameProviderId>()
				.ok_or_log("Failed to parse game provider ID")
			else {
				return;
			};

			let Some(game) = ui_database
				.get_game(&provider_id, row.game_id.as_str())
				.ok_or_log("Failed to load game")
			else {
				return;
			};

			if let Some(mut selected_game) = ui_selected_game
				.lock()
				.ok_or_log("Failed to lock selected game")
			{
				*selected_game = Some((index, game.clone()));
			}

			selected::show(&window, &game, row);
			mods::refresh_mods(&window, &ui_database, &game);
		});

	let ui_window = window_weak.clone();
	let ui_selected_game = selected_game.clone();
	window.global::<crate::GamesActions>().on_back(move || {
		if let Some(mut selected_game) = ui_selected_game
			.lock()
			.ok_or_log("Failed to lock selected game")
		{
			*selected_game = None;
		}

		if let Some(window) = ui_window.upgrade() {
			selected::hide(&window);
		}
	});

	let ui_window = window_weak.clone();
	let ui_database = database.clone();
	let ui_selected_game = selected_game.clone();
	window
		.global::<crate::GamesActions>()
		.on_run_provider_action(move |value| {
			let Some(action) = value
				.parse::<ProviderCommandAction>()
				.ok_or_log("Failed to parse provider action")
			else {
				return;
			};

			let Some((_, game)) = get_selected_game(&ui_selected_game) else {
				return;
			};

			analytics::send(
				rai_pal_core::analytics::Event::ProviderCommand,
				&[
					("action", action.to_string()),
					("game", game.display_title.clone()),
				],
			);

			if let Some(window) = ui_window.upgrade() {
				notifications::info(&window, "Running command... This might take a few seconds.");
			}

			actions::run_provider_action(
				ui_database.clone(),
				ui_window.clone(),
				game,
				action,
				format!("provider-action:{value}"),
			);
		});

	let ui_window = window_weak.clone();
	let ui_selected_game = selected_game.clone();
	window
		.global::<crate::GamesActions>()
		.on_open_game_folder(move || {
			if let Some((_, game)) = get_selected_game(&ui_selected_game) {
				actions::open_game_folder(ui_window.clone(), "open-game-folder".to_string(), game);
			}
		});

	let ui_window = window_weak.clone();
	let ui_selected_game = selected_game.clone();
	window
		.global::<crate::GamesActions>()
		.on_open_game_mods_folder(move || {
			if let Some((_, game)) = get_selected_game(&ui_selected_game) {
				actions::open_game_mods_folder(
					ui_window.clone(),
					"open-game-mods-folder".to_string(),
					game,
				);
			}
		});

	let ui_window = window_weak.clone();
	let ui_selected_game = selected_game.clone();
	window
		.global::<crate::GamesActions>()
		.on_open_game_data_folder(move || {
			if let Some((_, game)) = get_selected_game(&ui_selected_game) {
				actions::open_game_data_folder(
					ui_window.clone(),
					"open-game-data-folder".to_string(),
					game,
				);
			}
		});

	let ui_window = window_weak.clone();
	let ui_selected_game = selected_game.clone();
	window
		.global::<crate::GamesActions>()
		.on_open_wine_prefix_folder(move || {
			if let Some((_, game)) = get_selected_game(&ui_selected_game) {
				actions::open_wine_prefix_folder(
					ui_window.clone(),
					"open-wine-prefix-folder".to_string(),
					game,
				);
			}
		});

	let ui_window = window_weak.clone();
	let ui_selected_game = selected_game.clone();
	window
		.global::<crate::GamesActions>()
		.on_open_wine_binary_folder(move || {
			if let Some((_, game)) = get_selected_game(&ui_selected_game) {
				actions::open_wine_binary_folder(
					ui_window.clone(),
					"open-wine-binary-folder".to_string(),
					game,
				);
			}
		});

	let downloads = downloads::Tracker::new(window.as_weak());
	let updater = Rc::new(updater::Updater::start(window.as_weak()));

	let ui_window = window_weak.clone();
	let ui_updater = updater.clone();
	window
		.global::<crate::AppActions>()
		.on_install_update(move || {
			ui_updater.install(ui_window.clone());
		});

	let ui_window = window.as_weak();
	let ui_updater = updater.clone();
	window
		.global::<crate::AppActions>()
		.on_ignore_update(move || {
			if let Some(window) = ui_window.upgrade() {
				ui_updater.ignore(&window);
			}
		});

	deep_links::register();
	for argument in std::env::args().skip(1) {
		deep_links::handle(&argument, &window, &database, &selected_game, &downloads);
	}

	let instance_window = window.as_weak();
	let instance_database = database.clone();
	let instance_selected_game = selected_game.clone();
	let instance_downloads = downloads.clone();
	single_instance::listen(move |arguments| {
		let window = instance_window.clone();
		let database = instance_database.clone();
		let selected_game = instance_selected_game.clone();
		let downloads = instance_downloads.clone();

		crate::runtime::invoke_on_ui(move || {
			let Some(window) = window.upgrade() else {
				return;
			};

			window.window().set_minimized(false);
			window.window().with_winit_window(|winit_window| {
				winit_window.focus_window();
			});

			for argument in arguments {
				deep_links::handle(&argument, &window, &database, &selected_game, &downloads);
			}
		});
	});

	let ui_window = window_weak.clone();
	let ui_settings = settings.clone();
	let ui_pending_confirmation = pending_confirmation.clone();
	let ui_database = database.clone();
	let ui_selected_game = selected_game.clone();
	let ui_downloads = downloads.clone();
	window.global::<ModActions>().on_request_install(
		move |mod_id, confirmation_text, confirm_label, skip_id| {
			let Some((_, game)) = get_selected_game(&ui_selected_game) else {
				return;
			};

			let Some(window) = ui_window.upgrade() else {
				return;
			};

			confirmation::request_install(
				&window,
				&ui_settings,
				&ui_pending_confirmation,
				ui_database.clone(),
				game,
				mod_id.to_string(),
				ui_downloads.clone(),
				confirmation_text.to_string(),
				confirm_label.to_string(),
				skip_id.to_string(),
			);
		},
	);

	let ui_window = window_weak.clone();
	let ui_database = database.clone();
	let ui_selected_game = selected_game.clone();
	let ui_downloads = downloads.clone();
	window.global::<ModActions>().on_update(move |mod_id| {
		let Some((_, game)) = get_selected_game(&ui_selected_game) else {
			return;
		};

		analytics::send_mod(
			rai_pal_core::analytics::Event::UpdateMod,
			mod_id.as_str(),
			Some(&game),
		);
		mods::install_mod(
			ui_database.clone(),
			ui_window.clone(),
			game,
			mod_id.to_string(),
			ui_downloads.clone(),
		);
	});

	let ui_window = window_weak.clone();
	let ui_database = database.clone();
	let ui_selected_game = selected_game.clone();
	let ui_downloads = downloads.clone();
	window.global::<ModActions>().on_reinstall(move |mod_id| {
		let Some((_, game)) = get_selected_game(&ui_selected_game) else {
			return;
		};

		analytics::send_mod(
			rai_pal_core::analytics::Event::InstallMod,
			mod_id.as_str(),
			Some(&game),
		);
		mods::install_mod(
			ui_database.clone(),
			ui_window.clone(),
			game,
			mod_id.to_string(),
			ui_downloads.clone(),
		);
	});

	let ui_window = window_weak.clone();
	let ui_database = database.clone();
	let ui_selected_game = selected_game.clone();
	window.global::<ModActions>().on_uninstall(move |mod_id| {
		let Some((_, game)) = get_selected_game(&ui_selected_game) else {
			return;
		};

		analytics::send_mod(
			rai_pal_core::analytics::Event::UninstallMod,
			mod_id.as_str(),
			Some(&game),
		);
		mods::uninstall_mod(
			ui_database.clone(),
			ui_window.clone(),
			game,
			mod_id.to_string(),
		);
	});

	let ui_window = window_weak.clone();
	let ui_database = database.clone();
	let ui_selected_game = selected_game.clone();
	window.global::<ModActions>().on_run(move |mod_id| {
		let Some((_, game)) = get_selected_game(&ui_selected_game) else {
			return;
		};

		analytics::send_mod(
			rai_pal_core::analytics::Event::RunMod,
			mod_id.as_str(),
			Some(&game),
		);
		mods::run_mod(
			ui_database.clone(),
			ui_window.clone(),
			game,
			mod_id.to_string(),
		);
	});

	let ui_window = window_weak.clone();
	let ui_database = database.clone();
	window
		.global::<ModActions>()
		.on_run_standalone(move |mod_id| {
			actions::run_standalone_mod(ui_database.clone(), ui_window.clone(), mod_id.to_string());
		});

	let ui_window = window_weak.clone();
	let ui_database = database.clone();
	let ui_downloads = downloads.clone();
	window
		.global::<ModActions>()
		.on_install_standalone(move |mod_id| {
			actions::install_standalone_mod(
				ui_database.clone(),
				ui_window.clone(),
				mod_id.to_string(),
				ui_downloads.clone(),
			);
		});

	let ui_downloads = downloads.clone();
	window
		.global::<crate::AppActions>()
		.on_clear_downloads(move || {
			ui_downloads.clear_completed();
		});

	window.global::<ModActions>().on_open_url(move |url| {
		actions::open_url(url.to_string());
	});

	let ui_window = window_weak.clone();
	window
		.global::<ModActions>()
		.on_open_local_mods_folder(move || {
			actions::open_local_mods_folder(
				ui_window.clone(),
				"open-local-mods-folder".to_string(),
			);
		});

	let ui_window = window_weak.clone();
	let ui_database = database.clone();
	let ui_selected_game = selected_game.clone();
	window.global::<ModActions>().on_open_folder(move |mod_id| {
		let Some((_, game)) = get_selected_game(&ui_selected_game) else {
			return;
		};

		mods::open_mod_folder(
			ui_database.clone(),
			ui_window.clone(),
			game,
			mod_id.to_string(),
		);
	});

	let ui_window = window_weak.clone();
	let ui_database = database.clone();
	let ui_selected_game = selected_game.clone();
	window
		.global::<ModActions>()
		.on_download_config(move |mod_id| {
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

	let ui_window = window_weak.clone();
	let ui_database = database.clone();
	let ui_selected_game = selected_game.clone();
	window.global::<ModActions>().on_configure(move |mod_id| {
		let Some((_, game)) = get_selected_game(&ui_selected_game) else {
			return;
		};

		mods::configure_mod(
			ui_database.clone(),
			ui_window.clone(),
			game,
			mod_id.to_string(),
			false,
		);
	});

	let ui_window = window_weak.clone();
	let ui_database = database.clone();
	let ui_selected_game = selected_game.clone();
	window
		.global::<ModActions>()
		.on_configure_folder(move |mod_id| {
			let Some((_, game)) = get_selected_game(&ui_selected_game) else {
				return;
			};

			mods::configure_mod(
				ui_database.clone(),
				ui_window.clone(),
				game,
				mod_id.to_string(),
				true,
			);
		});

	let ui_window = window_weak.clone();
	let ui_settings = settings.clone();
	let ui_pending_confirmation = pending_confirmation.clone();
	let ui_database = database.clone();
	let ui_selected_game = selected_game.clone();
	window.global::<ModActions>().on_request_uninstall_all(
		move |confirmation_text, confirm_label| {
			let Some((_, game)) = get_selected_game(&ui_selected_game) else {
				return;
			};

			let Some(window) = ui_window.upgrade() else {
				return;
			};

			confirmation::request_uninstall_all(
				&window,
				&ui_settings,
				&ui_pending_confirmation,
				ui_database.clone(),
				game,
				confirmation_text.to_string(),
				confirm_label.to_string(),
			);
		},
	);

	let ui_window = window_weak.clone();
	let ui_database = database.clone();
	let ui_selected_game = selected_game.clone();
	window
		.global::<crate::GamesActions>()
		.on_refresh_game(move || {
			let Some((index, game)) = get_selected_game(&ui_selected_game) else {
				return;
			};

			let action_id = "refresh-game".to_string();
			action_state::start(&ui_window, action_id.clone());

			let database = ui_database.clone();
			let window = ui_window.clone();
			let ui_selected_game = ui_selected_game.clone();

			thread::spawn(move || {
				let mut game = game;

				let result = (|| {
					game.refresh_executable()?;
					database.insert_game(&game);
					database.refresh_installed_mods()?;
					database.get_game(&game.provider_id, &game.game_id)
				})();

				let game = match result {
					Ok(game) => game,
					Err(err) => {
						log::error!("Failed to refresh game: {err}");
						action_state::finish(
							&window,
							&action_id,
							Err(err),
							"Failed to refresh game",
						);
						return;
					}
				};

				let finish_window = window.clone();
				crate::runtime::invoke_on_ui(move || {
					let Some(window) = window.upgrade() else {
						return;
					};

					if let Some(mut selected_game) = ui_selected_game
						.lock()
						.ok_or_log("Failed to lock selected game")
					{
						*selected_game = Some((index, game.clone()));
					}

					let rows = window.global::<crate::GamesState>().get_rows();
					let mut row = rows.row_data(index).unwrap_or_default();
					model::apply_game_to_row(&mut row, &game);
					rows.set_row_data(index, row.clone());
					selected::show(&window, &game, row);
					mods::refresh_mods(&window, &database, &game);
				});

				action_state::finish(
					&finish_window,
					&action_id,
					Ok::<(), rai_pal_core::result::Error>(()),
					"",
				);
			});
		});

	let ui_window = window.as_weak();
	let ui_database = database.clone();
	let ui_scheduler = scheduler.clone();
	let ui_settings = settings.clone();
	let window_settings = settings.clone();
	window.window().on_winit_window_event(move |window, event| {
		match event {
			winit::event::WindowEvent::Resized(_) | winit::event::WindowEvent::Moved(_) => {
				update_window_state(window, &mut window_settings.borrow_mut());
			}
			winit::event::WindowEvent::CloseRequested | winit::event::WindowEvent::Destroyed => {
				update_window_state(window, &mut window_settings.borrow_mut());
				persist_settings(&window_settings);
			}
			winit::event::WindowEvent::HoveredFile(_) => {
				if let Some(window) = ui_window.upgrade() {
					window.global::<crate::AppState>().set_drop_highlight(true);
				}
			}
			winit::event::WindowEvent::HoveredFileCancelled => {
				if let Some(window) = ui_window.upgrade() {
					window.global::<crate::AppState>().set_drop_highlight(false);
				}
			}
			winit::event::WindowEvent::DroppedFile(path) => {
				if let Some(window) = ui_window.upgrade() {
					window.global::<crate::AppState>().set_drop_highlight(false);
					manual::add_dropped_paths(
						&window,
						ui_database.clone(),
						ui_scheduler.clone(),
						ui_settings.borrow().games_query.clone(),
						vec![path.clone()],
					);
				}
			}
			_ => {}
		}

		EventResult::Propagate
	});

	let model = Rc::new(slint::VecModel::from(rows));
	window.global::<crate::GamesState>().set_rows(model.into());
	begin_refresh_all(&window, database, scheduler, &settings, &selected_game);
	window.run()
}

fn get_selected_game(state: &selected::SelectedGameState) -> Option<(usize, Game)> {
	state
		.lock()
		.ok_or_log("Failed to lock selected game")
		.and_then(|selected_game| selected_game.clone())
}

fn apply_window_state(window: &MainWindow, state: Option<WindowState>) {
	let Some(state) = state else {
		return;
	};

	// Gamescope (Steam Deck game mode) scales windows larger than its output for display,
	// but pointer input stays in unscaled window coordinates, leaving the bottom/right part
	// of the window unreachable. Never restore a size that doesn't fit the current monitor.
	let monitor_size = window
		.window()
		.with_winit_window(|winit_window| {
			winit_window
				.current_monitor()
				.or_else(|| winit_window.primary_monitor())
				.map(|monitor| monitor.size())
		})
		.flatten();

	let (width, height) = match monitor_size {
		Some(monitor_size) => (
			state.width.min(monitor_size.width),
			state.height.min(monitor_size.height),
		),
		None => (state.width, state.height),
	};

	if (width, height) != (state.width, state.height) {
		log::warn!(
			"Clamping restored window size {}x{} to {}x{}",
			state.width,
			state.height,
			width,
			height
		);
	}

	if width > 0 && height > 0 {
		window
			.window()
			.set_size(slint::PhysicalSize::new(width, height));
	}

	let position = match monitor_size {
		Some(monitor_size) => slint::PhysicalPosition::new(
			state.x.clamp(
				0,
				i32::try_from(monitor_size.width.saturating_sub(width)).unwrap_or(i32::MAX),
			),
			state.y.clamp(
				0,
				i32::try_from(monitor_size.height.saturating_sub(height)).unwrap_or(i32::MAX),
			),
		),
		None => slint::PhysicalPosition::new(state.x, state.y),
	};

	window.window().set_position(position);

	if state.maximized {
		window.window().set_maximized(true);
	}
}

fn update_window_state(window: &slint::Window, settings: &mut AppSettings) {
	let maximized = window.is_maximized();
	let position = window.position();
	let size = window.size();

	let state = settings.window_state.get_or_insert(WindowState {
		x: position.x,
		y: position.y,
		width: size.width,
		height: size.height,
		maximized,
	});

	state.maximized = maximized;
	if !maximized && size.width > 0 && size.height > 0 {
		state.x = position.x;
		state.y = position.y;
		state.width = size.width;
		state.height = size.height;
	}
}

pub(crate) fn persist_settings(settings: &Rc<RefCell<AppSettings>>) {
	if let Err(err) = settings.borrow().try_write() {
		log::error!("Failed to save app settings: {err}");
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
		window
			.global::<crate::GamesState>()
			.set_filters_have_changes(groups.iter().any(|group| group.has_changes));
		window
			.global::<crate::GamesState>()
			.set_filters_have_expanded(groups.iter().any(|group| group.expanded));
		window
			.global::<crate::GamesState>()
			.set_filter_groups(Rc::new(slint::VecModel::from(groups)).into());
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

fn app_color_scheme(color_scheme: ColorScheme) -> AppColorScheme {
	match color_scheme {
		ColorScheme::Dark => AppColorScheme::Dark,
		ColorScheme::Light => AppColorScheme::Light,
		ColorScheme::Auto => AppColorScheme::Auto,
	}
}

const SORT_OPTIONS: [(GamesSortBy, bool); 6] = [
	(GamesSortBy::Title, false),
	(GamesSortBy::Title, true),
	(GamesSortBy::Engine, false),
	(GamesSortBy::Engine, true),
	(GamesSortBy::ReleaseDate, false),
	(GamesSortBy::ReleaseDate, true),
];

fn sort_option_index(sort_by: GamesSortBy, sort_descending: bool) -> i32 {
	SORT_OPTIONS
		.iter()
		.position(|option| *option == (sort_by, sort_descending))
		.and_then(|index| i32::try_from(index).ok_or_log("Sort option index does not fit in i32"))
		.unwrap_or_default()
}

fn sort_option_from_index(index: i32) -> (GamesSortBy, bool) {
	usize::try_from(index)
		.ok_or_log("Sort option index is negative")
		.and_then(|index| SORT_OPTIONS.get(index).copied())
		.unwrap_or((GamesSortBy::Title, false))
}

fn app_tab(tab: TabId) -> AppTab {
	match tab {
		TabId::Games => AppTab::Games,
		TabId::Mods => AppTab::Mods,
		TabId::Thanks => AppTab::Thanks,
	}
}

fn core_tab(tab: AppTab) -> TabId {
	match tab {
		AppTab::Games => TabId::Games,
		AppTab::Mods => TabId::Mods,
		AppTab::Thanks => TabId::Thanks,
	}
}

fn begin_refresh_all(
	window: &MainWindow,
	database: Arc<DbMutex>,
	scheduler: Arc<FetchScheduler>,
	settings: &Rc<RefCell<AppSettings>>,
	selected_game: &selected::SelectedGameState,
) {
	if window.global::<crate::Refresh>().get_refreshing() {
		return;
	}

	window.global::<crate::Refresh>().set_refreshing(true);
	window.global::<crate::Refresh>().set_task_names("".into());

	let query = settings.borrow().games_query.clone();
	refresh::refresh_all(
		window.as_weak(),
		database,
		scheduler,
		query,
		selected_game.clone(),
	);
}

fn monospace_font_family() -> &'static str {
	if cfg!(target_os = "windows") {
		return "Consolas";
	}

	if cfg!(target_os = "macos") {
		return "Menlo";
	}

	"DejaVu Sans Mono"
}

/// Gamescope's Xwayland reports bogus output dimensions and no `Xft.dpi`, so winit's X11
/// DPI heuristic derives a ~2.17x scale factor from them. Gamescope already scales the
/// window to the display, so the UI must be rendered at 1x there.
fn fix_gamescope_scale_factor() {
	let in_gamescope = std::env::var_os("GAMESCOPE_WAYLAND_DISPLAY").is_some()
		|| std::env::var("XDG_SESSION_DESKTOP").is_ok_and(|value| value == "gamescope");

	if !in_gamescope
		|| std::env::var_os("SLINT_SCALE_FACTOR").is_some()
		|| std::env::var_os("WINIT_X11_SCALE_FACTOR").is_some()
	{
		return;
	}

	// SAFETY: called before the Slint backend is initialized and before any threads are spawned.
	unsafe {
		std::env::set_var("SLINT_SCALE_FACTOR", "1");
	}
}

fn platform_name() -> String {
	#[cfg(target_os = "linux")]
	{
		if std::env::var("WAYLAND_DISPLAY").is_ok_and(|value| !value.is_empty()) {
			return "Linux (Wayland)".to_string();
		}

		if std::env::var("DISPLAY").is_ok_and(|value| !value.is_empty()) {
			return "Linux (X11)".to_string();
		}

		return "Linux".to_string();
	}

	#[cfg(target_os = "windows")]
	{
		"Windows".to_string()
	}

	#[cfg(not(any(target_os = "linux", target_os = "windows")))]
	{
		std::env::consts::OS.to_string()
	}
}
