// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
// There's some weird tauri thing making this clippy error show up everywhere.
#![allow(clippy::used_underscore_binding)]
// Command stuff needs to be async so I can spawn tasks.
#![allow(clippy::unused_async)]
// This is here because tauri specta would go over whatever the default limit is.
#![recursion_limit = "256"]

use std::{collections::HashMap, path::PathBuf, sync::Mutex};

use app_state::{AppState, DataValue, StateData, StatefulHandle};
use events::EventEmitter;
use log::error;
use rai_pal_core::installed_game::InstalledGame;
use rai_pal_core::local_mod::{self, LocalMod};
use rai_pal_core::maps::TryGettable;
use rai_pal_core::mod_loaders::mod_loader::{self, ModLoaderActions};
use rai_pal_core::owned_game::OwnedGame;
use rai_pal_core::paths::{self, normalize_path};
use rai_pal_core::providers::provider::ProviderId;
use rai_pal_core::providers::{
	manual_provider,
	provider::{self, ProviderActions},
	provider_command::ProviderCommandAction,
};
use rai_pal_core::remote_games::{self};
use rai_pal_core::result::{Error, Result};
#[cfg(target_os = "windows")]
use rai_pal_core::windows;
use rai_pal_core::{analytics, remote_mod, steam};
use specta_typescript::{BigIntExportBehavior, Typescript};
use tauri::path::BaseDirectory;
use tauri::{AppHandle, Manager};
use tauri_plugin_log::{Target, TargetKind};
use tauri_specta::Builder;

mod app_state;
mod events;

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

fn update_state<TData>(data: TData, mutex: &Mutex<Option<TData>>) {
	if let Ok(mut mutex_guard) = mutex.lock() {
		*mutex_guard = Some(data);
	}
}

#[tauri::command]
#[specta::specta]
async fn open_game_folder(installed_game: InstalledGame) -> Result {
	installed_game.open_game_folder()
}

