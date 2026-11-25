use std::{rc::Rc, sync::Arc, thread};

use rai_pal_backend::{
	local_database::{app_database::Db, game_database::GameDatabase},
	mod_sources::url,
	result::{Error, LogErrExt},
};
use slint::ComponentHandle;

use crate::{MainWindow, MessageContext, MessageData, UrlModSourceData, notifications, runtime};

use super::{CommandHandler, on_command, refresh};

impl CommandHandler {
	pub fn show_mod_sources(&self) {
		let Some(window) = self.ui() else {
			return;
		};

		window
			.global::<crate::ModSourcesState>()
			.set_error(MessageData::default());
		load_sources(&window);
	}

	pub fn prompt_add_mod_source(&self, url: &str) {
		let Some(window) = self.ui() else {
			return;
		};

		let url = url.trim().to_string();

		if url.is_empty() {
			window
				.global::<crate::ModSourcesState>()
				.set_error(notifications::data(MessageContext::EmptyUrl, ""));
			return;
		}

		window
			.global::<crate::ConfirmModSourceState>()
			.set_url(url.clone().into());
		window
			.global::<crate::ConfirmModSourceState>()
			.set_mods(Rc::new(slint::VecModel::from(Vec::<slint::SharedString>::new())).into());
		window
			.global::<crate::ConfirmModSourceState>()
			.set_state(crate::ActionState::Loading);
		window
			.global::<crate::ConfirmModSourceState>()
			.set_error(MessageData::default());
		window
			.global::<crate::ConfirmModSourceState>()
			.set_open(true);

		let window = window.as_weak();

		thread::spawn(move || {
			let result =
				runtime::block_on(url::fetch_mod_database(&url)).map(|database| database.mods);

			crate::runtime::with_ui_window(&window, move |window| match result {
				Ok(mods) => {
					let titles = mods
						.into_iter()
						.map(|game_mod| slint::SharedString::from(game_mod.title))
						.collect::<Vec<_>>();
					window
						.global::<crate::ConfirmModSourceState>()
						.set_mods(Rc::new(slint::VecModel::from(titles)).into());
					window
						.global::<crate::ConfirmModSourceState>()
						.set_state(crate::ActionState::Success);
				}
				Err(err) => {
					window
						.global::<crate::ConfirmModSourceState>()
						.set_error(notifications::data(MessageContext::UpdateModSources, &err));
					window
						.global::<crate::ConfirmModSourceState>()
						.set_state(crate::ActionState::Error);
				}
			});
		});
	}

	pub fn cancel_add_mod_source(&self) {
		let Some(window) = self.ui() else {
			return;
		};

		let state = window.global::<crate::ConfirmModSourceState>();
		state.set_error(MessageData::default());
		state.set_mods(slint::ModelRc::default());
		state.set_state(crate::ActionState::Idle);
	}

	pub fn add_mod_source(&self) {
		let Some(window) = self.ui() else {
			return;
		};

		let state = window.global::<crate::ConfirmModSourceState>();
		let url = state.get_url().to_string();

		if url.is_empty() || state.get_state() == crate::ActionState::Loading {
			return;
		}

		state.set_state(crate::ActionState::Loading);
		state.set_error(MessageData::default());

		run_mod_source_change(
			window.as_weak(),
			Arc::clone(&self.database),
			move || url::add_url_mod_source(url),
			|window, result| {
				let confirm_state = window.global::<crate::ConfirmModSourceState>();

				if let Err(err) = result {
					confirm_state
						.set_error(notifications::data(MessageContext::UpdateModSources, err));
					confirm_state.set_state(crate::ActionState::Error);
					return;
				}

				confirm_state.set_state(crate::ActionState::Idle);
				confirm_state.set_open(false);
				window
					.global::<crate::ModSourcesState>()
					.set_new_url("".into());
			},
		);
	}

	pub fn set_mod_source_enabled(&self, url: String, enabled: bool) {
		self.change_mod_sources(move || url::set_url_mod_source_enabled(&url, enabled));
	}

