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

serializable_enum!(SyncDataEvent {
	SyncInstalledGames,
	SyncOwnedGames,
	SyncDiscoverGames,
	SyncMods,
});

struct AppState {
	installed_games: Mutex<Option<game::Map>>,
	owned_games: Mutex<Option<Vec<OwnedGame>>>,
	discover_games: Mutex<Option<Vec<SteamGame>>>,
	mod_loaders: Mutex<Option<mod_loader::DataMap>>,
}

fn get_game(game_id: &str, state: &tauri::State<'_, AppState>) -> Result<Game> {
	if let Ok(read_guard) = state.installed_games.lock() {
		let installed_games = read_guard
			.as_ref()
			.ok_or(Error::GameNotFound(game_id.to_string()))?;

		let game = installed_games
			.get(game_id)
			.ok_or_else(|| Error::GameNotFound(game_id.to_string()))?;

		return Ok(game.clone());
	}

	Err(Error::GameNotFound(game_id.to_string()))
}

fn get_state_data<TData: Clone>(mutex: &Mutex<Option<TData>>) -> Result<TData> {
	match mutex.lock() {
		Ok(guard) => {
			let data = guard
				.as_ref()
				.ok_or(Error::FailedToGetStateData("Data is empty".to_string()))?;

			Ok(data.clone())
		}
		Err(err) => Err(Error::FailedToGetStateData(err.to_string())),
	}
}

#[tauri::command]
#[specta::specta]
async fn get_installed_games(state: tauri::State<'_, AppState>) -> Result<game::Map> {
	get_state_data(&state.installed_games)
}

#[tauri::command]
#[specta::specta]
async fn get_owned_games(state: tauri::State<'_, AppState>) -> Result<Vec<OwnedGame>> {
	get_state_data(&state.owned_games)
}

#[tauri::command]
#[specta::specta]
async fn get_discover_games(state: tauri::State<'_, AppState>) -> Result<Vec<SteamGame>> {
	get_state_data(&state.discover_games)
}

#[tauri::command]
#[specta::specta]
async fn get_mod_loaders(state: tauri::State<'_, AppState>) -> Result<mod_loader::DataMap> {
	get_state_data(&state.mod_loaders)
}

fn update_state<TData>(
	event: SyncDataEvent,
	data: TData,
	mutex: &Mutex<Option<TData>>,
	handle: &tauri::AppHandle,
) -> Result {
	if let Ok(mut mutex_guard) = mutex.lock() {
		*mutex_guard = Some(data);
	}

	// Sends a signal to make the frontend request an app state refresh.
	// I would have preferred to just send the state with the signal,
	// but it seems like Tauri events are really slow for large data.
	handle.emit_all(&event.to_string(), ())?;

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

	refresh_single_game(game_id, &state, &handle)?;

	Ok(())
}

fn refresh_single_game(
	game_id: &str,
	state: &tauri::State<'_, AppState>,
	handle: &tauri::AppHandle,
) -> Result {
	let mod_loaders = get_state_data(&state.mod_loaders)?;
	let mut installed_games = get_state_data(&state.installed_games)?;

	installed_games
		.get_mut(game_id)
		.ok_or_else(|| Error::GameNotFound(game_id.to_string()))?
		.refresh_mods(&mod_loaders);

	update_state(
		SyncDataEvent::SyncInstalledGames,
		installed_games,
		&state.installed_games,
		handle,
	)?;

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

	refresh_single_game(game_id, &state, &handle)?;

	Ok(())
}

#[tauri::command]
#[specta::specta]
async fn update_data(handle: tauri::AppHandle, state: tauri::State<'_, AppState>) -> Result {
	let resources_path = paths::resources_path(&handle)?;

	let mod_loaders = mod_loader::get_data_map(&resources_path)
		.await
		.unwrap_or_default();
	update_state(
		SyncDataEvent::SyncMods,
		mod_loaders.clone(),
		&state.mod_loaders,
		&handle,
	)?;

	let steam_dir = SteamDir::locate()?;
	let app_info = appinfo::read(steam_dir.path())?;

	let installed_games = steam::installed_games::get(&steam_dir, &app_info, &mod_loaders)
		.await
		.unwrap_or_default();
	update_state(
		SyncDataEvent::SyncInstalledGames,
		installed_games.clone(),
		&state.installed_games,
		&handle,
	)?;

	let (discover_games, owned_games) = Box::pin(future::join!(
		steam::discover_games::get(&app_info),
		steam::owned_games::get(&steam_dir, &app_info)
	))
	.await;

	let discover_games = discover_games.unwrap_or_default();
	update_state(
		SyncDataEvent::SyncDiscoverGames,
		discover_games,
		&state.discover_games,
		&handle,
	)?;

	let owned_games = owned_games.unwrap_or_default();
	update_state(
		SyncDataEvent::SyncOwnedGames,
		owned_games,
		&state.owned_games,
		&handle,
	)?;

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
async fn dummy_command() -> Result<(Game, SyncDataEvent)> {
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

	if let Err(specta_err) = specta::collect_types![
		dummy_command,
		update_data,
		get_installed_games,
		get_owned_games,
		get_discover_games,
		get_mod_loaders,
		open_game_folder,
		install_mod,
		uninstall_mod,
		open_game_mods_folder,
		start_game,
		open_mod_folder,
		delete_steam_appinfo_cache,
		open_mods_folder
	]
	.map(|types| {
		#[cfg(debug_assertions)]
		return tauri_specta::ts::export_with_cfg(
			types,
			specta::ts::ExportConfiguration::default()
				.bigint(specta::ts::BigIntExportBehavior::BigInt),
			"../frontend/api/bindings.ts",
		);
	}) {
		println!("Failed to generate TypeScript bindings: {specta_err}");
	}
	tauri::Builder::default()
		.plugin(tauri_plugin_window_state::Builder::default().build())
		.manage(AppState {
			installed_games: Mutex::default(),
			owned_games: Mutex::default(),
			discover_games: Mutex::default(),
			mod_loaders: Mutex::default(),
		})
		.setup(|app| {
			#[cfg(target_os = "linux")]
			{
				// This prevents/reduces the white flashbang on app start.
				// Unfortunately, it will still show the default window color for the system for a bit,
				// which can some times be white.
				if let Some(window) = app.get_window("main") {
					window.with_webview(|webview| {
						use webkit2gtk::traits::WebViewExt;
						let mut color = webview.inner().background_color();
						color.set_red(0.102);
						color.set_green(0.106);
						color.set_blue(0.118);
						webview.inner().set_background_color(&color);
					})?;
				}
			}
			Ok(())
		})
		.invoke_handler(tauri::generate_handler![
			dummy_command,
			update_data,
			get_installed_games,
			get_owned_games,
			get_discover_games,
			get_mod_loaders,
			open_game_folder,
			install_mod,
			uninstall_mod,
			open_game_mods_folder,
			start_game,
			open_mod_folder,
			delete_steam_appinfo_cache,
		])
		.run(tauri::generate_context!())
		.unwrap_or_else(|err| println!("Failed to run Tauri application: {err}"));
}
