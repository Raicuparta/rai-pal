// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{
	future::Future,
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
mod game_mod;
mod macros;
mod mod_loaders;
mod paths;
mod result;
mod steam;

struct AppState {
	game_map: Mutex<Option<game::Map>>,
	owned_games: Mutex<Option<Vec<OwnedGame>>>,
	discover_games: Mutex<Option<Vec<DiscoverGame>>>,
	mod_loaders: Mutex<Option<mod_loader::DataMap>>,
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

#[tauri::command]
#[specta::specta]
async fn get_game_map(state: tauri::State<'_, AppState>, ignore_cache: bool) -> Result<game::Map> {
	get_state_data(&state.game_map, steam::installed_games::get, ignore_cache).await
}

#[tauri::command]
#[specta::specta]
async fn get_owned_games(
	state: tauri::State<'_, AppState>,
	ignore_cache: bool,
) -> Result<Vec<OwnedGame>> {
	get_state_data(&state.owned_games, steam::owned_games::get, ignore_cache).await
}

#[tauri::command]
#[specta::specta]
async fn get_mod_loaders(
	state: tauri::State<'_, AppState>,
	ignore_cache: bool,
	handle: tauri::AppHandle,
) -> Result<mod_loader::DataMap> {
	let resources_path = paths::resources_path(&handle)?;

	get_state_data(
		&state.mod_loaders,
		|| mod_loader::get_data_map(&resources_path),
		ignore_cache,
	)
	.await
}

#[tauri::command]
#[specta::specta]
async fn get_discover_games(
	state: tauri::State<'_, AppState>,
	ignore_cache: bool,
) -> Result<Vec<DiscoverGame>> {
	get_state_data(
		&state.discover_games,
		steam::discover_games::get,
		ignore_cache,
	)
	.await
}

#[tauri::command]
#[specta::specta]
async fn open_game_folder(game_id: &str, state: tauri::State<'_, AppState>) -> Result {
	let game_map = get_game_map(state, false).await?;

	let game = game_map
		.get(game_id)
		.ok_or_else(|| Error::GameNotFound(game_id.to_string()))?;

	game.open_game_folder()
}

#[tauri::command]
#[specta::specta]
async fn open_game_mods_folder(game_id: &str, state: tauri::State<'_, AppState>) -> Result {
	let game_map = get_game_map(state, false).await?;
	let game = game_map
		.get(game_id)
		.ok_or_else(|| Error::GameNotFound(game_id.to_string()))?;

	game.open_mods_folder()
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
	let game_map = get_game_map(state, false).await?;
	let game = game_map
		.get(game_id)
		.ok_or_else(|| Error::GameNotFound(game_id.to_string()))?;

	game.start()
}

#[tauri::command]
#[specta::specta]
async fn update_game_info(game_id: &str, state: tauri::State<'_, AppState>) -> Result<game::Map> {
	let mut game_map = get_game_map(state, false).await?;
	let game = game_map
		.get(game_id)
		.ok_or_else(|| Error::GameNotFound(game_id.to_string()))?;

	let game_copy = Game::new(
		&game.id,
		&game.name,
		game.discriminator.clone(),
		&game.full_path,
		game.steam_launch.as_ref(),
		game.thumbnail_url.clone(),
	)
	.ok_or_else(|| Error::GameCopyFailed(game.id.clone()))?;

	game_map.insert(game.id.clone(), game_copy);

	Ok(game_map)
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

	let game_map = get_game_map(state, false).await?;
	let game = game_map
		.get(game_id)
		.ok_or_else(|| Error::GameNotFound(game_id.to_string()))?;

	let mod_loader = mod_loader::get(&resources_path, mod_loader_id)?;

	mod_loader.install_mod(game, mod_id)
}

#[tauri::command]
#[specta::specta]
async fn uninstall_mod(game_id: &str, mod_id: &str, state: tauri::State<'_, AppState>) -> Result {
	let game_map = get_game_map(state, false).await?;
	let game = game_map
		.get(game_id)
		.ok_or_else(|| Error::GameNotFound(game_id.to_string()))?;

	game.uninstall_mod(mod_id)
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
			game_map: Mutex::default(),
			owned_games: Mutex::default(),
			discover_games: Mutex::default(),
			mod_loaders: Mutex::default(),
		},
		[
			get_game_map,
			get_owned_games,
			open_game_folder,
			get_mod_loaders,
			install_mod,
			uninstall_mod,
			open_game_mods_folder,
			start_game,
			open_mod_folder,
			update_game_info,
			delete_steam_appinfo_cache,
			get_discover_games,
			open_mods_folder
		]
	);
}
