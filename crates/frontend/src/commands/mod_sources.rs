use std::{rc::Rc, sync::Arc, thread};

use rai_pal_backend::{
	local_database::game_database::GameDatabase, mod_providers::url_mod_provider, result::LogErrExt,
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
			let result = get_mods(&url);

			crate::runtime::invoke_on_ui(move || {
				let Some(window) = window.upgrade() else {
					return;
				};

				match result {
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

		let url = window
			.global::<crate::ConfirmModSourceState>()
			.get_url()
			.to_string();

		if url.is_empty()
			|| (window.global::<crate::ConfirmModSourceState>().get_state()
				== crate::ActionState::Loading)
		{
			return;
		}

		window
			.global::<crate::ConfirmModSourceState>()
			.set_state(crate::ActionState::Loading);
		window
			.global::<crate::ConfirmModSourceState>()
			.set_error(MessageData::default());

		let window = window.as_weak();
		let database = Arc::clone(&self.database);

		thread::spawn(move || {
			let result = url_mod_provider::add_url_mod_source(url.clone());

			if result.is_ok()
				&& let Err(err) = refresh::refresh_mods(&database)
			{
				log::warn!("Failed to refresh mods after changing URL mod sources: {err}");
			}

			crate::runtime::invoke_on_ui(move || {
				let Some(window) = window.upgrade() else {
					return;
				};

				if let Err(err) = result {
					window
						.global::<crate::ConfirmModSourceState>()
						.set_error(notifications::data(MessageContext::UpdateModSources, &err));
					window
						.global::<crate::ConfirmModSourceState>()
						.set_state(crate::ActionState::Error);
					return;
				}

				window
					.global::<crate::ConfirmModSourceState>()
					.set_state(crate::ActionState::Idle);
				window
					.global::<crate::ConfirmModSourceState>()
					.set_open(false);

				window
					.global::<crate::ModSourcesState>()
					.set_new_url("".into());
				load_sources(&window);

				if let Some(handler) = Self::current() {
					handler.after_mod_refresh();
				}
			});
		});
	}

	pub fn set_mod_source_enabled(&self, url: String, enabled: bool) {
		let Some(main_window) = self.ui() else {
			return;
		};

		if main_window.global::<crate::ModSourcesState>().get_state() == crate::ActionState::Loading
		{
			return;
		}

		main_window
			.global::<crate::ModSourcesState>()
			.set_state(crate::ActionState::Loading);
		main_window
			.global::<crate::ModSourcesState>()
			.set_error(MessageData::default());

		let window_weak = main_window.as_weak();
		let database = Arc::clone(&self.database);

		thread::spawn(move || {
			let result = url_mod_provider::set_url_mod_source_enabled(&url, enabled);

			if result.is_ok()
				&& let Err(err) = refresh::refresh_mods(&database)
			{
				log::warn!("Failed to refresh mods after changing URL mod sources: {err}");
			}

			crate::runtime::invoke_on_ui(move || {
				let Some(window) = window_weak.upgrade() else {
					return;
				};

				if let Err(err) = result {
					window
						.global::<crate::ModSourcesState>()
						.set_error(notifications::data(MessageContext::UpdateModSources, &err));
					window
						.global::<crate::ModSourcesState>()
						.set_state(crate::ActionState::Error);
				} else {
					window
						.global::<crate::ModSourcesState>()
						.set_state(crate::ActionState::Idle);
				}

				load_sources(&window);

				if let Some(handler) = Self::current() {
					handler.after_mod_refresh();
				}
			});
		});
	}

	pub fn remove_mod_source(&self, url: String) {
		let Some(main_window) = self.ui() else {
			return;
		};

		if main_window.global::<crate::ModSourcesState>().get_state() == crate::ActionState::Loading
		{
			return;
		}

		main_window
			.global::<crate::ModSourcesState>()
			.set_state(crate::ActionState::Loading);
		main_window
			.global::<crate::ModSourcesState>()
			.set_error(MessageData::default());

		let window_weak = main_window.as_weak();
		let database = Arc::clone(&self.database);

		thread::spawn(move || {
			let result = url_mod_provider::remove_url_mod_source(&url);

			if result.is_ok()
				&& let Err(err) = refresh::refresh_mods(&database)
			{
				log::warn!("Failed to refresh mods after changing URL mod sources: {err}");
			}

			crate::runtime::invoke_on_ui(move || {
				let Some(window) = window_weak.upgrade() else {
					return;
				};

				window
					.global::<crate::ModSourcesState>()
					.set_state(crate::ActionState::Idle);

				if let Err(err) = result {
					window
						.global::<crate::ModSourcesState>()
						.set_error(notifications::data(MessageContext::UpdateModSources, &err));
				}

				load_sources(&window);

				if let Some(handler) = Self::current() {
					handler.after_mod_refresh();
				}
			});
		});
	}

	fn after_mod_refresh(&self) {
		self.load_mods_tab();

		let Some((_, game)) = self.selected_game() else {
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

fn load_sources(window: &MainWindow) {
	let response = url_mod_provider::get_url_mod_sources();

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

fn get_mods(url: &str) -> Result<Vec<rai_pal_backend::mods::manifest::Mod>, String> {
	runtime::block_on(url_mod_provider::get_mods_from_url_mod_source(url))
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
}
