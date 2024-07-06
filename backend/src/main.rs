// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
// There's some weird tauri thing making this clippy error show up everywhere.
#![allow(clippy::used_underscore_binding)]

use std::{collections::HashMap, path::PathBuf, sync::Mutex};

use app_state::{AppState, DataValue, StateData, StatefulHandle};
use events::EventEmitter;
use local_mod::LocalMod;
use log::error;
use maps::TryGettable;
use mod_loaders::mod_loader::{self, ModLoaderActions};
use paths::{hash_path, normalize_path};
use providers::{
	manual_provider,
	provider::{self, ProviderActions},
	provider_command::ProviderCommandAction,
};
use result::{Error, Result};
use steamlocate::SteamDir;
use tauri::{AppHandle, Manager};
use tauri_plugin_log::{Target, TargetKind};

mod analytics;
mod app_state;
mod app_type;
mod debug;
mod events;
mod files;
mod game_engines;
mod game_executable;
mod game_mod;
mod game_mode;
mod installed_game;
mod local_mod;
mod macros;
mod maps;
mod mod_loaders;
mod mod_manifest;
mod operating_systems;
mod owned_game;
mod paths;
mod pc_gaming_wiki;
mod providers;
mod remote_game;
mod remote_mod;
mod result;
mod steam;
mod windows;

#[tauri::command]
#[specta::specta]
async fn get_installed_games(handle: AppHandle) -> Result<installed_game::Map> {
	handle.app_state().installed_games.get_data()
}

#[tauri::command]
#[specta::specta]
async fn get_owned_games(handle: AppHandle) -> Result<owned_game::Map> {
	handle.app_state().owned_games.get_data()
}

#[tauri::command]
#[specta::specta]
async fn get_mod_loaders(handle: AppHandle) -> Result<mod_loader::DataMap> {
	mod_loader::get_data_map(&handle.app_state().mod_loaders.get_data()?)
}

#[tauri::command]
#[specta::specta]
async fn get_local_mods(handle: AppHandle) -> Result<local_mod::Map> {
	handle.app_state().local_mods.get_data()
}

#[tauri::command]
#[specta::specta]
async fn get_remote_mods(handle: AppHandle) -> Result<remote_mod::Map> {
	handle.app_state().remote_mods.get_data()
}

#[tauri::command]
#[specta::specta]
async fn get_remote_games(handle: AppHandle) -> Result<remote_game::Map> {
	handle.app_state().remote_games.get_data()
}

fn update_state<TData, TEvent>(
	event: TEvent,
	data: TData,
	mutex: &Mutex<Option<TData>>,
	handle: &AppHandle,
) where
	TEvent: tauri_specta::Event + std::clone::Clone + serde::Serialize,
{
	if let Ok(mut mutex_guard) = mutex.lock() {
		*mutex_guard = Some(data);
	}

	// Sends a signal to make the frontend request an app state refresh.
	// I would have preferred to just send the state with the signal,
	// but it seems like Tauri events are really slow for large data.
	handle.emit_safe(event);
}

#[tauri::command]
#[specta::specta]
async fn open_game_folder(game_id: &str, handle: AppHandle) -> Result {
	handle
		.app_state()
		.installed_games
		.try_get(game_id)?
		.open_game_folder()
}

#[tauri::command]
#[specta::specta]
async fn open_game_mods_folder(game_id: &str, handle: AppHandle) -> Result {
	handle
		.app_state()
		.installed_games
		.try_get(game_id)?
		.open_mods_folder()
}

#[tauri::command]
#[specta::specta]
async fn open_mods_folder() -> Result {
	Ok(open::that_detached(paths::installed_mods_path()?)?)
}

#[tauri::command]
#[specta::specta]
async fn open_mod_folder(mod_id: &str, handle: AppHandle) -> Result {
	handle.app_state().local_mods.try_get(mod_id)?.open_folder()
}

#[tauri::command]
#[specta::specta]
async fn open_mod_loader_folder(mod_loader_id: &str, handle: AppHandle) -> Result {
	handle
		.app_state()
		.mod_loaders
		.try_get(mod_loader_id)?
		.open_folder()
}

#[tauri::command]
#[specta::specta]
async fn download_mod(mod_id: &str, handle: AppHandle) -> Result {
	let state = handle.app_state();
	let remote_mod = state.remote_mods.try_get(mod_id)?;
	let mod_loaders = state.mod_loaders.get_data()?;

	mod_loaders
		.try_get(&remote_mod.common.loader_id)?
		.download_mod(&remote_mod)
		.await?;

	refresh_local_mods(&mod_loaders, &handle);

	Ok(())
}

