use std::{collections::HashMap, sync::Arc};

use rai_pal_core::{
	game_providers::game_provider::GameProviderId,
	local_database::{app_database::DbMutex, game_database::GameDatabase},
	result::LogErrExt,
};
use slint::ComponentHandle;
use url::Url;

use crate::{MainWindow, downloads::Tracker, mod_sources, mods, selected::SelectedGameState};

#[derive(Clone, Copy, Debug)]
enum ModAction {
	Install,
	Uninstall,
	Run,
}

pub fn handle(
	raw_url: &str,
	window: &MainWindow,
	database: &Arc<DbMutex>,
	selected_game: &SelectedGameState,
	downloads: &Tracker,
) {
	let Some(url) = Url::parse(raw_url).ok_or_log("Failed to parse deep link URL") else {
		return;
	};

	if url.scheme() != "rai-pal" {
		return;
	}

	let Some(action) = url.host_str() else {
		return;
	};

	let query = url.query_pairs().into_owned().collect::<HashMap<_, _>>();

	match action {
		"add-mod-source" => {
			if let Some(source_url) = query.get("url") {
				mod_sources::request_add(window, source_url.clone());
			}
		}
		"install-mod" => dispatch_mod_action(
			ModAction::Install,
			&query,
			window,
			database,
			selected_game,
			downloads,
		),
		"uninstall-mod" => dispatch_mod_action(
			ModAction::Uninstall,
			&query,
			window,
			database,
			selected_game,
			downloads,
		),
		"run-mod" => dispatch_mod_action(
			ModAction::Run,
			&query,
			window,
			database,
			selected_game,
			downloads,
		),
		_ => log::warn!("Ignoring unknown rai-pal:// path `{action}`"),
	}
}

fn dispatch_mod_action(
	action: ModAction,
	query: &HashMap<String, String>,
	window: &MainWindow,
	database: &Arc<DbMutex>,
	selected_game: &SelectedGameState,
	downloads: &Tracker,
) {
	let Some(provider_id) = query.get("providerId") else {
		log::warn!("Ignoring rai-pal:// {action:?} deep link without a providerId");
		return;
	};
	let Some(provider_id) = provider_id
		.parse::<GameProviderId>()
		.ok_or_log("Failed to parse deep link provider ID")
	else {
		log::warn!("Ignoring rai-pal:// {action:?} deep link with an invalid providerId");
		return;
	};

	let (Some(game_id), Some(mod_id)) = (query.get("gameId"), query.get("modId")) else {
		log::warn!("Ignoring rai-pal:// {action:?} deep link without gameId or modId");
		return;
	};

	let Some(game) = database
		.get_game(&provider_id, game_id)
		.ok_or_log("Failed to load game from deep link")
	else {
		log::warn!("Ignoring rai-pal:// {action:?} deep link for an unknown game");
		return;
	};

	let window = window.as_weak();
	let mod_id = mod_id.clone();

	match action {
		ModAction::Install => {
			mods::install_mod(database.clone(), window, game, mod_id, downloads.clone());
		}
		ModAction::Uninstall => {
			mods::uninstall_mod(database.clone(), window, game, mod_id);
		}
		ModAction::Run => {
			mods::run_mod(database.clone(), window, game, mod_id);
		}
	}

	// Keep the reference used so callers can select the game later if needed.
	let _ = selected_game;
}
