use std::{rc::Rc, sync::Arc, thread};

use rai_pal_core::{
	local_database::{app_database::DbMutex, game_database::GameDatabase},
	mod_providers::url_mod_provider,
};
use slint::ComponentHandle;

use crate::{MainWindow, UrlModSourceData, mods, mods_tab, refresh, selected::SelectedGameState};

pub fn show(window: &MainWindow) {
	window.set_url_mod_sources_error("".into());
	load_sources(window);
	window.set_url_mod_sources_open(true);
}

fn load_sources(window: &MainWindow) {
	let response = url_mod_provider::get_url_mod_sources();

	window.set_default_url_mod_source(UrlModSourceData {
		url: response.default_source.url.into(),
		enabled: response.default_source.enabled,
		is_default: true,
	});

	let sources = response
		.sources
		.into_iter()
		.map(|source| UrlModSourceData {
			url: source.url.into(),
			enabled: source.enabled,
			is_default: false,
		})
		.collect::<Vec<_>>();

	window.set_url_mod_sources(Rc::new(slint::VecModel::from(sources)).into());
}

pub fn request_add(window: &MainWindow, url: String) {
	let url = url.trim().to_string();

	if url.is_empty() {
		window.set_url_mod_sources_error("URL is empty".into());
		return;
	}

	window.set_confirm_mod_source_url(url.clone().into());
	window.set_confirm_mod_source_mods(
		Rc::new(slint::VecModel::from(Vec::<slint::SharedString>::new())).into(),
	);
	window.set_confirm_mod_source_loading(true);
	window.set_confirm_mod_source_error("".into());
	window.set_confirm_mod_source_saving(false);
	window.set_confirm_mod_source_open(true);

	let window = window.as_weak();

	thread::spawn(move || {
		let result = get_mods(&url);

		let _ = slint::invoke_from_event_loop(move || {
			let Some(window) = window.upgrade() else {
				return;
			};

			window.set_confirm_mod_source_loading(false);

			match result {
				Ok(mods) => {
					let titles = mods
						.into_iter()
						.map(|game_mod| slint::SharedString::from(game_mod.title))
						.collect::<Vec<_>>();
					window
						.set_confirm_mod_source_mods(Rc::new(slint::VecModel::from(titles)).into());
				}
				Err(err) => {
					window.set_confirm_mod_source_error(err.into());
				}
			}
		});
	});
}

pub fn confirm_add(window: &MainWindow, database: Arc<DbMutex>, selected_game: SelectedGameState) {
	let url = window.get_confirm_mod_source_url().to_string();

	if url.is_empty() || window.get_confirm_mod_source_saving() {
		return;
	}

	window.set_confirm_mod_source_saving(true);
	window.set_confirm_mod_source_error("".into());

	let window = window.as_weak();
	let database_for_refresh = database.clone();

	thread::spawn(move || {
		let result = url_mod_provider::add_url_mod_source(url.clone());

		if result.is_ok() {
			refresh::refresh_mods(&database_for_refresh);
		}

		let _ = slint::invoke_from_event_loop(move || {
			let Some(window) = window.upgrade() else {
				return;
			};

			window.set_confirm_mod_source_saving(false);

			if let Err(err) = result {
				window.set_confirm_mod_source_error(err.to_string().into());
				return;
			}

			window.set_confirm_mod_source_open(false);
			window.set_url_mod_sources_open(false);
			window.set_new_url_mod_source("".into());
			load_sources(&window);
			after_mod_refresh(&window, &database, &selected_game);
		});
	});
}

pub fn set_enabled(
	window: &MainWindow,
	database: Arc<DbMutex>,
	selected_game: SelectedGameState,
	url: String,
	enabled: bool,
) {
	if window.get_url_mod_sources_toggling() {
		return;
	}

	window.set_url_mod_sources_toggling(true);
	window.set_url_mod_sources_error("".into());

	let window_weak = window.as_weak();
	let database_for_refresh = database.clone();

	thread::spawn(move || {
		let result = url_mod_provider::set_url_mod_source_enabled(&url, enabled);

		if result.is_ok() {
			refresh::refresh_mods(&database_for_refresh);
		}

		let _ = slint::invoke_from_event_loop(move || {
			let Some(window) = window_weak.upgrade() else {
				return;
			};

			window.set_url_mod_sources_toggling(false);

			if let Err(err) = result {
				window.set_url_mod_sources_error(err.to_string().into());
			}

			load_sources(&window);
			after_mod_refresh(&window, &database, &selected_game);
		});
	});
}

pub fn remove(
	window: &MainWindow,
	database: Arc<DbMutex>,
	selected_game: SelectedGameState,
	url: String,
) {
	if window.get_url_mod_sources_toggling() {
		return;
	}

	window.set_url_mod_sources_toggling(true);
	window.set_url_mod_sources_error("".into());

	let window_weak = window.as_weak();
	let database_for_refresh = database.clone();

	thread::spawn(move || {
		let result = url_mod_provider::remove_url_mod_source(&url);

		if result.is_ok() {
			refresh::refresh_mods(&database_for_refresh);
		}

		let _ = slint::invoke_from_event_loop(move || {
			let Some(window) = window_weak.upgrade() else {
				return;
			};

			window.set_url_mod_sources_toggling(false);

			if let Err(err) = result {
				window.set_url_mod_sources_error(err.to_string().into());
			}

			load_sources(&window);
			after_mod_refresh(&window, &database, &selected_game);
		});
	});
}

fn get_mods(url: &str) -> Result<Vec<rai_pal_core::mods::game_mod::GameMod>, String> {
	let runtime = tokio::runtime::Builder::new_current_thread()
		.enable_all()
		.build()
		.map_err(|err| err.to_string())?;

	runtime
		.block_on(url_mod_provider::get_mods_from_url_mod_source(url))
		.map_err(|err| err.to_string())
}

fn after_mod_refresh(
	window: &MainWindow,
	database: &Arc<DbMutex>,
	selected_game: &SelectedGameState,
) {
	mods_tab::load(window, database);

	let Some((_, game)) = selected_game.lock().ok().and_then(|game| game.clone()) else {
		return;
	};

	let Ok(updated_game) = database.get_game(&game.provider_id, &game.game_id) else {
		return;
	};

	mods::refresh_mods(window, database, &updated_game);
}