#[tauri::command]
#[specta::specta]
async fn open_game_mods_folder(installed_game: InstalledGame) -> Result {
	installed_game.open_mods_folder()
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
async fn start_game(installed_game: InstalledGame, handle: AppHandle) -> Result {
	installed_game.start()?;

	handle.emit_safe(events::ExecutedProviderCommand);

	Ok(())
}

#[tauri::command]
#[specta::specta]
async fn start_game_exe(installed_game: InstalledGame) -> Result {
	installed_game.start_exe()
}

#[tauri::command]
#[specta::specta]
async fn install_mod(installed_game: InstalledGame, mod_id: &str, handle: AppHandle) -> Result {
	let state = handle.app_state();

	let mod_loaders = state.mod_loaders.get_data()?;

	let local_mod = refresh_and_get_local_mod(mod_id, &mod_loaders, &handle).await?;

	let mod_loader = mod_loaders.try_get(&local_mod.common.loader_id)?;

	// Uninstall mod if it already exists, in case there are conflicting leftover files when updating.
	mod_loader
		.uninstall_mod(&installed_game, &local_mod)
		.await?;

	mod_loader.install_mod(&installed_game, &local_mod).await?;

	refresh_game_mods_and_exe(&installed_game, &handle)?;

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
async fn configure_mod(installed_game: InstalledGame, mod_id: &str, handle: AppHandle) -> Result {
	let state = handle.app_state();

	let mod_loaders = state.mod_loaders.get_data()?;
	let local_mod = refresh_and_get_local_mod(mod_id, &mod_loaders, &handle).await?;

	let mod_loader = mod_loaders.try_get(&local_mod.common.loader_id)?;

	mod_loader.configure_mod(&installed_game, &local_mod)?;

	Ok(())
}

#[tauri::command]
#[specta::specta]
async fn open_installed_mod_folder(
	installed_game: InstalledGame,
	mod_id: &str,
	handle: AppHandle,
) -> Result {
	let state = handle.app_state();

	let mod_loaders = state.mod_loaders.get_data()?;
	let local_mod = refresh_and_get_local_mod(mod_id, &mod_loaders, &handle).await?;

	let mod_loader = mod_loaders.try_get(&local_mod.common.loader_id)?;

	mod_loader.open_installed_mod_folder(&installed_game, &local_mod)?;

	Ok(())
}

fn refresh_game_mods_and_exe(installed_game: &InstalledGame, handle: &AppHandle) -> Result {
	let mut new_game = installed_game.clone();
	new_game.refresh_installed_mods();
	new_game.refresh_executable()?;

	handle.emit_safe(events::FoundInstalledGame(new_game));

	Ok(())
}

#[tauri::command]
#[specta::specta]
async fn refresh_game(installed_game: InstalledGame, handle: AppHandle) -> Result {
	refresh_game_mods_and_exe(&installed_game, &handle)
}

#[tauri::command]
#[specta::specta]
async fn uninstall_mod(installed_game: InstalledGame, mod_id: &str, handle: AppHandle) -> Result {
	let state = handle.app_state();

	let mod_loaders = state.mod_loaders.get_data()?;

	let local_mod = refresh_and_get_local_mod(mod_id, &mod_loaders, &handle).await?;

	let mod_loader = mod_loaders.try_get(&local_mod.common.loader_id)?;

	// Uninstall mod if it already exists, in case there are conflicting leftover files when updating.
	mod_loader
		.uninstall_mod(&installed_game, &local_mod)
		.await?;

	refresh_game_mods_and_exe(&installed_game, &handle)?;

	Ok(())
}

#[tauri::command]
#[specta::specta]
async fn uninstall_all_mods(installed_game: InstalledGame, handle: AppHandle) -> Result {
	installed_game.uninstall_all_mods()?;

	refresh_game_mods_and_exe(&installed_game, &handle)?;

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

	handle.emit_safe(events::SyncLocalMods(local_mods.clone()));

	update_state(local_mods.clone(), &handle.app_state().local_mods);

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

	handle.emit_safe(events::SyncRemoteMods(remote_mods.clone()));

	update_state(remote_mods.clone(), &handle.app_state().remote_mods);

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

fn installed_game_callback(handle: AppHandle) -> impl Fn(InstalledGame) {
	move |game: InstalledGame| {
		handle.emit_safe(events::FoundInstalledGame(game));
	}
}

fn owned_game_callback(handle: AppHandle) -> impl Fn(OwnedGame) {
	move |game: OwnedGame| {
		handle.emit_safe(events::FoundOwnedGame(game));
	}
}

#[tauri::command]
#[specta::specta]
async fn update_local_mods(handle: AppHandle) -> Result {
	let resources_path = handle
		.path()
		.resolve("resources", BaseDirectory::Resource)
		.map_err(|err| Error::FailedToGetResourcesPath(err.to_string()))?;

	let mod_loaders = mod_loader::get_map(&resources_path);

	handle.emit_safe(events::SyncModLoaders(mod_loader::get_data_map(
		&mod_loaders,
	)?));
	update_state(mod_loaders.clone(), &handle.app_state().mod_loaders);

	refresh_local_mods(&mod_loaders, &handle);
	refresh_remote_mods(&mod_loaders, &handle).await;

	Ok(())
}

#[tauri::command]
#[specta::specta]
async fn update_remote_games(handle: AppHandle) -> Result {
	handle.emit_safe(events::SyncRemoteGames(remote_games::get().await?));
	Ok(())
}

#[tauri::command]
#[specta::specta]
async fn get_provider_games(handle: AppHandle, provider_id: ProviderId) -> Result {
	let provider = provider::get_provider(provider_id)?;

	provider
		.get_games(
			installed_game_callback(handle.clone()),
			owned_game_callback(handle.clone()),
		)
		.await?;

	Ok(())
}

#[tauri::command]
#[specta::specta]
async fn add_game(path: PathBuf, handle: AppHandle) -> Result {
	let normalized_path = normalize_path(&path);

	let installed_game = manual_provider::add_game(&normalized_path)?;
	let game_name = installed_game.name.clone();

	handle.emit_safe(events::SelectInstalledGame(installed_game.id.clone()));
	handle.emit_safe(events::FoundInstalledGame(installed_game));

	analytics::send_event(analytics::Event::ManuallyAddGame, &game_name).await;

	Ok(())
}

#[tauri::command]
#[specta::specta]
async fn remove_game(installed_game: InstalledGame, handle: AppHandle) -> Result {
	manual_provider::remove_game(&installed_game.executable.path)?;

	handle.emit_safe(events::GameRemoved(installed_game.id));

	Ok(())
}

#[tauri::command]
#[specta::specta]
async fn run_provider_command(
	owned_game: OwnedGame,
	command_action: ProviderCommandAction,
	handle: AppHandle,
) -> Result {
	owned_game
		.provider_commands
		.try_get(&command_action)?
		.run()?;

	handle.emit_safe(events::ExecutedProviderCommand);

	Ok(())
}

#[tauri::command]
#[specta::specta]
async fn delete_steam_appinfo_cache() -> Result {
	steam::appinfo::delete()
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
	std::panic::set_hook(Box::new(|info| {
		#[cfg(target_os = "windows")]
		windows::error_dialog(&info.to_string());

		#[cfg(target_os = "linux")]
		log::error!("Panic: {info}");
	}));

	let builder = Builder::<tauri::Wry>::new()
		.commands(tauri_specta::collect_commands![
			add_game,
			configure_mod,
			delete_mod,
			delete_steam_appinfo_cache,
			download_mod,
			frontend_ready,
			get_local_mods,
			get_mod_loaders,
			get_remote_mods,
			install_mod,
			open_game_folder,
			open_game_mods_folder,
			open_installed_mod_folder,
			open_logs_folder,
			open_mod_folder,
			open_mod_loader_folder,
			open_mods_folder,
			refresh_game,
			remove_game,
			run_provider_command,
			run_runnable_without_game,
			start_game_exe,
			start_game,
			uninstall_all_mods,
			uninstall_mod,
			update_local_mods,
			update_remote_games,
			get_provider_games
		])
		.events(events::collect_events());

	#[cfg(debug_assertions)]
	builder
		.export(
			Typescript::default().bigint(BigIntExportBehavior::BigInt),
			"../frontend/api/bindings.ts",
		)
		.unwrap_or_else(|err| {
			error!("Failed to generate TypeScript bindings: {err}");
			std::process::exit(1);
		});

	tauri::Builder::default()
		.plugin(tauri_plugin_store::Builder::new().build())
		.plugin(tauri_plugin_window_state::Builder::default().build())
		.plugin(tauri_plugin_dialog::init())
		.plugin(tauri_plugin_updater::Builder::default().build())
		.plugin(
			tauri_plugin_log::Builder::new()
				.level(log::LevelFilter::Info)
				.targets([
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
			mod_loaders: Mutex::default(),
			local_mods: Mutex::default(),
			remote_mods: Mutex::default(),
		})
		.invoke_handler(builder.invoke_handler())
		.setup(move |app| {
			builder.mount_events(app);

			if let Some(window) = app.get_webview_window("main") {
				window.set_title(&format!("Rai Pal {}", env!("CARGO_PKG_VERSION")))?;
			}

			Ok(())
		})
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

			#[cfg(target_os = "linux")]
			log::error!("Error: {error}");
		});
}