	pub fn remove_mod_source(&self, url: String) {
		self.change_mod_sources(move || url::remove_url_mod_source(&url));
	}

	pub fn move_mod_source(&self, url: String, up: bool) {
		self.change_mod_sources(move || url::move_url_mod_source(&url, up));
	}

	fn change_mod_sources(&self, action: impl FnOnce() -> Result<(), Error> + Send + 'static) {
		let Some(window) = self.ui() else {
			return;
		};

		let state = window.global::<crate::ModSourcesState>();

		if state.get_state() == crate::ActionState::Loading {
			return;
		}

		state.set_state(crate::ActionState::Loading);
		state.set_error(MessageData::default());

		run_mod_source_change(
			window.as_weak(),
			Arc::clone(&self.database),
			action,
			|window, result| {
				let sources_state = window.global::<crate::ModSourcesState>();

				if let Err(err) = result {
					sources_state
						.set_error(notifications::data(MessageContext::UpdateModSources, err));
					sources_state.set_state(crate::ActionState::Error);
				} else {
					sources_state.set_state(crate::ActionState::Idle);
				}
			},
		);
	}

	fn after_mod_refresh(&self) {
		self.load_mods_tab();
		self.refresh_filters();
		self.start_query(
			std::time::Duration::ZERO,
			super::query::ScrollBehavior::Reset,
		);

		let Some(game) = self.selected_game() else {
			return;
		};

		let Some(updated_game) = self
			.database
			.get_game(&game.provider_id, &game.game_id)
			.ok_or_log("Failed to reload selected game")
		else {
			return;
		};

		self.refresh_mods(&updated_game);
	}
}

fn run_mod_source_change(
	window: slint::Weak<MainWindow>,
	database: Arc<Db>,
	action: impl FnOnce() -> Result<(), Error> + Send + 'static,
	on_finish: impl FnOnce(&MainWindow, &Result<(), Error>) + Send + 'static,
) {
	thread::spawn(move || {
		let result = action();

		if result.is_ok() {
			if let Err(err) = refresh::refresh_mods(&database) {
				log::warn!("Failed to refresh mods after changing URL mod sources: {err}");
			}

			if let Err(err) = database.refresh_game_mod_tags() {
				log::warn!("Failed to refresh game mod tags after changing URL mod sources: {err}");
			}
		}

		crate::runtime::with_ui_window(&window, move |window| {
			on_finish(&window, &result);
			load_sources(&window);

			if let Some(handler) = CommandHandler::current() {
				handler.after_mod_refresh();
			}
		});
	});
}

fn load_sources(window: &MainWindow) {
	let response = url::get_url_mod_sources();

	window
		.global::<crate::ModSourcesState>()
		.set_default_source(UrlModSourceData {
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

	window
		.global::<crate::ModSourcesState>()
		.set_sources(Rc::new(slint::VecModel::from(sources)).into());
}

pub fn register(window: &crate::MainWindow) {
	on_command!(window, DialogActions, on_request_url_mod_sources, || handler => {
		handler.show_mod_sources();
	});
	on_command!(window, DialogActions, on_add_url_mod_source, |url| handler => {
		handler.prompt_add_mod_source(url.as_str());
	});
	on_command!(window, DialogActions, on_confirm_url_mod_source, || handler => {
		handler.add_mod_source();
	});
	on_command!(window, DialogActions, on_cancel_url_mod_source, || handler => {
		handler.cancel_add_mod_source();
	});
	on_command!(window, DialogActions, on_set_url_mod_source_enabled, |url, enabled| handler => {
		handler.set_mod_source_enabled(url.to_string(), enabled);
	});
	on_command!(window, DialogActions, on_remove_url_mod_source, |url| handler => {
		handler.remove_mod_source(url.to_string());
	});
	on_command!(window, DialogActions, on_move_url_mod_source, |url, up| handler => {
		handler.move_mod_source(url.to_string(), up);
	});
}
