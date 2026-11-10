use std::{cell::RefCell, rc::Rc, sync::Arc};

use rai_pal_backend::{
	app_settings::{AppSettings, ColorScheme, TabId, WindowState},
	game_providers::provider_command::ProviderCommandAction,
	games_query::FilterAction,
	local_database::{game_database::try_create, mod_database::ModDatabase},
	result::LogErrExt,
};
use slint::{
	ComponentHandle,
	winit_030::{EventResult, WinitWindowAccessor, winit},
};

use crate::{
	ActionKind, ActionStates, AppColorScheme, AppTab, Fonts, MainWindow, ModActions, action_state,
	analytics,
	commands::{CommandHandler, os_integration, query::load_game_ids, settings::sort_option_index},
	fetch::spawn_fetchers,
	filters::{build_groups, load_mod_families},
	i18n, logging,
	model::make_rows,
	notifications,
	scheduler::{enqueue_index, schedule_visible_range},
	single_instance,
};

pub fn run() -> Result<(), slint::PlatformError> {
	fix_gamescope_scale_factor();
	logging::init();

	let Some(_single_instance) = single_instance::acquire() else {
		return Ok(());
	};

	analytics::send(
		rai_pal_backend::analytics::Event::StartApp,
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

	let app_database = match try_create() {
		Ok(database) => database,
		Err(err) => {
			log::error!("Failed to open local database: {err}");
			return Ok(());
		}
	};

	if let Err(err) = app_database.setup_mod_tables() {
		log::error!("Failed to set up mod tables: {err}");
	}

	let app_settings = Rc::new(RefCell::new(AppSettings::read()));
	let detected_locale = i18n::detect_locale();
	let initial_locale = i18n::effective_locale(&app_settings.borrow(), detected_locale);
	let mod_families = load_mod_families(&app_database);

	let initial_query = app_settings.borrow().games_query.clone();
	let initial_tab = app_tab(app_settings.borrow().selected_tab);
	let initial_groups = build_groups(
		&initial_query.filter,
		&mod_families,
		&app_settings.borrow().expanded_filters,
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
	apply_window_state(&main_window, app_settings.borrow().window_state);
	main_window
		.global::<ActionStates>()
		.on_lookup(|key| action_state::lookup(&key));
	i18n::apply_translation(initial_locale);
	main_window
		.global::<crate::AppSettings>()
		.set_selected_language(
			app_settings
				.borrow()
				.override_language
				.map(i18n::locale_code)
				.unwrap_or_default()
				.into(),
		);
	main_window
		.global::<crate::AppSettings>()
		.set_detected_language_name(
			detected_locale
				.map(i18n::native_name)
				.unwrap_or_default()
				.into(),
		);
	main_window
		.global::<crate::AppSettings>()
		.set_selected_color_scheme(app_color_scheme(app_settings.borrow().color_scheme));
	main_window
		.global::<crate::AppSettings>()
		.set_hide_game_thumbnails(app_settings.borrow().hide_game_thumbnails);
	main_window
		.global::<crate::AppInfo>()
		.set_version(env!("CARGO_PKG_VERSION").into());
	main_window
		.global::<crate::GamesQuery>()
		.set_search_text(initial_query.search.clone().into());
	main_window.global::<crate::GamesList>().set_total_count(
		i32::try_from(games_total_count).unwrap_or_else(|err| {
			log::warn!("Games count exceeds i32: {err}");
			i32::MAX
		}),
	);
	main_window
		.global::<crate::GamesQuery>()
		.set_sort_index(sort_option_index(
			initial_query.sort_by,
			initial_query.sort_descending,
		));
	main_window.set_current_tab(initial_tab);
	main_window
		.global::<crate::GamesQuery>()
		.set_filters_have_expanded(
			initial_groups
				.iter()
				.any(|filter_group| filter_group.expanded),
		);
	main_window
		.global::<crate::GamesQuery>()
		.set_filter_groups(Rc::new(slint::VecModel::from(initial_groups)).into());
	main_window
		.global::<crate::GamesQuery>()
		.set_filters_have_changes(initial_have_changes);

	let renderer_name = match main_window.window().set_rendering_notifier(|_, _| {}) {
		Ok(()) => "FemtoVG (OpenGL)",
		Err(slint::SetRenderingNotifierError::Unsupported) => "Software",
		Err(err) => {
			log::warn!("Failed to determine renderer name: {err}");
			"Unknown"
		}
	};
	main_window
		.global::<crate::AppInfo>()
		.set_renderer(renderer_name.into());
	main_window
		.global::<crate::AppInfo>()
		.set_platform(platform_name().into());
	main_window
		.global::<Fonts>()
		.set_monospace_family(monospace_font_family().into());

	let window_weak = main_window.as_weak();
	let scheduler = spawn_fetchers(&app_database, &window_weak);
	scheduler.set_thumbnails_enabled(!app_settings.borrow().hide_game_thumbnails);
	scheduler.begin_query();

	{
		let ui_window = window_weak.clone();
		let ui_scheduler = Arc::clone(&scheduler);
		main_window
			.global::<crate::GamesActions>()
			.on_request_game(move |index| {
				let Some(window) = ui_window.upgrade() else {
					return;
				};

				let rows = window.global::<crate::GamesList>().get_rows();
				enqueue_index(
					&ui_scheduler,
					&rows,
					usize::try_from(index.max(0)).unwrap_or_default(),
				);
			});
	}

	{
		let ui_window = window_weak.clone();
		let ui_scheduler = Arc::clone(&scheduler);
		main_window
			.global::<crate::GamesActions>()
			.on_visible_range_changed(move |first, last| {
				if last < 0 {
					return;
				}

				let Some(window) = ui_window.upgrade() else {
					return;
				};

				let rows = window.global::<crate::GamesList>().get_rows();
				schedule_visible_range(
					&ui_scheduler,
					&rows,
					usize::try_from(first.max(0)).unwrap_or_default(),
					usize::try_from(last).unwrap_or_default(),
				);
			});
	}

	let ctx = CommandHandler::create(
		window_weak.clone(),
		Arc::clone(&app_database),
		Rc::clone(&app_settings),
		Arc::clone(&scheduler),
		mod_families,
	);
	ctx.load_mods_tab();
	ctx.load_user();
	if initial_tab == AppTab::Thanks {
		ctx.load_patrons();
	}

	{
		let ctx = Rc::clone(&ctx);
		main_window
			.global::<crate::Refresh>()
			.on_refresh_all(move || ctx.refresh_all());
	}

	{
		let ctx = Rc::clone(&ctx);
		main_window
			.global::<crate::DialogActions>()
			.on_request_url_mod_sources(move || ctx.show_mod_sources());
	}

	{
		let ctx = Rc::clone(&ctx);
		main_window
			.global::<crate::DialogActions>()
			.on_set_url_mod_source_enabled(move |url, enabled| {
				ctx.set_mod_source_enabled(url.to_string(), enabled);
			});
	}

	{
		let ctx = Rc::clone(&ctx);
		main_window
			.global::<crate::DialogActions>()
			.on_remove_url_mod_source(move |url| {
				ctx.remove_mod_source(url.to_string());
			});
	}

	{
		let ctx = Rc::clone(&ctx);
		main_window
			.global::<crate::DialogActions>()
			.on_add_url_mod_source(move |url| {
				ctx.request_add_mod_source(url.as_str());
			});
	}

	{
		let ctx = Rc::clone(&ctx);
		main_window
			.global::<crate::DialogActions>()
			.on_confirm_url_mod_source(move || ctx.confirm_add_mod_source());
	}

	{
		let ui_window = window_weak;
		main_window
			.global::<crate::DialogActions>()
			.on_cancel_url_mod_source(move || {
				if let Some(window) = ui_window.upgrade() {
					window
						.global::<crate::ConfirmModSourceState>()
						.set_state(crate::ActionState::Success);
					window
						.global::<crate::ConfirmModSourceState>()
						.set_error("".into());
					window
						.global::<crate::ConfirmModSourceState>()
						.set_mods(slint::ModelRc::default());
					window
						.global::<crate::ConfirmModSourceState>()
						.set_state(crate::ActionState::Idle);
				}
			});
	}

	{
		let ctx = Rc::clone(&ctx);
		main_window
			.global::<crate::DialogActions>()
			.on_reset_steam_cache(move || {
				if let Some(window) = ctx.ui()
					&& window.global::<crate::SteamCacheState>().get_state()
						!= crate::ActionState::Loading
				{
					ctx.reset_steam_cache();
				}
			});
	}

	{
		let ctx = Rc::clone(&ctx);
		main_window
			.global::<crate::DialogActions>()
			.on_add_steam_shortcut(move || {
				if let Some(window) = ctx.ui()
					&& window.global::<crate::SteamShortcutState>().get_state()
						!= crate::ActionState::Loading
				{
					ctx.add_steam_shortcut();
				}
			});
	}

	{
		let ctx = Rc::clone(&ctx);
		main_window
			.global::<crate::AppActions>()
			.on_open_logs_folder(move || {
				ctx.open_logs_folder(action_state::key(ActionKind::OpenLogsFolder, ""));
			});
	}

	{
		let ctx = Rc::clone(&ctx);
		main_window
			.global::<crate::AppActions>()
			.on_reset_settings(move || ctx.reset_settings(detected_locale));
	}

	{
		let ctx = Rc::clone(&ctx);
		main_window
			.global::<crate::DialogActions>()
			.on_request_add_game(move || {
				if let Some(window) = ctx.ui() {
					window.set_current_tab(AppTab::Games);
					window
						.global::<crate::AppActions>()
						.invoke_tab_changed(AppTab::Games);
				}

				ctx.show_add_game();
			});
	}

	{
		let ctx = Rc::clone(&ctx);
		main_window
			.global::<crate::DialogActions>()
			.on_add_game_closed(move || ctx.cancel_game_scan());
	}

	{
		let ctx = Rc::clone(&ctx);
		main_window
			.global::<crate::DialogActions>()
			.on_select_game_executable(move || ctx.select_executable());
	}

	{
		let ctx = Rc::clone(&ctx);
		main_window
			.global::<crate::DialogActions>()
			.on_select_game_directory(move || ctx.select_directory());
	}

	{
		let ctx = Rc::clone(&ctx);
		main_window
			.global::<crate::DialogActions>()
			.on_cancel_game_scan(move || ctx.cancel_game_scan());
	}

	{
		let ctx = Rc::clone(&ctx);
		main_window
			.global::<crate::DialogActions>()
			.on_confirm_game_directory(move || ctx.confirm_add_directory());
	}

	{
		let ctx = Rc::clone(&ctx);
		main_window
			.global::<crate::DialogActions>()
			.on_remove_game_directory(move |path| {
				ctx.remove_directory(path.to_string());
			});
	}

	{
		let ctx = Rc::clone(&ctx);
		main_window.global::<crate::GamesActions>().on_remove_game(
			move |confirmation_text, confirm_label| {
				let Some((_, game)) = ctx.selected_game() else {
					return;
				};

				ctx.request_remove_game(
					game,
					confirmation_text.to_string(),
					confirm_label.to_string(),
				);
			},
		);
	}

	{
		let ctx = Rc::clone(&ctx);
		main_window
			.global::<crate::AppActions>()
			.on_log_in(move || ctx.log_in());
	}

	{
		let ctx = Rc::clone(&ctx);
		main_window
			.global::<crate::AppActions>()
			.on_log_out(move || ctx.log_out());
	}

	{
		let ui_window = main_window.as_weak();
		main_window
			.global::<crate::AppActions>()
			.on_dismiss_notification(move |id| {
				if let Some(window) = ui_window.upgrade() {
					notifications::dismiss(&window, id);
				}
			});
	}

	{
		let ctx = Rc::clone(&ctx);
		main_window
			.global::<crate::GamesActions>()
			.on_request_debug_data(move || {
				let Some(window) = ctx.ui() else {
					return;
				};
				let Some((_, game)) = ctx.selected_game() else {
					return;
				};

				match serde_json::to_string_pretty(&game) {
					Ok(json) => window
						.global::<crate::DebugState>()
						.set_details_json(json.into()),
					Err(err) => {
						notifications::error(&window, format!("Failed to serialize game: {err}"));
					}
				}
			});
	}

	{
		let ui_window = main_window.as_weak();
		let ui_database = Arc::clone(&app_database);
		main_window
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
							notifications::error(
								&window,
								format!("Failed to serialize mod: {err}"),
							);
						}
					},
					Err(err) => notifications::error(&window, format!("Failed to load mod: {err}")),
				}
			});
	}

	{
		let ctx = Rc::clone(&ctx);
		main_window
			.global::<crate::DialogActions>()
			.on_confirmation_response(move |confirmed| {
				ctx.respond_to_confirmation(confirmed);
			});
	}

	{
		let ctx = Rc::clone(&ctx);
		main_window
			.global::<crate::GamesActions>()
			.on_search_changed(move |text| ctx.set_search_text(text.as_str()));
	}

	{
		let ctx = Rc::clone(&ctx);
		main_window
			.global::<crate::GamesActions>()
			.on_sort_changed(move |index| ctx.set_sort_index(index));
	}

	{
		let ctx = Rc::clone(&ctx);
		main_window
			.global::<crate::GamesActions>()
			.on_filter_toggled(move |group, value, checked| {
				ctx.handle_filter_action(
					group.as_str(),
					value.as_str(),
					FilterAction::Toggle(checked),
				);
			});
	}

	{
		let ctx = Rc::clone(&ctx);
		main_window
			.global::<crate::GamesActions>()
			.on_filter_lock_toggled(move |group, value| {
				ctx.handle_filter_action(group.as_str(), value.as_str(), FilterAction::Lock);
			});
	}

	{
		let ctx = Rc::clone(&ctx);
		main_window
			.global::<crate::GamesActions>()
			.on_filter_only(move |group, value| {
				ctx.handle_filter_action(group.as_str(), value.as_str(), FilterAction::Only);
			});
	}

	{
		let ctx = Rc::clone(&ctx);
		main_window
			.global::<crate::GamesActions>()
			.on_filter_reset(move |group| {
				ctx.reset_filter(group.as_str());
			});
	}

	{
		let ctx = Rc::clone(&ctx);
		main_window
			.global::<crate::GamesActions>()
			.on_filters_reset(move || ctx.reset_filters());
	}

	{
		let ctx = Rc::clone(&ctx);
		main_window
			.global::<crate::GamesActions>()
			.on_filters_collapse_all(move || ctx.collapse_filters());
	}

	{
		let ctx = Rc::clone(&ctx);
		main_window
			.global::<crate::GamesActions>()
			.on_filter_expanded_changed(move |group, expanded| {
				ctx.set_filter_expanded(group.as_str(), expanded);
			});
	}

	{
		let ctx = Rc::clone(&ctx);
		main_window
			.global::<crate::AppActions>()
			.on_language_changed(move |value| {
				ctx.set_language(value.as_str(), detected_locale);
			});
	}

	{
		let ctx = Rc::clone(&ctx);
		main_window
			.global::<crate::AppActions>()
			.on_color_scheme_changed(move |value| ctx.set_color_scheme(value));
	}

	{
		let ctx = Rc::clone(&ctx);
		main_window
			.global::<crate::AppActions>()
			.on_hide_game_thumbnails_changed(move |hide| {
				ctx.set_hide_game_thumbnails(hide);
			});
	}

	{
		let ctx = Rc::clone(&ctx);
		main_window
			.global::<crate::AppActions>()
			.on_tab_changed(move |tab| ctx.set_tab(tab));
	}

	{
		let ctx = Rc::clone(&ctx);
		main_window
			.global::<crate::GamesActions>()
			.on_select_game(move |index| ctx.select_game(index));
	}

	{
		let ctx = Rc::clone(&ctx);
		main_window
			.global::<crate::GamesActions>()
			.on_back(move || ctx.deselect_game());
	}

	{
		let ctx = Rc::clone(&ctx);
		main_window
			.global::<crate::GamesActions>()
			.on_run_provider_action(move |value| {
				let Some(action) = value
					.parse::<ProviderCommandAction>()
					.ok_or_log("Failed to parse provider action")
				else {
					return;
				};

				let Some((_, game)) = ctx.selected_game() else {
					return;
				};

				analytics::send(
					rai_pal_backend::analytics::Event::ProviderCommand,
					&[
						("action", action.to_string()),
						("game", game.display_title.clone()),
					],
				);

				if let Some(window) = ctx.ui() {
					notifications::info(
						&window,
						"Running command... This might take a few seconds.",
					);
				}

				ctx.run_provider_action(
					game,
					action,
					action_state::key(ActionKind::ProviderAction, value),
				);
			});
	}

	{
		let ctx = Rc::clone(&ctx);
		main_window
			.global::<crate::GamesActions>()
			.on_open_game_folder(move || {
				if let Some((_, game)) = ctx.selected_game() {
					ctx.open_game_folder(action_state::key(ActionKind::OpenGameFolder, ""), game);
				}
			});
	}

	{
		let ctx = Rc::clone(&ctx);
		main_window
			.global::<crate::GamesActions>()
			.on_open_game_mods_folder(move || {
				if let Some((_, game)) = ctx.selected_game() {
					ctx.open_game_mods_folder(
						action_state::key(ActionKind::OpenGameModsFolder, ""),
						game,
					);
				}
			});
	}

	{
		let ctx = Rc::clone(&ctx);
		main_window
			.global::<crate::GamesActions>()
			.on_open_game_data_folder(move || {
				if let Some((_, game)) = ctx.selected_game() {
					ctx.open_game_data_folder(
						action_state::key(ActionKind::OpenGameDataFolder, ""),
						game,
					);
				}
			});
	}

	{
		let ctx = Rc::clone(&ctx);
		main_window
			.global::<crate::GamesActions>()
			.on_open_wine_prefix_folder(move || {
				if let Some((_, game)) = ctx.selected_game() {
					ctx.open_wine_prefix_folder(
						action_state::key(ActionKind::OpenWinePrefixFolder, ""),
						game,
					);
				}
			});
	}

	{
		let ctx = Rc::clone(&ctx);
		main_window
			.global::<crate::GamesActions>()
			.on_open_wine_binary_folder(move || {
				if let Some((_, game)) = ctx.selected_game() {
					ctx.open_wine_binary_folder(
						action_state::key(ActionKind::OpenWineBinaryFolder, ""),
						game,
					);
				}
			});
	}

	{
		let ctx = Rc::clone(&ctx);
		main_window
			.global::<crate::AppActions>()
			.on_install_update(move || ctx.install_update());
	}

	{
		let ctx = Rc::clone(&ctx);
		main_window
			.global::<crate::AppActions>()
			.on_ignore_update(move || ctx.ignore_update());
	}

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

	{
		let ctx = Rc::clone(&ctx);
		main_window.global::<ModActions>().on_request_install(
			move |mod_id, confirmation_text, confirm_label, skip_id| {
				let Some((_, game)) = ctx.selected_game() else {
					return;
				};

				ctx.request_install(
					game,
					mod_id.to_string(),
					confirmation_text.to_string(),
					confirm_label.to_string(),
					skip_id.to_string(),
				);
			},
		);
	}

	{
		let ctx = Rc::clone(&ctx);
		main_window.global::<ModActions>().on_update(move |mod_id| {
			let Some((_, game)) = ctx.selected_game() else {
				return;
			};

			analytics::send_mod(
				rai_pal_backend::analytics::Event::UpdateMod,
				mod_id.as_str(),
				Some(&game),
			);
			ctx.install_mod(game, mod_id.to_string());
		});
	}

	{
		let ctx = Rc::clone(&ctx);
		main_window
			.global::<ModActions>()
			.on_reinstall(move |mod_id| {
				let Some((_, game)) = ctx.selected_game() else {
					return;
				};

				analytics::send_mod(
					rai_pal_backend::analytics::Event::InstallMod,
					mod_id.as_str(),
					Some(&game),
				);
				ctx.install_mod(game, mod_id.to_string());
			});
	}

	{
		let ctx = Rc::clone(&ctx);
		main_window
			.global::<ModActions>()
			.on_uninstall(move |mod_id| {
				let Some((_, game)) = ctx.selected_game() else {
					return;
				};

				analytics::send_mod(
					rai_pal_backend::analytics::Event::UninstallMod,
					mod_id.as_str(),
					Some(&game),
				);
				ctx.uninstall_mod(game, mod_id.to_string());
			});
	}

	{
		let ctx = Rc::clone(&ctx);
		main_window.global::<ModActions>().on_run(move |mod_id| {
			let Some((_, game)) = ctx.selected_game() else {
				return;
			};

			analytics::send_mod(
				rai_pal_backend::analytics::Event::RunMod,
				mod_id.as_str(),
				Some(&game),
			);
			ctx.run_mod(game, mod_id.to_string());
		});
	}

	{
		let ctx = Rc::clone(&ctx);
		main_window
			.global::<ModActions>()
			.on_run_standalone(move |mod_id| ctx.run_standalone_mod(mod_id.to_string()));
	}

	{
		let ctx = Rc::clone(&ctx);
		main_window
			.global::<ModActions>()
			.on_install_standalone(move |mod_id| {
				ctx.install_standalone_mod(mod_id.to_string());
			});
	}

	{
		let ctx = Rc::clone(&ctx);
		main_window
			.global::<crate::AppActions>()
			.on_clear_downloads(move || ctx.downloads.clear_completed());
	}

	main_window.global::<ModActions>().on_open_url(move |url| {
		crate::commands::actions::open_url(url.to_string());
	});

	{
		let ctx = Rc::clone(&ctx);
		main_window
			.global::<ModActions>()
			.on_open_local_mods_folder(move || {
				ctx.open_local_mods_folder(action_state::key(ActionKind::OpenLocalModsFolder, ""));
			});
	}

	{
		let ctx = Rc::clone(&ctx);
		main_window
			.global::<ModActions>()
			.on_open_folder(move |mod_id| {
				if let Some((_, game)) = ctx.selected_game() {
					ctx.open_mod_folder(game, mod_id.to_string());
				}
			});
	}

	{
		let ctx = Rc::clone(&ctx);
		main_window
			.global::<ModActions>()
			.on_download_config(move |mod_id| {
				if let Some((_, game)) = ctx.selected_game() {
					ctx.download_mod_config(game, mod_id.to_string());
				}
			});
	}

	{
		let ctx = Rc::clone(&ctx);
		main_window
			.global::<ModActions>()
			.on_configure(move |mod_id| {
				if let Some((_, game)) = ctx.selected_game() {
					ctx.configure_mod(game, mod_id.to_string(), false);
				}
			});
	}

	{
		let ctx = Rc::clone(&ctx);
		main_window
			.global::<ModActions>()
			.on_configure_folder(move |mod_id| {
				if let Some((_, game)) = ctx.selected_game() {
					ctx.configure_mod(game, mod_id.to_string(), true);
				}
			});
	}

	{
		let ctx = Rc::clone(&ctx);
		main_window.global::<ModActions>().on_request_uninstall_all(
			move |confirmation_text, confirm_label| {
				let Some((_, game)) = ctx.selected_game() else {
					return;
				};

				ctx.request_uninstall_all(
					game,
					confirmation_text.to_string(),
					confirm_label.to_string(),
				);
			},
		);
	}

	{
		let ctx = Rc::clone(&ctx);
		main_window
			.global::<crate::GamesActions>()
			.on_refresh_game(move || ctx.refresh_game());
	}

	{
		let ctx = Rc::clone(&ctx);
		main_window
			.window()
			.on_winit_window_event(move |window, event| {
				match event {
					winit::event::WindowEvent::Resized(_) | winit::event::WindowEvent::Moved(_) => {
						update_window_state(window, &mut ctx.settings.borrow_mut());
					}
					winit::event::WindowEvent::CloseRequested
					| winit::event::WindowEvent::Destroyed => {
						update_window_state(window, &mut ctx.settings.borrow_mut());
						ctx.persist_settings();
					}
					winit::event::WindowEvent::HoveredFile(_) => {
						if let Some(app_window) = ctx.ui() {
							app_window.set_drop_highlight(true);
						}
					}
					winit::event::WindowEvent::HoveredFileCancelled => {
						if let Some(app_window) = ctx.ui() {
							app_window.set_drop_highlight(false);
						}
					}
					winit::event::WindowEvent::DroppedFile(path) => {
						if let Some(app_window) = ctx.ui() {
							app_window.set_drop_highlight(false);
							ctx.add_dropped_paths(vec![path.clone()]);
						}
					}
					_ => {}
				}

				EventResult::Propagate
			});
	}

	let model = Rc::new(slint::VecModel::from(initial_rows));
	main_window
		.global::<crate::GamesList>()
		.set_rows(model.into());
	ctx.refresh_all();
	main_window.run()
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

	let (width, height) = monitor_size.map_or((state.width, state.height), |monitor_size| {
		(
			state.width.min(monitor_size.width),
			state.height.min(monitor_size.height),
		)
	});

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

	let position = monitor_size.map_or_else(
		|| slint::PhysicalPosition::new(state.x, state.y),
		|monitor_size| {
			slint::PhysicalPosition::new(
				state.x.clamp(
					0,
					i32::try_from(monitor_size.width.saturating_sub(width)).unwrap_or(i32::MAX),
				),
				state.y.clamp(
					0,
					i32::try_from(monitor_size.height.saturating_sub(height)).unwrap_or(i32::MAX),
				),
			)
		},
	);

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

const fn app_color_scheme(color_scheme: ColorScheme) -> AppColorScheme {
	match color_scheme {
		ColorScheme::Dark => AppColorScheme::Dark,
		ColorScheme::Light => AppColorScheme::Light,
		ColorScheme::Auto => AppColorScheme::Auto,
	}
}

const fn app_tab(tab: TabId) -> AppTab {
	match tab {
		TabId::Games => AppTab::Games,
		TabId::Mods => AppTab::Mods,
		TabId::Thanks => AppTab::Thanks,
	}
}

const fn monospace_font_family() -> &'static str {
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

		"Linux".to_string()
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
