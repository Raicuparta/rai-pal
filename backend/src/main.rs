// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![feature(future_join)]

use std::{
	collections::HashMap,
	path::PathBuf,
	sync::Mutex,
};

use events::{
	AppEvent,
	EventEmitter,
};
use installed_game::InstalledGame;
use mod_loaders::mod_loader::{
	self,
	ModLoaderActions,
};
use owned_game::OwnedGame;
use providers::{
	manual_provider,
	provider::{
		self,
		ProviderActions,
	},
};
use result::{
	Error,
	Result,
};
use steam::{
	appinfo,
	id_lists::SteamGame,
};
use steamlocate::SteamDir;
use tauri::{
	api::dialog::{
		blocking::FileDialogBuilder,
		message,
	},
	FileDropEvent,
	Manager,
	WindowEvent,
};

use crate::paths::file_name_without_extension;

mod events;
mod files;
mod game_engines;
mod game_executable;
mod game_mod;
mod installed_game;
mod macros;
mod mod_loaders;
mod owned_game;
mod paths;
mod providers;
mod result;
mod steam;
mod windows;

struct AppState {
	installed_games: Mutex<Option<installed_game::Map>>,
	owned_games: Mutex<Option<Vec<OwnedGame>>>,
	discover_games: Mutex<Option<Vec<SteamGame>>>,
	mod_loaders: Mutex<Option<mod_loader::DataMap>>,
}

fn get_game(game_id: &str, state: &tauri::State<'_, AppState>) -> Result<InstalledGame> {
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
async fn get_installed_games(state: tauri::State<'_, AppState>) -> Result<installed_game::Map> {
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
	event: AppEvent,
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
	handle.emit_event(event)?;

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
async fn start_game(
	game_id: &str,
	state: tauri::State<'_, AppState>,
	handle: tauri::AppHandle,
) -> Result {
	get_game(game_id, &state)?.start(&handle)
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
		AppEvent::SyncInstalledGames,
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
		AppEvent::SyncMods,
		mod_loaders.clone(),
		&state.mod_loaders,
		&handle,
	)?;

	let provider_map = provider::get_map();

	let steam_dir = SteamDir::locate()?;
	let app_info = appinfo::read(steam_dir.path())?;

	let installed_games: HashMap<String, InstalledGame> = provider_map
		.iter()
		.flat_map(
			|(_, provider)| match provider.get_installed_games(&mod_loaders) {
				Ok(games) => games,
				Err(err) => {
					// TODO properly handle these errors message to frontend.
					println!("Error gettings installed games for provider: {}", err);
					HashMap::new()
				}
			},
		)
		.collect();

	update_state(
		AppEvent::SyncInstalledGames,
		installed_games.clone(),
		&state.installed_games,
		&handle,
	)?;

	let (discover_games_result, owned_games_result) = futures::future::join(
		steam::discover_games::get(&app_info),
		futures::future::join_all(
			provider_map
				.values()
				.map(provider::ProviderActions::get_owned_games),
		),
	)
	.await;

	let owned_games: Vec<OwnedGame> = owned_games_result
		.into_iter()
		.flat_map(result::Result::unwrap_or_default)
		.collect();

	let discover_games = discover_games_result.unwrap_or_default();
	update_state(
		AppEvent::SyncDiscoverGames,
		discover_games,
		&state.discover_games,
		&handle,
	)?;

	update_state(
		AppEvent::SyncOwnedGames,
		owned_games,
		&state.owned_games,
		&handle,
	)?;

	Ok(())
}

#[tauri::command]
#[specta::specta]
async fn add_game(
	path: &str,
	state: tauri::State<'_, AppState>,
	handle: tauri::AppHandle,
) -> Result {
	manual_provider::add_game(&PathBuf::from(path))?;
	// TODO: add only new game, instead of refreshing entire list.
	update_data(handle, state).await?;

	Ok(())
}

#[tauri::command]
#[specta::specta]
// This command is here just so tauri_specta exports these types.
// This should stop being needed once tauri_specta starts supporting events.
async fn dummy_command() -> Result<(InstalledGame, AppEvent)> {
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

	let tauri_builder = tauri::Builder::default()
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
					window.set_title(&format!("Rai Pal {}", env!("CARGO_PKG_VERSION")))?;

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
		});

	let (tauri_builder, types_result) = set_up_api!(
		tauri_builder,
		[
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
			open_mods_folder,
			add_game,
		]
	);

	match types_result {
		Ok(types) => {
			#[cfg(debug_assertions)]
			if let Err(err) = tauri_specta::ts::export_with_cfg(
				types,
				specta::ts::ExportConfiguration::default()
					.bigint(specta::ts::BigIntExportBehavior::BigInt),
				"../frontend/api/bindings.ts",
			) {
				println!("Failed to generate TypeScript bindings: {err}");
			}
		}
		Err(err) => {
			println!("Failed to generate api bindings: {err}");
		}
	}
	tauri_builder
		.run(tauri::generate_context!())
		.unwrap_or_else(|err| println!("Failed to run Tauri application: {err}"));
}