#[tauri::command]
#[specta::specta]
async fn delete_mod(mod_id: &str, handle: AppHandle) -> Result {
	let state = handle.app_state();
	let local_mod = state.local_mods.try_get(mod_id)?;
	let mod_loaders = state.mod_loaders.get_data()?;

	mod_loaders
		.try_get(&local_mod.common.loader_id)?
		.delete_mod(&local_mod)?;

	refresh_local_mods(&mod_loaders, &handle);

	Ok(())
}

#[tauri::command]
#[specta::specta]
async fn start_game(game_id: &str, handle: AppHandle) -> Result {
	handle
		.app_state()
		.installed_games
		.try_get(game_id)?
		.start()?;

	handle.emit_safe(events::ExecutedProviderCommand);

	Ok(())
}

#[tauri::command]
#[specta::specta]
async fn start_game_exe(game_id: &str, handle: AppHandle) -> Result {
	handle
		.app_state()
		.installed_games
		.try_get(game_id)?
		.start_exe()
}

#[tauri::command]
#[specta::specta]
async fn install_mod(game_id: &str, mod_id: &str, handle: AppHandle) -> Result {
	let state = handle.app_state();

	let mut installed_games = state.installed_games.get_data()?;
	let game = installed_games.try_get_mut(game_id)?;
	let mod_loaders = state.mod_loaders.get_data()?;

	let local_mod = refresh_and_get_local_mod(mod_id, &mod_loaders, &handle).await?;

	let mod_loader = mod_loaders.try_get(&local_mod.common.loader_id)?;

	// Uninstall mod if it already exists, in case there are conflicting leftover files when updating.
	mod_loader.uninstall_mod(game, &local_mod).await?;

	mod_loader.install_mod(game, &local_mod).await?;

	refresh_game_mods_and_exe(&game.id, &handle)?;

	analytics::send_event(analytics::Event::InstallOrRunMod, mod_id).await;

	Ok(())
}

#[tauri::command]
#[specta::specta]
async fn run_runnable_without_game(mod_id: &str, handle: AppHandle) -> Result {
	let state = handle.app_state();

	let mod_loaders = state.mod_loaders.get_data()?;
	let local_mod = refresh_and_get_local_mod(mod_id, &mod_loaders, &handle).await?;
	let mod_loader = mod_loaders.try_get(&local_mod.common.loader_id)?;

	mod_loader.run_without_game(&local_mod).await?;

	analytics::send_event(analytics::Event::InstallOrRunMod, mod_id).await;

	Ok(())
}

#[tauri::command]
#[specta::specta]
async fn configure_mod(game_id: &str, mod_id: &str, handle: AppHandle) -> Result {
	let state = handle.app_state();

	let mut installed_games = state.installed_games.get_data()?;
	let game = installed_games.try_get_mut(game_id)?;
	let mod_loaders = state.mod_loaders.get_data()?;
	let local_mod = refresh_and_get_local_mod(mod_id, &mod_loaders, &handle).await?;

	let mod_loader = mod_loaders.try_get(&local_mod.common.loader_id)?;

	mod_loader.configure_mod(game, &local_mod)?;

	Ok(())
}

#[tauri::command]
#[specta::specta]
async fn open_installed_mod_folder(game_id: &str, mod_id: &str, handle: AppHandle) -> Result {
	let state = handle.app_state();

	let mut installed_games = state.installed_games.get_data()?;
	let game = installed_games.try_get_mut(game_id)?;
	let mod_loaders = state.mod_loaders.get_data()?;
	let local_mod = refresh_and_get_local_mod(mod_id, &mod_loaders, &handle).await?;

	let mod_loader = mod_loaders.try_get(&local_mod.common.loader_id)?;

	mod_loader.open_installed_mod_folder(game, &local_mod)?;

	Ok(())
}

fn refresh_game_mods_and_exe(game_id: &str, handle: &AppHandle) -> Result {
	let state = handle.app_state();

	let mut installed_games = state.installed_games.get_data()?;

	let game = installed_games.try_get_mut(game_id)?;

	game.refresh_installed_mods();
	game.refresh_executable()?;

	update_state(
		events::SyncInstalledGames(installed_games.clone()),
		installed_games,
		&handle.app_state().installed_games,
		handle,
	);

	Ok(())
}

