use std::{sync::Arc, thread};

use crate::{MainWindow, action_state, downloads::Tracker};

use rai_pal_core::{
	app_paths,
	game::Game,
	game_providers::{game_provider, provider_command::ProviderCommandAction},
	local_database::{
		app_database::DbMutex,
		mod_database::{GameEnvironment, ModDatabase},
	},
	mods::manifest::install_steps,
	open_better,
	path_extensions::PathExt,
	result::{Error, Result},
};

pub fn run_provider_action(
	database: Arc<DbMutex>,
	window: slint::Weak<MainWindow>,
	game: Game,
	action: ProviderCommandAction,
	action_id: String,
) {
	spawn_action(window, action_id, "Failed to run game action", move || {
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

pub fn open_game_folder(window: slint::Weak<MainWindow>, action_id: String, game: Game) {
	spawn_action(window, action_id, "Failed to open game folder", move || {
		game.open_game_folder()
	});
}

pub fn open_game_mods_folder(window: slint::Weak<MainWindow>, action_id: String, game: Game) {
	spawn_action(
		window,
		action_id,
		"Failed to open game mods folder",
		move || game.open_mods_folder(),
	);
}

pub fn open_game_data_folder(window: slint::Weak<MainWindow>, action_id: String, game: Game) {
	spawn_action(
		window,
		action_id,
		"Failed to open game data folder",
		move || game.open_data_folder(),
	);
}

pub fn open_wine_prefix_folder(window: slint::Weak<MainWindow>, action_id: String, game: Game) {
	spawn_action(
		window,
		action_id,
		"Failed to open Wine prefix folder",
		move || {
			game_provider::get_provider(game.provider_id)?
				.get_wine_prefix_path(&game)?
				.open_folder_or_parent()
		},
	);
}

pub fn open_wine_binary_folder(window: slint::Weak<MainWindow>, action_id: String, game: Game) {
	spawn_action(
		window,
		action_id,
		"Failed to open Wine binary folder",
		move || {
			game_provider::get_provider(game.provider_id)?
				.get_wine_binary_path(&game)?
				.open_folder_or_parent()
		},
	);
}

pub fn run_standalone_mod(database: Arc<DbMutex>, window: slint::Weak<MainWindow>, mod_id: String) {
	let action_id = format!("standalone-run:{mod_id}");

	spawn_action(
		window,
		action_id,
		format!("Failed to run mod {mod_id}"),
		move || {
			database
				.get_mod(&mod_id)
				.and_then(|game_mod| game_mod.run(None))
		},
	);
}

pub fn install_standalone_mod(
	database: Arc<DbMutex>,
	window: slint::Weak<MainWindow>,
	mod_id: String,
	downloads: Tracker,
) {
	let action_id = format!("standalone-install:{mod_id}");
	action_state::start(&window, action_id.clone());

	thread::spawn(move || {
		let Ok(runtime) = tokio::runtime::Builder::new_current_thread()
			.enable_all()
			.build()
		else {
			action_state::finish(
				&window,
				&action_id,
				Err("Failed to create async runtime"),
				format!("Failed to install mod {mod_id}"),
			);
			return;
		};

		let game_mod = match database.get_mod(&mod_id) {
			Ok(game_mod) => game_mod,
			Err(err) => {
				action_state::finish(
					&window,
					&action_id,
					Err(err),
					format!("Failed to install mod {mod_id}"),
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
			Ok::<(), rai_pal_core::result::Error>(())
		});

		match result {
			Ok(()) => {
				downloads.finish_steps(&steps);
				action_state::finish(
					&window,
					&action_id,
					Ok::<(), rai_pal_core::result::Error>(()),
					"",
				);
			}
			Err(err) => {
				downloads.fail_steps(&steps);
				action_state::finish(
					&window,
					&action_id,
					Err(err),
					format!("Failed to install mod {mod_id}"),
				);
			}
		}
	});
}

pub fn open_local_mods_folder(window: slint::Weak<MainWindow>, action_id: String) {
	spawn_action(
		window,
		action_id,
		"Failed to open local mods folder",
		|| app_paths::local_mods_path()?.open_folder_or_parent(),
	);
}

pub fn open_logs_folder(window: slint::Weak<MainWindow>, action_id: String) {
	spawn_action(window, action_id, "Failed to open logs folder", || {
		app_paths::logs_path()?.open_folder_or_parent()
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
	window: slint::Weak<MainWindow>,
	action_id: String,
	error_context: impl Into<String>,
	action: impl FnOnce() -> Result + Send + 'static,
) {
	let error_context = error_context.into();
	action_state::start(&window, action_id.clone());

	thread::spawn(move || {
		let result = action();
		action_state::finish(&window, &action_id, result, error_context);
	});
}
