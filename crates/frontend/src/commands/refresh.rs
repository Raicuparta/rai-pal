use std::{
	sync::{Arc, Mutex},
	thread,
	time::Duration,
};

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
use slint::{ComponentHandle, ModelRc, SharedString};

use crate::{
	ActionKind, MainWindow, MessageContext, RefreshTaskData, RefreshTaskKind, action_state,
	model::apply_game_to_row, notifications, runtime, selected,
};

use super::query::ScrollBehavior;
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
			.set_tasks(ModelRc::default());

		let window = self.window();
		let database = Arc::clone(&self.database);

		thread::spawn(move || {
			let tasks = RefreshTasks::new(window.clone());

			thread::scope(|scope| {
				scope.spawn(|| {
					let task = task(RefreshTaskKind::RemoteData);
					tasks.push(task.clone());

					if let Err(err) = refresh_remote_database(&database) {
						log::error!("Failed to refresh remote database: {err}");
						notifications::error_async(
							&window,
							notifications::data(MessageContext::RefreshRemoteData, &err),
						);
					}

					tasks.pop(&task);
				});

				scope.spawn(|| {
					let task = task(RefreshTaskKind::Mods);
					tasks.push(task.clone());

					if let Err(err) = refresh_mods(&database) {
						log::error!("Failed to refresh mods: {err}");
						notifications::error_async(
							&window,
							notifications::data(MessageContext::RefreshMods, &err),
						);
					}

					tasks.pop(&task);
				});

				for provider in GameProviderId::ALL {
					let window = &window;
					let database = &database;
					let tasks = &tasks;

					scope.spawn(move || {
						let task = provider_task(provider);
						tasks.push(task.clone());

						match provider.insert_games(database) {
							Ok(()) => reload_games_from_database(),
							Err(err) => {
								log::error!("Failed to refresh {provider} games: {err}");
								notifications::error_async(
									window,
									notifications::data_with_subject(
										MessageContext::RefreshGames,
										provider.to_string(),
										&err,
									),
								);
							}
						}

						tasks.pop(&task);
					});
				}
			});

			if let Err(err) = database.refresh_installed_mods() {
				log::error!("Failed to refresh installed mods: {err}");
				notifications::error_async(
					&window,
					notifications::data(MessageContext::RefreshInstalledMods, &err),
				);
			}

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
					.set_tasks(ModelRc::default());
				handler.load_mods_tab();
				handler.refresh_filters();
				handler.start_query(Duration::ZERO, ScrollBehavior::Keep);
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
						MessageContext::RefreshGame,
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
				MessageContext::RefreshGame,
			);
		});
	}

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

fn refresh_remote_database(database: &DbMutex) -> Result<(), String> {
	runtime::block_on(async {
		let path = remote_game::download_database().await?;
		attach_remote(database, &path)
	})
}

fn provider_task(provider: GameProviderId) -> RefreshTaskData {
	RefreshTaskData {
		kind: RefreshTaskKind::Provider,
		provider: provider.to_string().into(),
	}
}

fn task(kind: RefreshTaskKind) -> RefreshTaskData {
	RefreshTaskData {
		kind,
		provider: SharedString::default(),
	}
}

fn reload_games_from_database() {
	crate::runtime::invoke_on_ui(|| {
		let Some(handler) = CommandHandler::current() else {
			return;
		};

		handler.refresh_filters();
		handler.start_query(Duration::from_millis(100), ScrollBehavior::Keep);
	});
}

struct RefreshTasks {
	window: slint::Weak<MainWindow>,
	tasks: Mutex<Vec<RefreshTaskData>>,
}

impl RefreshTasks {
	fn new(window: slint::Weak<MainWindow>) -> Self {
		Self {
			window,
			tasks: Mutex::default(),
		}
	}

	fn push(&self, task: RefreshTaskData) {
		let Some(mut tasks) = self.tasks.lock().ok_or_log("Failed to lock refresh tasks") else {
			return;
		};

		tasks.push(task);
		self.publish(&tasks);
	}

	fn pop(&self, task: &RefreshTaskData) {
		let Some(mut tasks) = self.tasks.lock().ok_or_log("Failed to lock refresh tasks") else {
			return;
		};

		if let Some(index) = tasks.iter().position(|item| item == task) {
			tasks.remove(index);
		}

		self.publish(&tasks);
	}

	fn publish(&self, tasks: &[RefreshTaskData]) {
		let window = self.window.clone();
		let tasks = tasks.to_vec();
		let active = !tasks.is_empty();

		crate::runtime::invoke_on_ui(move || {
			let Some(window) = window.upgrade() else {
				return;
			};

			window.global::<crate::Refresh>().set_refreshing(active);
			window
				.global::<crate::Refresh>()
				.set_tasks(std::rc::Rc::new(slint::VecModel::from(tasks)).into());
		});
	}
}

pub fn register(window: &MainWindow) {
	on_command!(window, Refresh, on_refresh_all, || handler => {
		handler.refresh_all();
	});
	on_command!(window, GamesActions, on_refresh_game, || handler => {
		handler.refresh_game();
	});
}
