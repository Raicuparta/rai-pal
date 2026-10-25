use std::{collections::BTreeMap, sync::Arc, thread};

use rai_pal_core::{
	app_paths,
	game::DbGame,
	game_providers::{game_provider, provider_command::ProviderCommandAction},
	local_database::{app_database::DbMutex, mod_database::ModDatabase},
	maps::TryGettable,
	open_better,
	path_extensions::PathExt,
	result::Result,
};

pub fn primary_start(game: &DbGame) -> Option<ProviderCommandAction> {
	if game
		.provider_commands
		.contains_key(&ProviderCommandAction::StartViaProvider)
	{
		Some(ProviderCommandAction::StartViaProvider)
	} else if game
		.provider_commands
		.contains_key(&ProviderCommandAction::StartViaExe)
	{
		Some(ProviderCommandAction::StartViaExe)
	} else {
		None
	}
}

pub fn secondary_actions(game: &DbGame) -> Vec<ProviderCommandAction> {
	game.provider_commands
		.keys()
		.copied()
		.filter(|action| {
			!matches!(
				action,
				ProviderCommandAction::StartViaProvider | ProviderCommandAction::StartViaExe
			)
		})
		.collect()
}

pub fn run_provider_action(database: Arc<DbMutex>, game: DbGame, action: ProviderCommandAction) {
	spawn_logged(move || {
		let provider_command = game.provider_commands.try_get(&action)?;
		let (environment, environment_mod_ids) = match action {
			ProviderCommandAction::StartViaProvider | ProviderCommandAction::StartViaExe => {
				database.get_game_environment(&game)?
			}
			_ => (BTreeMap::default(), Vec::new()),
		};

		provider_command.run(&game, &environment, &environment_mod_ids)
	});
}

pub fn open_game_folder(game: DbGame) {
	spawn_logged(move || game.open_game_folder());
}

pub fn open_game_mods_folder(game: DbGame) {
	spawn_logged(move || game.open_mods_folder());
}

pub fn open_game_data_folder(game: DbGame) {
	spawn_logged(move || game.open_data_folder());
}

pub fn open_wine_prefix_folder(game: DbGame) {
	spawn_logged(move || {
		game_provider::get_provider(game.provider_id)?
			.get_wine_prefix_path(&game)?
			.open_folder_or_parent()
	});
}

pub fn open_wine_binary_folder(game: DbGame) {
	spawn_logged(move || {
		game_provider::get_provider(game.provider_id)?
			.get_wine_binary_path(&game)?
			.open_folder_or_parent()
	});
}

pub fn run_standalone_mod(database: Arc<DbMutex>, mod_id: String) {
	thread::spawn(move || {
		let result = database
			.get_mod(&mod_id)
			.and_then(|game_mod| game_mod.run(None));

		if let Err(err) = result {
			eprintln!("Failed to run mod {mod_id}: {err}");
		}
	});
}

pub fn install_standalone_mod(database: Arc<DbMutex>, mod_id: String) {
	thread::spawn(move || {
		let Ok(runtime) = tokio::runtime::Builder::new_current_thread()
			.enable_all()
			.build()
		else {
			return;
		};

		let result = runtime.block_on(async {
			database.get_mod(&mod_id)?.install(None, |_| {}).await?;
			database.refresh_installed_mods()?;
			Ok::<(), rai_pal_core::result::Error>(())
		});

		if let Err(err) = result {
			eprintln!("Failed to install mod {mod_id}: {err}");
		}
	});
}

pub fn open_local_mods_folder() {
	spawn_logged(|| app_paths::local_mods_path()?.open_folder_or_parent());
}

pub fn open_url(url: String) {
	spawn_logged(move || open_better::open_detached_better(url));
}

fn spawn_logged(action: impl FnOnce() -> Result + Send + 'static) {
	thread::spawn(move || {
		if let Err(err) = action() {
			eprintln!("Game action failed: {err}");
		}
	});
}
