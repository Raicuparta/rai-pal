use std::{sync::Arc, thread, time::Duration};

use rai_pal_backend::{
	game_providers::game_provider::GameProviderId,
	local_database::{
		app_database::DbMutex,
		game_database::{GameDatabase, attach_remote},
		mod_database::ModDatabase,
	},
	mod_providers::mod_provider,
	remote_game,
	result::LogErrExt,
};
use slint::ComponentHandle;

use crate::{
	ActionKind, MainWindow, action_state, model::apply_game_to_row, notifications, runtime,
	selected,
};

use super::{CommandHandler, on_command};

impl CommandHandler {
	pub fn refresh_all(&self) {
		let Some(ui_window) = self.ui() else {
			return;
		};

		if ui_window.global::<crate::Refresh>().get_refreshing() {
			return;
		}

		ui_window.global::<crate::Refresh>().set_refreshing(true);
		ui_window
			.global::<crate::Refresh>()
			.set_task_names("".into());

		let strings = ui_window.global::<crate::HostStrings>();
		let mods_task = strings.get_refresh_mods_task().to_string();
		let remote_data_task = strings.get_refresh_remote_data_task().to_string();
		let window = self.window();
		let database = Arc::clone(&self.database);

		thread::spawn(move || {
			let mut tasks = Vec::<String>::new();

			for provider in GameProviderId::ALL {
				push_task(&window, &mut tasks, &format!("{provider}"));

				if let Err(err) = provider.insert_games(&database) {
					log::error!("Failed to refresh {provider} games: {err}");
					notifications::error_async(
						&window,
						format!("Failed to refresh {provider} games: {err}"),
					);
				}

				pop_task(&window, &mut tasks, &format!("{provider}"));
			}

			if let Err(err) = database.refresh_installed_mods() {
				log::error!("Failed to refresh installed mods: {err}");
				notifications::error_async(
					&window,
					format!("Failed to refresh installed mods: {err}"),
				);
			}

			push_task(&window, &mut tasks, &mods_task);
			if let Err(err) = refresh_mods(&database) {
				log::error!("Failed to refresh mods: {err}");
				notifications::error_async(&window, format!("Failed to refresh mods: {err}"));
			}
			pop_task(&window, &mut tasks, &mods_task);

			push_task(&window, &mut tasks, &remote_data_task);
			if let Err(err) = refresh_remote_database() {
				log::error!("Failed to refresh remote database: {err}");
				notifications::error_async(
					&window,
					format!("Failed to refresh remote database: {err}"),
				);
			}
			pop_task(&window, &mut tasks, &remote_data_task);

			crate::runtime::invoke_on_ui(move || {
				let Some(handler) = Self::current() else {
					return;
				};
				let Some(handler_window) = handler.ui() else {
					return;
				};

				handler_window
					.global::<crate::Refresh>()
					.set_refreshing(false);
				handler_window
					.global::<crate::Refresh>()
					.set_task_names("".into());
				handler.load_mods_tab();
				handler.start_query(Duration::ZERO);
				handler.reload_selected_game();
			});
		});
	}

	pub fn refresh_game(&self) {
		let Some((_, game)) = self.selected_game() else {
			return;
		};

		let action_id = action_state::key(ActionKind::RefreshGame, "");
		let window = self.window();
		action_state::start(&window, action_id.clone());

		let database = Arc::clone(&self.database);

		thread::spawn(move || {
			let mut current_game = game;

			let result = (|| {
				current_game.refresh_executable()?;
				database.insert_game(&current_game);
				database.refresh_installed_mods()?;
				database.get_game(&current_game.provider_id, &current_game.game_id)
			})();

			let updated_game = match result {
				Ok(updated_game) => updated_game,
				Err(err) => {
					log::error!("Failed to refresh game: {err}");
					action_state::finish(
						&window,
						action_id.clone(),
						Err(err),
						"Failed to refresh game",
					);
					return;
				}
			};

			let finish_window = window.clone();
			crate::runtime::invoke_on_ui(move || {
				let Some(handler) = Self::current() else {
					return;
				};

				// The refresh runs in the background; don't apply it if the user
				// selected another game in the meantime.
				let Some((index, selected_game)) = handler.selected_game() else {
					return;
				};

				if selected_game.provider_id != updated_game.provider_id
					|| selected_game.game_id != updated_game.game_id
				{
					return;
				}

				handler.set_selected_game(index, updated_game.clone());
				handler.refresh_selected_row(index, &updated_game);
				handler.refresh_mods(&updated_game);
			});

			action_state::finish(
				&finish_window,
				action_id,
				Ok::<(), rai_pal_backend::result::Error>(()),
				"",
			);
		});
	}

	/// Tauri refreshed the selected game whenever the window regained focus, so
	/// mods installed while the game was running showed up on return.
	pub fn refresh_selected_game_on_focus(&self) {
		let Some((_, game)) = self.selected_game() else {
			return;
		};

		if game.exe_path.is_none() {
			return;
		}

		if self
			.ui()
			.is_some_and(|window| window.global::<crate::Refresh>().get_refreshing())
		{
			return;
		}

		self.refresh_game();
	}

	pub fn reload_selected_game(&self) {
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

		let Some(window) = self.ui() else {
			return;
		};

		let mut row = window.get_selected_game_row();
		apply_game_to_row(&mut row, &updated_game);
		selected::show(&window, &updated_game, row);
		self.refresh_mods(&updated_game);
	}
}

pub fn refresh_mods(database: &DbMutex) -> Result<(), String> {
	runtime::block_on(mod_provider::refresh_all_mods(database))
}

fn refresh_remote_database() -> Result<(), String> {
	runtime::block_on(async {
		let path = remote_game::download_database().await?;
		attach_remote(&path)
	})
}

fn push_task(window: &slint::Weak<MainWindow>, tasks: &mut Vec<String>, task: &str) {
	tasks.push(task.to_string());
	publish_tasks(window, tasks);
}

fn pop_task(window: &slint::Weak<MainWindow>, tasks: &mut Vec<String>, task: &str) {
	if let Some(index) = tasks.iter().position(|item| item == task) {
		tasks.remove(index);
	}
	publish_tasks(window, tasks);
}

fn publish_tasks(window: &slint::Weak<MainWindow>, tasks: &[String]) {
	let window = window.clone();
	let names = tasks.join(", ");
	let active = !tasks.is_empty();

	crate::runtime::invoke_on_ui(move || {
		let Some(window) = window.upgrade() else {
			return;
		};

		window.global::<crate::Refresh>().set_refreshing(active);
		window
			.global::<crate::Refresh>()
			.set_task_names(names.into());
	});
}

pub fn register(window: &MainWindow) {
	on_command!(window, Refresh, on_refresh_all, || handler => {
		handler.refresh_all();
	});
	on_command!(window, GamesActions, on_refresh_game, || handler => {
		handler.refresh_game();
	});
}
