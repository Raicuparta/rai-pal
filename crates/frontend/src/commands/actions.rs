use std::{sync::Arc, thread};

use slint::ComponentHandle as _;

use crate::{
	ActionKey, ActionKind, MessageContext, ProviderAction, action_state, analytics, notifications,
};

use rai_pal_backend::{
	app_paths,
	game::Game,
	game_providers::{game_provider, provider_command::ProviderCommandAction},
	local_database::mod_database::{GameEnvironment, ModDatabase},
	mods::manifest::install_steps,
	open_better,
	path_extensions::PathExt,
	result::{Error, LogErrExt, Result},
};

use super::{CommandHandler, on_command};

impl CommandHandler {
	pub fn run_provider_action(&self, value: ProviderAction) {
		let Some(action) = Option::<ProviderCommandAction>::from(value) else {
			return;
		};
		let action_id = action.to_string();

		let Some((_, game)) = self.selected_game() else {
			return;
		};

		analytics::send(
			rai_pal_backend::analytics::Event::ProviderCommand,
			[
				("action", action.to_string()),
				("game", game.display_title.clone()),
			],
		);

		if let Some(window) = self.ui() {
			notifications::info(
				&window,
				notifications::data(MessageContext::ProviderCommandRunning, ""),
			);
		}

		self.run_game_action(
			game,
			action,
			action_state::key(ActionKind::ProviderAction, action_id),
		);
	}

	fn run_game_action(&self, game: Game, action: ProviderCommandAction, action_id: ActionKey) {
		let database = Arc::clone(&self.database);
		let window = self.window();

		spawn_action(window, action_id, MessageContext::RunGame, move || {
			let provider_command = game
				.get_provider_command(action)
				.ok_or_else(|| Error::DataEntryNotFound(action.to_string()))?;
			let environment = match action {
				ProviderCommandAction::StartViaProvider | ProviderCommandAction::StartViaExe => {
					database.get_game_environment(&game)?
				}
				_ => GameEnvironment::default(),
			};

			provider_command.run(&game, &environment.variables, &environment.mod_ids)
		});
	}

	pub fn open_game_folder(&self) {
		self.open_selected(
			ActionKind::OpenGameFolder,
			MessageContext::OpenGameFolder,
			Game::open_game_folder,
		);
	}

	pub fn open_game_mods_folder(&self) {
		self.open_selected(
			ActionKind::OpenGameModsFolder,
			MessageContext::OpenGameModsFolder,
			Game::open_mods_folder,
		);
	}

	pub fn open_game_data_folder(&self) {
		self.open_selected(
			ActionKind::OpenGameDataFolder,
			MessageContext::OpenGameDataFolder,
			Game::open_data_folder,
		);
	}

	pub fn open_wine_prefix_folder(&self) {
		self.open_selected(
			ActionKind::OpenWinePrefixFolder,
			MessageContext::OpenWinePrefixFolder,
			|game| {
				game_provider::get_provider(game.provider_id)?
					.get_wine_prefix_path(game)?
					.open_folder_or_parent()
			},
		);
	}

	pub fn open_wine_binary_folder(&self) {
		self.open_selected(
			ActionKind::OpenWineBinaryFolder,
			MessageContext::OpenWineBinaryFolder,
			|game| {
				game_provider::get_provider(game.provider_id)?
					.get_wine_binary_path(game)?
					.open_folder_or_parent()
			},
		);
	}

	fn open_selected(
		&self,
		kind: ActionKind,
		context: MessageContext,
		open: impl FnOnce(&Game) -> Result + Send + 'static,
	) {
		let Some((_, game)) = self.selected_game() else {
			return;
		};

		spawn_action(
			self.window(),
			action_state::key(kind, ""),
			context,
			move || open(&game),
		);
	}

	pub fn run_standalone_mod(&self, mod_id: String) {
		analytics::send_mod(rai_pal_backend::analytics::Event::RunMod, &mod_id, None);

		let database = Arc::clone(&self.database);
		let action_id = action_state::key(ActionKind::StandaloneRun, mod_id.clone());

		spawn_action(
			self.window(),
			action_id,
			MessageContext::RunMod,
			move || {
				database
					.get_mod(&mod_id)
					.and_then(|game_mod| game_mod.run(None))
			},
		);
	}