#[tauri::command]
#[specta::specta]
async fn refresh_game(game_id: &str, handle: AppHandle) -> Result {
	refresh_game_mods_and_exe(game_id, &handle)
}

#[tauri::command]
#[specta::specta]
async fn uninstall_mod(game_id: &str, mod_id: &str, handle: AppHandle) -> Result {
	let state = handle.app_state();
	let mut installed_games = state.installed_games.get_data()?;
	let game = installed_games.try_get_mut(game_id)?;

	let mod_loaders = state.mod_loaders.get_data()?;

	let local_mod = refresh_and_get_local_mod(mod_id, &mod_loaders, &handle).await?;

	let mod_loader = mod_loaders.try_get(&local_mod.common.loader_id)?;

	// Uninstall mod if it already exists, in case there are conflicting leftover files when updating.
	mod_loader.uninstall_mod(game, &local_mod).await?;

	refresh_game_mods_and_exe(&game.id, &handle)?;

	Ok(())
}

#[tauri::command]
#[specta::specta]
async fn uninstall_all_mods(game_id: &str, handle: AppHandle) -> Result {
	let state = handle.app_state();
	let game = state.installed_games.try_get(game_id)?;
	game.uninstall_all_mods()?;

	refresh_game_mods_and_exe(&game.id, &handle)?;

	Ok(())
}

fn refresh_local_mods(mod_loaders: &mod_loader::Map, handle: &AppHandle) -> local_mod::Map {
	let local_mods: HashMap<_, _> = mod_loaders
		.values()
		.filter_map(|mod_loader| {
			mod_loader.get_local_mods().ok() // don't swallow error.
		})
		.flatten()
		.collect();

	update_state(
		events::SyncLocalMods(local_mods.clone()),
		local_mods.clone(),
		&handle.app_state().local_mods,
		handle,
	);

	local_mods
}

async fn refresh_remote_mods(mod_loaders: &mod_loader::Map, handle: &AppHandle) -> remote_mod::Map {
	let mut remote_mods = remote_mod::Map::default();

	for mod_loader in mod_loaders.values() {
		for (mod_id, remote_mod) in mod_loader
			.get_remote_mods(|error| {
				handle.emit_error(format!("Failed to get remote mods: {error}"));
			})
			.await
		{
			remote_mods.insert(mod_id.clone(), remote_mod.clone());
		}
	}

	update_state(
		events::SyncRemoteMods(remote_mods.clone()),
		remote_mods.clone(),
		&handle.app_state().remote_mods,
		handle,
	);

	remote_mods
}

async fn refresh_and_get_local_mod(
	mod_id: &str,
	mod_loaders: &mod_loader::Map,
	handle: &AppHandle,
) -> Result<LocalMod> {
	let local_mods = {
		let state = handle.app_state();

		let state_local_mods = state.local_mods.get_data()?;
		if state_local_mods.contains_key(mod_id) {
			state_local_mods
		} else {
			// Local mod wasn't in app state,
			// so let's sync app state to local files in case some file was manually changed.
			let disk_local_mods = refresh_local_mods(mod_loaders, handle);

			if state_local_mods.contains_key(mod_id) {
				disk_local_mods
			} else {
				let remote_mod = state.remote_mods.try_get(mod_id)?;
				let mod_loader = mod_loaders.try_get(&remote_mod.common.loader_id)?;

				if remote_mod.data.latest_version.is_some() {
					// If local mod still can't be found on disk,
					// we try to download it from the database.
					mod_loader
						.download_mod(&state.remote_mods.try_get(mod_id)?)
						.await?;
				} else {
					// If downloading from the database isn't possible,
					// we just open the mod loader folder so the user can install it themselves.
					mod_loader.open_folder()?;
				}

				refresh_local_mods(mod_loaders, handle)
			}
		}
	};

	local_mods.try_get(mod_id).cloned()
}

async fn update_installed_games(handle: AppHandle, provider_map: provider::Map) {
	let installed_games: HashMap<_, _> = provider_map
		.iter()
		.flat_map(|(provider_id, provider)| {
			let installed_games = provider.get_installed_games(|game| {
				handle.emit_safe(events::FoundInstalledGame(game));
			});

			match installed_games {
				Ok(games) => games,
				Err(err) => {
					error!("Error getting installed games for provider ({provider_id}): {err}");
					Vec::default()
				}
			}
		})
		.map(|game| (game.id.clone(), game))
		.collect();

	update_state(
		events::SyncInstalledGames(installed_games.clone()),
		installed_games,
		&handle.app_state().installed_games,
		&handle,
	);
}

