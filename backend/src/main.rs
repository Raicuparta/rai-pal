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
	id_lists::SteamGame,
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

serializable_struct!(LocalState {
	game_map: game::Map,
	mod_loaders: mod_loader::DataMap,
});

serializable_struct!(RemoteState {
	owned_games: Vec<OwnedGame>,
	discover_games: Vec<SteamGame>,
});

struct AppState {
	local: Mutex<Option<LocalState>>,
	remote: Mutex<Option<RemoteState>>,
}

fn get_game(game_id: &str, state: &tauri::State<'_, AppState>) -> Result<Game> {
	if let Ok(read_guard) = state.local.lock() {
		let local_state = read_guard
			.as_ref()
			.ok_or(Error::GameNotFound(game_id.to_string()))?;

		let game = local_state
			.game_map
			.get(game_id)
			.ok_or_else(|| Error::GameNotFound(game_id.to_string()))?;

		return Ok(game.clone());
	}

	Err(Error::GameNotFound(game_id.to_string()))
}

// Sends a signal to make the frontend request an app state refresh.
// I would have preferred to just send the state with the signal,
// but it seems like Tauri events are really slow for large data.
fn sync_local_state(handle: &tauri::AppHandle) -> Result {
	handle.emit_all("sync_local", ())?;

	Ok(())
}

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

	sync_local_state(&handle)?;

	Ok(())
}

#[tauri::command]
#[specta::specta]
async fn uninstall_mod(
	game_id: &str,
	mod_id: &str,
	state: tauri::State<'_, AppState>,
	handle: tauri::AppHandle,
) -> Result {
	get_game(game_id, &state)?.uninstall_mod(mod_id)?;

	sync_local_state(&handle)?;

	Ok(())
}

#[tauri::command]
#[specta::specta]
async fn get_local_state(
	handle: tauri::AppHandle,
	state: tauri::State<'_, AppState>,
) -> Result<LocalState> {
	let resources_path = paths::resources_path(&handle)?;

	let mod_loaders = mod_loader::get_data_map(&resources_path)
		.await
		.unwrap_or_default();

	let steam_dir = SteamDir::locate()?;
	let app_info = appinfo::read(steam_dir.path())?;

	let game_map = steam::installed_games::get(&steam_dir, &app_info, &mod_loaders)
		.await
		.unwrap_or_default();

	let local_state = LocalState {
		game_map,
		mod_loaders,
	};

	if let Ok(mut write_guard) = state.local.lock() {
		*write_guard = Some(local_state.clone());
	}

	Ok(local_state)
}

#[tauri::command]
#[specta::specta]
async fn get_remote_state(state: tauri::State<'_, AppState>) -> Result<RemoteState> {
	let steam_dir = SteamDir::locate()?;
	let app_info = appinfo::read(steam_dir.path())?;

	let (discover_games, owned_games) = Box::pin(future::join!(
		steam::discover_games::get(&app_info),
		steam::owned_games::get(&steam_dir, &app_info)
	))
	.await;

	let discover_games = discover_games.unwrap_or_default();
	let owned_games = owned_games.unwrap_or_default();

	let remote_state = RemoteState {
		owned_games,
		discover_games,
	};

	if let Ok(mut write_guard) = state.remote.lock() {
		*write_guard = Some(remote_state.clone());
	}

	Ok(remote_state)
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
async fn dummy_command() -> Result<Game> {
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
			local: Mutex::default(),
			remote: Mutex::default(),
		},
		[
			dummy_command,
			get_local_state,
			get_remote_state,
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
