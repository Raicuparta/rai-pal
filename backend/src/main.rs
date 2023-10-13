// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![feature(future_join)]

use std::{
	future::{
		self,
		Future,
	},
	sync::Mutex,
};

use game::Game;
use mod_loaders::mod_loader::{
	self,
	ModLoaderActions,
};
use result::{
	Error,
	Result,
};
use steam::{
	appinfo,
	discover_games::DiscoverGame,
	owned_games::OwnedGame,
};
use steamlocate::SteamDir;
use tauri::{
	api::dialog::message,
	Manager,
};

mod files;
mod game;
mod game_engines;
mod game_executable;
mod game_mod;
mod macros;
mod mod_loaders;
mod paths;
mod result;
mod steam;
mod windows;

struct AppState {
	full_state: Mutex<Option<FullState>>,
}

fn get_game(game_id: &str, state: &tauri::State<'_, AppState>) -> Result<Game> {
	if let Ok(read_guard) = state.full_state.lock() {
		let full_state = read_guard
			.as_ref()
			.ok_or(Error::GameNotFound(game_id.to_string()))?;

		let game_map = full_state
			.game_map
			.as_ref()
			.ok_or_else(|| Error::GameNotFound(game_id.to_string()))?;

		let game = game_map
			.get(game_id)
			.ok_or_else(|| Error::GameNotFound(game_id.to_string()))?;

		return Ok(game.clone());
	}

	Err(Error::GameNotFound(game_id.to_string()))
}

serializable_struct!(FullState {
	game_map: Option<game::Map>,
	owned_games: Option<Vec<OwnedGame>>,
	discover_games: Option<Vec<DiscoverGame>>,
	mod_loaders: Option<mod_loader::DataMap>,
});

#[tauri::command]
#[specta::specta]
async fn open_game_folder(game_id: &str, state: tauri::State<'_, AppState>) -> Result {
	get_game(game_id, &state)?.open_game_folder()
}

#[tauri::command]
#[specta::specta]
async fn open_game_mods_folder(game_id: &str, state: tauri::State<'_, AppState>) -> Result {
	get_game(game_id, &state)?.open_mods_folder()
}

#[tauri::command]
#[specta::specta]
async fn open_mods_folder(handle: tauri::AppHandle) -> Result {
	let resources_path = paths::resources_path(&handle)?;
	Ok(open::that_detached(resources_path)?)
}

#[tauri::command]
#[specta::specta]
async fn open_mod_folder(mod_loader_id: &str, mod_id: &str, handle: tauri::AppHandle) -> Result {
	let resources_path = paths::resources_path(&handle)?;
	let mod_loader = mod_loader::get(&resources_path, mod_loader_id)?;
	mod_loader.open_mod_folder(mod_id)
}

#[tauri::command]
#[specta::specta]
async fn start_game(game_id: &str, state: tauri::State<'_, AppState>) -> Result {
	get_game(game_id, &state)?.start()
}

#[tauri::command]
#[specta::specta]
async fn install_mod(
	mod_loader_id: &str,
	mod_id: &str,
	game_id: &str,
	state: tauri::State<'_, AppState>,
	handle: tauri::AppHandle,
) -> Result {
	let resources_path = paths::resources_path(&handle)?;

	let game = get_game(game_id, &state)?;

	let mod_loader = mod_loader::get(&resources_path, mod_loader_id)?;

	mod_loader.install_mod(&game, mod_id)?;

	get_full_state(handle, state).await?;

	Ok(())
}

#[tauri::command]
#[specta::specta]
async fn uninstall_mod(game_id: &str, mod_id: &str, state: tauri::State<'_, AppState>) -> Result {
	get_game(game_id, &state)?.uninstall_mod(mod_id)
}

#[tauri::command]
#[specta::specta]
async fn get_full_state(handle: tauri::AppHandle, state: tauri::State<'_, AppState>) -> Result {
	let resources_path = paths::resources_path(&handle)?;

	let mut full_state = FullState {
		discover_games: None,
		game_map: None,
		mod_loaders: None,
		owned_games: None,
	};

	let mod_loaders = mod_loader::get_data_map(&resources_path)
		.await
		.unwrap_or_default();

	let steam_dir = SteamDir::locate()?;
	let app_info = appinfo::read(steam_dir.path())?;

	full_state.game_map = Some(
		steam::installed_games::get(&steam_dir, &app_info, &mod_loaders)
			.await
			.unwrap_or_default(),
	);

	full_state.mod_loaders = Some(mod_loaders);

	handle.emit_all("update_state", full_state.clone())?;

	let (discover_games, owned_games) = future::join!(
		steam::discover_games::get(&app_info),
		steam::owned_games::get(&steam_dir, &app_info)
	)
	.await;

	full_state.discover_games = Some(discover_games.unwrap_or_default());
	full_state.owned_games = Some(owned_games.unwrap_or_default());

	if let Ok(mut write_guard) = state.full_state.lock() {
		*write_guard = Some(full_state.clone());
	}

	handle.emit_all("update_state", full_state)?;

	Ok(())
}

#[tauri::command]
#[specta::specta]
async fn delete_steam_appinfo_cache() -> Result {
	let steam_dir = SteamDir::locate()?;
	steam::appinfo::delete(steam_dir.path())
}

#[tauri::command]
#[specta::specta]
// This command is here just so tauri_specta exports these types.
// This should stop being needed once tauri_specta starts supporting events.
async fn dummy_command() -> Result<(Game, FullState)> {
	Err(Error::NotImplemented)
}

fn main() {
	// Since I'm making all exposed functions async, panics won't crash anything important, I think.
	// So I can just catch panics here and show a system message with the error.
	std::panic::set_hook(Box::new(|info| {
		println!("Panic: {info}");
		message(
			None::<&tauri::Window>,
			"Failed to execute command",
			info.to_string(),
		);
	}));

	set_up_tauri!(
		"../frontend/api/bindings.ts",
		AppState {
			full_state: Mutex::default(),
		},
		[
			dummy_command,
			get_full_state,
			open_game_folder,
			install_mod,
			uninstall_mod,
			open_game_mods_folder,
			start_game,
			open_mod_folder,
			delete_steam_appinfo_cache,
			open_mods_folder
		]
	);
}