async fn update_owned_games(handle: AppHandle, provider_map: provider::Map) {
	let owned_games: owned_game::Map = provider_map
		.iter()
		.flat_map(|(provider_id, provider)| {
			provider
				.get_owned_games(|game| {
					handle.emit_safe(events::FoundOwnedGame(game));
				})
				.unwrap_or_else(|err| {
					error!("Failed to get owned games for provider '{provider_id}'. Error: {err}");
					Vec::default()
				})
		})
		.map(|owned_game| (owned_game.id.clone(), owned_game))
		.collect();

	update_state(
		events::SyncOwnedGames(owned_games.clone()),
		owned_games,
		&handle.app_state().owned_games,
		&handle,
	);
}

async fn update_remote_games(handle: AppHandle, provider_map: provider::Map) {
	let remote_games: remote_game::Map =
		futures::future::join_all(provider_map.values().map(|provider| {
			provider.get_remote_games(|remote_game| {
				handle.emit_safe(events::FoundRemoteGame(remote_game));
			})
		}))
		.await
		.into_iter()
		.flat_map(|result| {
			result.unwrap_or_else(|err| {
				error!("Failed to get remote games for a provider: {err}");
				Vec::default()
			})
		})
		.map(|remote_game| (remote_game.id.clone(), remote_game))
		.collect();

	update_state(
		events::SyncRemoteGames(remote_games.clone()),
		remote_games,
		&handle.app_state().remote_games,
		&handle,
	);
}

async fn update_mods(handle: AppHandle, resources_path: PathBuf) {
	let mod_loaders = mod_loader::get_map(&resources_path);

	match mod_loader::get_data_map(&mod_loaders) {
		Ok(mod_loaders_data_map) => {
			update_state(
				events::SyncModLoaders(mod_loaders_data_map),
				mod_loaders.clone(),
				&handle.app_state().mod_loaders,
				&handle,
			);

			refresh_local_mods(&mod_loaders, &handle);
			refresh_remote_mods(&mod_loaders, &handle).await;
		}
		Err(err) => {
			handle.emit_error(format!("Failed to get mod loaders: {err}"));
		}
	}
}

#[tauri::command]
#[specta::specta]
async fn update_data(handle: AppHandle) -> Result {
	let resources_path = paths::resources_path(&handle)?;

	let provider_map = provider::get_map();

	let results = futures::future::join_all([
		tokio::spawn(update_installed_games(handle.clone(), provider_map.clone())),
		tokio::spawn(update_owned_games(handle.clone(), provider_map.clone())),
		tokio::spawn(update_remote_games(handle.clone(), provider_map)),
		tokio::spawn(update_mods(handle.clone(), resources_path)),
	])
	.await;

	for result in results {
		if let Err(err) = result {
			handle.emit_error(format!("Error updating data: {err}"));
		}
	}

	Ok(())
}

#[tauri::command]
#[specta::specta]
async fn add_game(path: PathBuf, handle: AppHandle) -> Result {
	let state = handle.app_state();
	let normalized_path = normalize_path(&path);
	let game_id = hash_path(&normalized_path);

	if state.installed_games.try_get(&game_id).is_ok() {
		return Err(Error::GameAlreadyAdded(normalized_path));
	}

	let game = manual_provider::add_game(&normalized_path)?;
	let game_name = game.name.clone();

	let mut installed_games = state.installed_games.get_data()?.clone();
	installed_games.insert(game.id.clone(), game);

	update_state(
		events::SyncInstalledGames(installed_games.clone()),
		installed_games,
		&state.installed_games,
		&handle,
	);

	handle.emit_safe(events::GameAdded(game_name.clone()));

	analytics::send_event(analytics::Event::ManuallyAddGame, &game_name).await;

	Ok(())
}

#[tauri::command]
#[specta::specta]
async fn remove_game(game_id: &str, handle: AppHandle) -> Result {
	let state = handle.app_state();
	let game = state.installed_games.try_get(game_id)?;
	manual_provider::remove_game(&game.executable.path)?;

	let mut installed_games = state.installed_games.get_data()?;
	installed_games.remove(game_id);

	update_state(
		events::SyncInstalledGames(installed_games.clone()),
		installed_games,
		&handle.app_state().installed_games,
		&handle,
	);

	handle.emit_safe(events::GameRemoved(game.name));

	Ok(())
}

