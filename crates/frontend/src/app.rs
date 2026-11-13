use std::{cell::RefCell, rc::Rc, sync::Arc};

use rai_pal_backend::{
	app_settings::{AppLocale, AppSettings},
	app_variant::AppVariant,
	local_database::game_database::try_create,
	operating_system::Platform,
};
use slint::{ComponentHandle, winit_030::WinitWindowAccessor};

use crate::{
	AppTab, MainWindow, action_state, analytics,
	commands::{CommandHandler, os_integration, query::load_game_ids, settings::sort_option_index},
	fetch::spawn_fetchers,
	filters::{build_groups, load_mod_families, load_providers_with_games},
	i18n, logging,
	model::make_rows,
	single_instance, window_state,
};

pub fn run() -> Result<(), slint::PlatformError> {
	window_state::fix_gamescope_scale_factor();
	logging::init();

	if !single_instance::acquire() {
		return Ok(());
	}

	analytics::send(
		rai_pal_backend::analytics::Event::StartApp,
		[
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

	let app_database = match try_create() {
		Ok(database) => database,
		Err(err) => {
			log::error!("Failed to open local database: {err}");
			return Ok(());
		}
	};

	let app_settings = Rc::new(RefCell::new(AppSettings::read()));
	let detected_locale = i18n::detect_locale();
	let initial_locale = i18n::effective_locale(&app_settings.borrow(), detected_locale);
	let mod_families = load_mod_families(&app_database);
	let providers_with_games = load_providers_with_games(&app_database);

	let initial_query = app_settings.borrow().games_query.clone();
	let initial_tab = app_settings.borrow().selected_tab.into();
	let initial_groups = build_groups(
		&initial_query.filter,
		&mod_families,
		&app_settings.borrow().collapsed_filters,
		&providers_with_games,
	);

	let initial_have_changes = initial_groups.iter().any(|group| group.has_changes);

	let response = load_game_ids(&app_database, initial_query.clone()).unwrap_or_else(|err| {
		log::error!("Failed to load initial games: {err}");
		rai_pal_backend::local_database::game_database::GameIdsResponse {
			game_ids: Vec::new(),
			total_count: 0,
		}
	});
	let initial_rows = make_rows(&response.game_ids);
	let games_total_count = response.total_count;
	let app_database = Arc::new(app_database);

	let main_window = MainWindow::new()?;

	// Must happen before the winit window is created (on first show), so that
	// Wayland/X11 can associate the window with the desktop entry and its icon.
	#[cfg(target_os = "linux")]
	if let Err(err) = slint::set_xdg_app_id("Rai-Pal") {
		log::warn!("Failed to set XDG app id: {err}");
	}

	main_window.set_window_title(if cfg!(debug_assertions) {
		"Rai Pal DEV".into()
	} else {
		"Rai Pal".into()
	});
	window_state::apply(&main_window, app_settings.borrow().window_state);
	main_window
		.global::<crate::ActionStates>()
		.on_lookup(|key| action_state::lookup(&key));
	apply_initial_state(
		&main_window,
		&app_settings,
		detected_locale,
		initial_locale,
		&initial_query,
		initial_tab,
		initial_groups,
		initial_have_changes,
		games_total_count,
	);

	let renderer = match main_window.window().set_rendering_notifier({
		let window_weak = main_window.as_weak();
		let mut reported = false;
		move |state, graphics_api| {
			if reported || !matches!(state, slint::RenderingState::RenderingSetup) {
				return;
			}
			reported = true;
			let renderer = if let slint::GraphicsAPI::NativeOpenGL { .. } = graphics_api {
				crate::Renderer::OpenGl
			} else {
				log::warn!("Unexpected graphics API: {graphics_api:?}");
				crate::Renderer::Unknown
			};
			if let Some(window) = window_weak.upgrade() {
				window.global::<crate::AppInfo>().set_renderer(renderer);
			}
		}
	}) {
		Ok(()) => crate::Renderer::Unknown,
		Err(slint::SetRenderingNotifierError::Unsupported) => crate::Renderer::Software,
		Err(err) => {
			log::warn!("Failed to determine renderer: {err}");
			crate::Renderer::Unknown
		}
	};
	main_window
		.global::<crate::AppInfo>()
		.set_renderer(renderer);
	main_window
		.global::<crate::AppInfo>()
		.set_variant(AppVariant::get_current().into());
	main_window
		.global::<crate::AppInfo>()
		.set_platform(Platform::get_current().into());

	let window_weak = main_window.as_weak();
	let scheduler = spawn_fetchers(&app_database, &window_weak);
	scheduler.set_thumbnails_enabled(!app_settings.borrow().hide_game_thumbnails);
	scheduler.begin_query();

	let rows = Rc::new(slint::VecModel::from(initial_rows));
	main_window
		.global::<crate::GamesList>()
		.set_rows(rows.into());

	let ctx = CommandHandler::create(
		window_weak,
		Arc::clone(&app_database),
		Rc::clone(&app_settings),
		Arc::clone(&scheduler),
		mod_families,
		detected_locale,
	);
	ctx.load_mods_tab();
	ctx.load_user();
	if initial_tab == AppTab::Thanks {
		ctx.load_patrons();
	}

	crate::commands::register(&main_window);
	window_state::register_events(&main_window);

	os_integration::register();
	for argument in std::env::args().skip(1) {
		ctx.handle_deep_link(&argument);
	}

	single_instance::listen(move |arguments| {
		crate::runtime::invoke_on_ui(move || {
			let Some(handler) = CommandHandler::current() else {
				return;
			};
			let Some(window) = handler.ui() else {
				return;
			};

			window.window().set_minimized(false);
			window.window().with_winit_window(|winit_window| {
				winit_window.focus_window();
			});

			for argument in arguments {
				handler.handle_deep_link(&argument);
			}
		});
	});

	ctx.refresh_all();
	main_window.run()
}

#[allow(clippy::too_many_arguments)]
fn apply_initial_state(
	window: &MainWindow,
	app_settings: &Rc<RefCell<AppSettings>>,
	detected_locale: Option<AppLocale>,
	initial_locale: AppLocale,
	initial_query: &rai_pal_backend::games_query::GamesQuery,
	initial_tab: AppTab,
	initial_groups: Vec<crate::FilterGroupData>,
	initial_have_changes: bool,
	games_total_count: u32,
) {
	i18n::apply_translation(initial_locale);
	let settings = window.global::<crate::AppSettings>();
	settings.set_selected_language(
		app_settings
			.borrow()
			.override_language
			.map(AppLocale::code)
			.unwrap_or_default()
			.into(),
	);
	settings.set_detected_language_name(
		detected_locale
			.map(AppLocale::native_name)
			.unwrap_or_default()
			.into(),
	);
	crate::languages::refresh(window, &app_settings.borrow(), detected_locale);
	settings.set_selected_color_scheme(app_settings.borrow().color_scheme.into());
	settings.set_hide_game_thumbnails(app_settings.borrow().hide_game_thumbnails);

	window
		.global::<crate::AppInfo>()
		.set_version(env!("CARGO_PKG_VERSION").into());

	let query = window.global::<crate::GamesQuery>();
	query.set_search_text(initial_query.search.clone().into());
	query.set_sort_index(sort_option_index(
		initial_query.sort_by,
		initial_query.sort_descending,
	));
	query.set_filters_have_expanded(
		initial_groups
			.iter()
			.any(|filter_group| filter_group.expanded),
	);
	query.set_filter_groups(Rc::new(slint::VecModel::from(initial_groups)).into());
	query.set_filters_have_changes(initial_have_changes);

	window.global::<crate::GamesList>().set_total_count(
		i32::try_from(games_total_count).unwrap_or_else(|err| {
			log::warn!("Games count exceeds i32: {err}");
			i32::MAX
		}),
	);
	window.set_current_tab(initial_tab);
}