	pub fn install_standalone_mod(&self, mod_id: String) {
		analytics::send_mod(rai_pal_backend::analytics::Event::InstallMod, &mod_id, None);

		let database = Arc::clone(&self.database);
		let window = self.window();
		let downloads = self.downloads.clone();

		let action_id = action_state::key(ActionKind::StandaloneInstall, mod_id.clone());
		action_state::start(&window, action_id.clone());

		thread::spawn(move || {
			let Some(runtime) = tokio::runtime::Builder::new_current_thread()
				.enable_all()
				.build()
				.ok_or_log("Failed to create async runtime")
			else {
				action_state::finish(
					&window,
					action_id.clone(),
					Err("Failed to create async runtime"),
					MessageContext::InstallMod,
				);
				return;
			};

			let game_mod = match database.get_mod(&mod_id) {
				Ok(game_mod) => game_mod,
				Err(err) => {
					action_state::finish(
						&window,
						action_id.clone(),
						Err(err),
						MessageContext::InstallMod,
					);
					return;
				}
			};

			let steps = install_steps(std::slice::from_ref(&game_mod));
			downloads.pending_steps(&steps);

			let result = runtime.block_on(async {
				game_mod
					.install(None, |status| downloads.record(status))
					.await?;
				database.refresh_installed_mods()?;
				Ok::<(), rai_pal_backend::result::Error>(())
			});

			match result {
				Ok(()) => {
					downloads.finish_steps(&steps);
					action_state::finish(
						&window,
						action_id.clone(),
						Ok::<(), rai_pal_backend::result::Error>(()),
						MessageContext::InstallMod,
					);
				}
				Err(err) => {
					downloads.fail_steps(&steps);
					action_state::finish(
						&window,
						action_id.clone(),
						Err(err),
						MessageContext::InstallMod,
					);
				}
			}
		});
	}

	pub fn open_local_mods_folder(&self) {
		spawn_action(
			self.window(),
			action_state::key(ActionKind::OpenLocalModsFolder, ""),
			MessageContext::OpenLocalModsFolder,
			|| app_paths::local_mods_path()?.open_folder_or_parent(),
		);
	}

	pub fn open_logs_folder(&self) {
		spawn_action(
			self.window(),
			action_state::key(ActionKind::OpenLogsFolder, ""),
			MessageContext::OpenLogsFolder,
			|| app_paths::logs_path()?.open_folder_or_parent(),
		);
	}
}

pub fn register(window: &crate::MainWindow) {
	on_command!(window, GamesActions, on_run_provider_action, |action| handler => {
		handler.run_provider_action(action);
	});
	on_command!(window, GamesActions, on_open_game_folder, || handler => {
		handler.open_game_folder();
	});
	on_command!(window, GamesActions, on_open_game_mods_folder, || handler => {
		handler.open_game_mods_folder();
	});
	on_command!(window, GamesActions, on_open_game_data_folder, || handler => {
		handler.open_game_data_folder();
	});
	on_command!(window, GamesActions, on_open_wine_prefix_folder, || handler => {
		handler.open_wine_prefix_folder();
	});
	on_command!(window, GamesActions, on_open_wine_binary_folder, || handler => {
		handler.open_wine_binary_folder();
	});
	on_command!(window, ModActions, on_run_standalone, |mod_id| handler => {
		handler.run_standalone_mod(mod_id.to_string());
	});
	on_command!(window, ModActions, on_install_standalone, |mod_id| handler => {
		handler.install_standalone_mod(mod_id.to_string());
	});
	on_command!(window, ModActions, on_open_local_mods_folder, || handler => {
		handler.open_local_mods_folder();
	});
	on_command!(window, AppActions, on_open_logs_folder, || handler => {
		handler.open_logs_folder();
	});

	window.global::<crate::ModActions>().on_open_url(|url| {
		open_url(url.to_string());
	});
}

pub fn open_url(url: String) {
	thread::spawn(move || {
		if let Err(err) = open_better::open_detached_better(url) {
			log::error!("Failed to open URL: {err}");
		}
	});
}

fn spawn_action(
	window: slint::Weak<crate::MainWindow>,
	action_id: ActionKey,
	context: MessageContext,
	action: impl FnOnce() -> Result + Send + 'static,
) {
	action_state::start(&window, action_id.clone());

	thread::spawn(move || {
		let result = action();
		action_state::finish(&window, action_id, result, context);
	});
}
