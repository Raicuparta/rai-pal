use std::{collections::BTreeMap, sync::Arc, thread};

use rai_pal_core::{
	game::DbGame,
	game_providers::{
		game_provider,
		provider_command::ProviderCommandAction,
	},
	local_database::{
		app_database::DbMutex,
		mod_database::ModDatabase,
	},
	maps::TryGettable,
	path_extensions::PathExt,
	result::Result,
};

pub fn action_label(action: ProviderCommandAction) -> &'static str {
	match action {
		ProviderCommandAction::Install => "Install",
		ProviderCommandAction::ShowInLibrary => "Show in library",
		ProviderCommandAction::ShowInStore => "Show in store",
		ProviderCommandAction::StartViaProvider => "Start via provider",
		ProviderCommandAction::StartViaExe => "Start via exe",
		ProviderCommandAction::OpenInBrowser => "Open in browser",
	}
}

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

pub fn run_provider_action(
	database: Arc<DbMutex>,
	game: DbGame,
	action: ProviderCommandAction,
) {
	spawn_logged(move || {
		let provider_command = game.provider_commands.try_get(&action)?;
		let environment = match action {
			ProviderCommandAction::StartViaProvider | ProviderCommandAction::StartViaExe => {
				database.get_game_environment(&game)?
			}
			_ => BTreeMap::default(),
		};

		provider_command.run(&game, &environment)
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

fn spawn_logged(action: impl FnOnce() -> Result + Send + 'static) {
	thread::spawn(move || {
		if let Err(err) = action() {
			eprintln!("Game action failed: {err}");
		}
	});
}
