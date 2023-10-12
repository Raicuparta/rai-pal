// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![feature(future_join)]

use std::{
	collections::HashMap,
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
	discover_games::DiscoverGame,
	owned_games::OwnedGame,
};
use steamlocate::SteamDir;
use tauri::api::dialog::message;

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
	let state = state.full_state.lock().unwrap();
	let state = state
		.as_ref()
		.ok_or(Error::GameNotFound(game_id.to_string()))?;
	let game = state
		.game_map
		.get(game_id)
		.ok_or_else(|| Error::GameNotFound(game_id.to_string()))?;
	Ok(game.clone())
}

async fn get_state_data<TData, TFunction, TFunctionResult>(
	mutex: &Mutex<Option<TData>>,
	get_data: TFunction,
	ignore_cache: bool,
) -> Result<TData>
where
	TFunction: Fn() -> TFunctionResult + Send,
	TData: Clone + Send,
	TFunctionResult: Future<Output = Result<TData>> + Send,
{
	if !ignore_cache {
		if let Ok(mutex_guard) = mutex.lock() {
			if let Some(data) = &*mutex_guard {
				return Ok(data.clone());
			}
		}
	}

	let result = get_data();
	let data = result.await?;
	if let Ok(mut mutex_guard) = mutex.lock() {
		*mutex_guard = Some(data.clone());
	}

	Ok(data)
}

serializable_struct!(FullState {
	game_map: game::Map,
	owned_games: Vec<OwnedGame>,
	discover_games: Vec<DiscoverGame>,
	mod_loaders: mod_loader::DataMap,
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
async fn update_game_info(game_id: &str, state: tauri::State<'_, AppState>) -> Result<FullState> {
	if let Ok(read_guard) = state.full_state.lock() {
		let mut full_state = read_guard
			.as_ref()
			.ok_or(Error::GameNotFound(game_id.to_string()))?
			.clone();

		let game = full_state
			.game_map
			.get(game_id)
			.ok_or_else(|| Error::GameNotFound(game_id.to_string()))?;

		let game_copy = Game::new(
			&game.id,
			&game.name,
			game.discriminator.clone(),
			&game.executable.path,
			game.steam_launch.as_ref(),
			game.thumbnail_url.clone(),
		)
		.ok_or_else(|| Error::GameCopyFailed(game.id.clone()))?;

		full_state.game_map.insert(game_id.to_string(), game_copy);

		if let Ok(mut write_guard) = state.full_state.lock() {
			*write_guard = Some(full_state.clone());
		}

		return Ok(full_state.clone());
	}

	Err(Error::GameNotFound(game_id.to_string()))
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

	mod_loader.install_mod(&game, mod_id)
}

#[tauri::command]
#[specta::specta]
async fn uninstall_mod(game_id: &str, mod_id: &str, state: tauri::State<'_, AppState>) -> Result {
	get_game(game_id, &state)?.uninstall_mod(mod_id)
}

#[tauri::command]
#[specta::specta]
async fn get_full_state(
	handle: tauri::AppHandle,
	state: tauri::State<'_, AppState>,
) -> Result<FullState> {
	let resources_path = paths::resources_path(&handle)?;

	let mod_loaders = mod_loader::get_data_map(&resources_path)
		.await
		.unwrap_or_default();

	let (discover_games, game_map, owned_games) = future::join!(
		steam::discover_games::get(),
		steam::installed_games::get(&mod_loaders),
		steam::owned_games::get()
	)
	.await;

	let full_state = FullState {
		discover_games: discover_games.unwrap_or_default(),
		game_map: game_map.unwrap_or_default(),
		mod_loaders,
		owned_games: owned_games.unwrap_or_default(),
	};

	if let Ok(mut write_guard) = state.full_state.lock() {
		*write_guard = Some(full_state.clone());
	}

	Ok(full_state)
}

#[tauri::command]
#[specta::specta]
async fn delete_steam_appinfo_cache() -> Result {
	let steam_dir = SteamDir::locate()?;
	steam::appinfo::delete(steam_dir.path())
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
			get_full_state,
			open_game_folder,
			install_mod,
			uninstall_mod,
			open_game_mods_folder,
			start_game,
			open_mod_folder,
			update_game_info,
			delete_steam_appinfo_cache,
			open_mods_folder
		]
	);
}