#[tauri::command]
#[specta::specta]
async fn run_provider_command(
	owned_game_id: &str,
	command_action: ProviderCommandAction,
	handle: AppHandle,
) -> Result {
	handle
		.app_state()
		.owned_games
		.try_get(owned_game_id)?
		.provider_commands
		.try_get(&command_action)?
		.run()?;

	handle.emit_safe(events::ExecutedProviderCommand);

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
async fn frontend_ready() -> Result {
	analytics::send_event(analytics::Event::StartApp, "").await;

	Ok(())
}

#[tauri::command]
#[specta::specta]
async fn open_logs_folder() -> Result {
	paths::open_logs_folder()?;

	Ok(())
}

fn main() {
	// Since I'm making all exposed functions async, panics won't crash anything important, I think.
	// So I can just catch panics here and show a system message with the error.
	#[cfg(target_os = "windows")]
	std::panic::set_hook(Box::new(|info| {
		windows::error_dialog(&info.to_string());
		// TODO handle Linux.
	}));

	let (invoke_handler, register_events) = {
		let builder = tauri_specta::ts::builder()
			.config(
				specta::ts::ExportConfig::default()
					.bigint(specta::ts::BigIntExportBehavior::BigInt),
			)
			.commands(tauri_specta::collect_commands![
				update_data,
				get_installed_games,
				get_owned_games,
				get_mod_loaders,
				open_game_folder,
				install_mod,
				configure_mod,
				open_installed_mod_folder,
				uninstall_mod,
				uninstall_all_mods,
				open_game_mods_folder,
				start_game,
				start_game_exe,
				open_mod_folder,
				download_mod,
				run_runnable_without_game,
				delete_mod,
				open_mods_folder,
				add_game,
				remove_game,
				delete_steam_appinfo_cache,
				frontend_ready,
				get_local_mods,
				get_remote_mods,
				get_remote_games,
				open_mod_loader_folder,
				refresh_game,
				open_logs_folder,
				run_provider_command,
			])
			.events(events::collect_events());

		#[cfg(debug_assertions)]
		let builder = builder.path("../frontend/api/bindings.ts");

		builder.build().unwrap_or_else(|err| {
			error!("Failed to generate TypeScript bindings: {err}");
			std::process::exit(1);
		})
	};

	tauri::Builder::default()
		.plugin(tauri_plugin_window_state::Builder::default().build())
		.plugin(
			tauri_plugin_log::Builder::new()
				.level(log::LevelFilter::Info)
				.targets([
					// TODO: check if all of these are working.
					Target::new(TargetKind::Stdout),
					Target::new(paths::logs_path().map_or(
						TargetKind::LogDir { file_name: None },
						|logs_path| TargetKind::Folder {
							path: logs_path,
							file_name: None,
						},
					)),
				])
				.build(),
		)
		.manage(AppState {
			installed_games: Mutex::default(),
			owned_games: Mutex::default(),
			remote_games: Mutex::default(),
			mod_loaders: Mutex::default(),
			local_mods: Mutex::default(),
			remote_mods: Mutex::default(),
		})
		.setup(|app| {
			register_events(app);

			if let Some(window) = app.get_webview_window("main") {
				window.set_title(&format!("Rai Pal {}", env!("CARGO_PKG_VERSION")))?;
			}

			Ok(())
		})
		.invoke_handler(invoke_handler)
		.run(tauri::generate_context!())
		.unwrap_or_else(|error| {
			#[cfg(target_os = "windows")]
			if let tauri::Error::Runtime(tauri_runtime::Error::CreateWebview(webview_error)) = error
			{
				windows::webview_error_dialog(&webview_error.to_string());
				return;
			}
			#[cfg(target_os = "windows")]
			windows::error_dialog(&error.to_string());
			// TODO handle Linux.
		});

	// match types_result {
	// 	Ok(types) => {
	// 		#[cfg(debug_assertions)]
	// 		if let Err(err) = tauri_specta::ts::export_with_cfg(
	// 			types,
	// 			specta::ts::ExportConfiguration::default()
	// 				.bigint(specta::ts::BigIntExportBehavior::BigInt),
	// 			"../frontend/api/bindings.ts",
	// 		) {
	// 			error!("Failed to generate TypeScript bindings: {err}");
	// 		}
	// 	}
	// 	Err(err) => {
	// 		error!("Failed to generate api bindings: {err}");
	// 	}
	// }
}
