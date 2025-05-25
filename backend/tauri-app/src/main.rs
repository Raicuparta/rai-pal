// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
// Command stuff needs to be async so I can spawn tasks.
#![allow(clippy::unused_async)]

use std::time::{SystemTime, UNIX_EPOCH};
use std::{collections::HashMap, path::PathBuf};

use crate::result::{Error, Result};
use app_settings::AppSettings;
use app_state::{AppState, StateData, StatefulHandle};
use events::EventEmitter;
use rai_pal_core::game::{DbGame, GameId};
use rai_pal_core::games_query::GamesQuery;
use rai_pal_core::local_database::{GameDatabase, GameIdsResponse, attach_remote_database};
use rai_pal_core::local_mod::{self, LocalMod};
use rai_pal_core::maps::TryGettable;
use rai_pal_core::mod_loaders::mod_loader::{self, ModLoaderActions};
use rai_pal_core::paths::{self, normalize_path};
use rai_pal_core::providers::provider::ProviderId;
use rai_pal_core::providers::provider_command::ProviderCommand;
use rai_pal_core::providers::steam::steam_provider::Steam;
use rai_pal_core::providers::{
	manual_provider,
	provider::{self, ProviderActions},
};
use rai_pal_core::remote_game::{self};
#[cfg(target_os = "windows")]
use rai_pal_core::windows;
use rai_pal_core::{analytics, remote_mod};
use strum::IntoEnumIterator;
use tauri::path::BaseDirectory;
use tauri::{AppHandle, Manager};
use tauri_plugin_log::{Target, TargetKind};
use tauri_plugin_window_state::StateFlags;
use tauri_specta::Builder;

mod app_settings;
mod app_state;
mod events;
mod result;
#[cfg(debug_assertions)]
mod typescript;

#[tauri::command]
#[specta::specta]
async fn get_local_mods(handle: AppHandle) -> Result<local_mod::Map> {
	Ok(handle.app_state().local_mods.read_state()?.clone())
}

#[tauri::command]
#[specta::specta]
async fn get_remote_mods(handle: AppHandle) -> Result<remote_mod::Map> {
	Ok(handle.app_state().remote_mods.read_state()?.clone())
}

#[tauri::command]
#[specta::specta]
async fn open_game_folder(handle: AppHandle, game_id: GameId) -> Result {
	handle
		.app_state()
		.database
		.get_game(&game_id)?
		.open_game_folder()?;
	Ok(())
}

#[tauri::command]
#[specta::specta]
async fn open_game_mods_folder(handle: AppHandle, game_id: GameId) -> Result {
	handle
		.app_state()
		.database
		.get_game(&game_id)?
		.open_mods_folder()?;
	Ok(())
}

#[tauri::command]
#[specta::specta]
async fn open_mods_folder() -> Result {
	paths::open_folder_or_parent(&paths::installed_mods_path()?)?;
	Ok(())
}

#[tauri::command]
#[specta::specta]
async fn open_mod_folder(mod_id: &str, handle: AppHandle) -> Result {
	Ok(handle
		.app_state()
		.local_mods
		.read_state()?
		.try_get(mod_id)?
		.open_folder()?)
}

#[tauri::command]
#[specta::specta]
async fn open_mod_loader_folder(mod_loader_id: &str, handle: AppHandle) -> Result {
	Ok(handle
		.app_state()
		.mod_loaders
		.read_state()?
		.try_get(mod_loader_id)?
		.open_folder()?)
}

#[tauri::command]
#[specta::specta]
async fn download_mod(mod_id: &str, handle: AppHandle) -> Result {
	let state = handle.app_state();
	let remote_mods = state.remote_mods.read_state()?.clone();
	let remote_mod = remote_mods.try_get(mod_id)?;
	let mod_loaders = state.mod_loaders.read_state()?.clone();

	mod_loaders
		.try_get(&remote_mod.common.loader_id)?
		.download_mod(remote_mod)
		.await?;

	refresh_local_mods(&mod_loaders, &handle)?;

	Ok(())
}

#[tauri::command]
#[specta::specta]
async fn delete_mod(mod_id: &str, handle: AppHandle) -> Result {
	let state = handle.app_state();
	let local_mods = state.local_mods.read_state()?;
	let local_mod = local_mods.try_get(mod_id)?;
	let mod_loaders = state.mod_loaders.read_state()?;

	mod_loaders
		.try_get(&local_mod.common.loader_id)?
		.delete_mod(local_mod)?;

	refresh_local_mods(&mod_loaders, &handle)?;

	Ok(())
}

#[tauri::command]
#[specta::specta]
async fn install_mod(game_id: GameId, mod_id: &str, handle: AppHandle) -> Result {
	let state = handle.app_state();
	let game = state.database.get_game(&game_id)?;

	let mod_loaders = state.mod_loaders.read_state()?.clone();

	let local_mod = refresh_and_get_local_mod(mod_id, &mod_loaders, &handle).await?;

	let mod_loader = mod_loaders.try_get(&local_mod.common.loader_id)?;

	// Uninstall mod if it already exists, in case there are conflicting leftover files when updating.
	mod_loader.uninstall_mod(&game, &local_mod).await?;

	mod_loader.install_mod(&game, &local_mod).await?;

	handle.emit_safe(events::RefreshGame(game_id.clone()));

	analytics::send_event(analytics::Event::InstallOrRunMod, mod_id).await;

	Ok(())
}

#[tauri::command]
#[specta::specta]
async fn run_runnable_without_game(mod_id: &str, handle: AppHandle) -> Result {
	let state = handle.app_state();
	let mod_loaders = state.mod_loaders.read_state()?.clone();
	let local_mod = refresh_and_get_local_mod(mod_id, &mod_loaders, &handle).await?;
	let mod_loader = mod_loaders.try_get(&local_mod.common.loader_id)?;

	mod_loader.run_without_game(&local_mod).await?;

	analytics::send_event(analytics::Event::InstallOrRunMod, mod_id).await;

	Ok(())
}

#[tauri::command]
#[specta::specta]
async fn configure_mod(game_id: GameId, mod_id: &str, handle: AppHandle) -> Result {
	let state = handle.app_state();
	let game = state.database.get_game(&game_id)?;
	let mod_loaders = state.mod_loaders.read_state()?.clone();
	let local_mod = refresh_and_get_local_mod(mod_id, &mod_loaders, &handle).await?;
	let mod_loader = mod_loaders.try_get(&local_mod.common.loader_id)?;

	mod_loader.configure_mod(&game, &local_mod)?;

	Ok(())
}

#[tauri::command]
#[specta::specta]
async fn open_installed_mod_folder(game_id: GameId, mod_id: &str, handle: AppHandle) -> Result {
	let state = handle.app_state();
	let game = state.database.get_game(&game_id)?;

	let mod_loaders = state.mod_loaders.read_state()?.clone();
	let local_mod = refresh_and_get_local_mod(mod_id, &mod_loaders, &handle).await?;

	let mod_loader = mod_loaders.try_get(&local_mod.common.loader_id)?;

	mod_loader.open_installed_mod_folder(&game, &local_mod)?;

	Ok(())
}

#[tauri::command]
#[specta::specta]
async fn refresh_game(game_id: GameId, handle: AppHandle) -> Result {
	let state = handle.app_state();
	let mut game = state.database.get_game(&game_id)?;
	game.refresh_executable()?;
	state.database.insert_game(&game);

	log::info!("Refreshing game: {}", game_id.game_id);

	handle.emit_safe(events::RefreshGame(game_id.clone()));

	Ok(())
}

#[tauri::command]
#[specta::specta]
async fn uninstall_mod(game_id: GameId, mod_id: &str, handle: AppHandle) -> Result {
	let state = handle.app_state();
	let game = state.database.get_game(&game_id)?;

	let mod_loaders = state.mod_loaders.read_state()?.clone();

	let local_mod = refresh_and_get_local_mod(mod_id, &mod_loaders, &handle).await?;

	let mod_loader = mod_loaders.try_get(&local_mod.common.loader_id)?;

	// Uninstall mod if it already exists, in case there are conflicting leftover files when updating.
	mod_loader.uninstall_mod(&game, &local_mod).await?;

	handle.emit_safe(events::RefreshGame(game_id.clone()));

	Ok(())
}

#[tauri::command]
#[specta::specta]
async fn uninstall_all_mods(game_id: GameId, handle: AppHandle) -> Result {
	handle
		.app_state()
		.database
		.get_game(&game_id)?
		.uninstall_all_mods()?;

	handle.emit_safe(events::RefreshGame(game_id.clone()));

	Ok(())
}

fn refresh_local_mods(mod_loaders: &mod_loader::Map, handle: &AppHandle) -> Result<local_mod::Map> {
	let local_mods: HashMap<_, _> = mod_loaders
		.values()
		.filter_map(|mod_loader| match mod_loader.get_local_mods() {
			Ok(local_mods) => Some(local_mods),
			Err(err) => {
				log::error!("Failed to get local mods: {err}");
				None
			}
		})
		.flatten()
		.collect();

	log::info!("Found {} local mods.", { local_mods.len() });
	handle.emit_safe(events::SyncLocalMods(local_mods.clone()));

	handle
		.app_state()
		.local_mods
		.write_state_value(local_mods.clone())?;

	Ok(local_mods)
}

async fn refresh_remote_mods(
	mod_loaders: &mod_loader::Map,
	handle: &AppHandle,
) -> Result<remote_mod::Map> {
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

	handle
		.app_state()
		.remote_mods
		.write_state_value(remote_mods.clone())?;

	Ok(remote_mods)
}

async fn refresh_and_get_local_mod(
	mod_id: &str,
	mod_loaders: &mod_loader::Map,
	handle: &AppHandle,
) -> Result<LocalMod> {
	let local_mods = {
		let state = handle.app_state();

		let state_local_mods = state.local_mods.read_state()?.clone();
		if state_local_mods.contains_key(mod_id) {
			Ok(state_local_mods)
		} else {
			// Local mod wasn't in app state,
			// so let's sync app state to local files in case some file was manually changed.
			let disk_local_mods = refresh_local_mods(mod_loaders, handle);

			if state_local_mods.contains_key(mod_id) {
				disk_local_mods
			} else {
				let remote_mods = state.remote_mods.read_state()?.clone();
				let remote_mod = remote_mods.try_get(mod_id)?;
				let mod_loader = mod_loaders.try_get(&remote_mod.common.loader_id)?;

				if remote_mod.data.latest_version.is_some() {
					// If local mod still can't be found on disk,
					// we try to download it from the database.
					mod_loader
						.download_mod(remote_mods.try_get(mod_id)?)
						.await?;
				} else {
					// If downloading from the database isn't possible,
					// we just open the mod loader folder so the user can install it themselves.
					mod_loader.open_folder()?;
				}

				refresh_local_mods(mod_loaders, handle)
			}
		}
	}?;

	Ok(local_mods.try_get(mod_id).cloned()?)
}

#[tauri::command]
#[specta::specta]
async fn refresh_mods(handle: AppHandle) -> Result {
	let resources_path = handle
		.path()
		.resolve("resources", BaseDirectory::Resource)
		.map_err(|err| Error::FailedToGetResourcesPath(err.to_string()))?;

	let mod_loaders = mod_loader::get_map(&resources_path);

	handle.emit_safe(events::SyncModLoaders(mod_loader::get_data_map(
		&mod_loaders,
	)?));

	log::info!("Found {} mod loaders. Refreshing local mods...", {
		mod_loaders.len()
	});
	refresh_local_mods(&mod_loaders, &handle)?;

	log::info!("Refreshing remote mods...");
	refresh_remote_mods(&mod_loaders, &handle).await?;

	log::info!("Saving mods to state.");
	handle
		.app_state()
		.mod_loaders
		.write_state_value(mod_loaders)?;

	Ok(())
}

#[tauri::command]
#[specta::specta]
async fn refresh_remote_games(handle: AppHandle) -> Result {
	let state = handle.app_state();
	let path = remote_game::download_database().await?;
	attach_remote_database(state.database.lock().unwrap(), &path)?;

	Ok(())
}

#[tauri::command]
#[specta::specta]
async fn refresh_games(handle: AppHandle, provider_id: ProviderId) -> Result {
	let state = handle.app_state();

	let start_time = SystemTime::now()
		.duration_since(UNIX_EPOCH)
		.unwrap()
		.as_secs();

	let provider = provider::get_provider(provider_id)?;

	provider.insert_games(&state.database).await?;
	state
		.database
		.remove_stale_games(&provider_id, start_time)?;

	Ok(())
}

#[tauri::command]
#[specta::specta]
async fn add_game(path: PathBuf, handle: AppHandle) -> Result {
	let normalized_path = normalize_path(&path);

	let game = manual_provider::add_game(&normalized_path)?;
	let game_name = game.display_title.clone();

	let state = handle.app_state();

	state.database.insert_game(&game);

	handle.emit_safe(events::RefreshGame(GameId {
		provider_id: game.provider_id,
		game_id: game.game_id.clone(),
	}));

	handle.emit_safe(events::GamesChanged());

	handle.emit_safe(events::SelectInstalledGame(
		ProviderId::Manual,
		game.game_id,
	));

	analytics::send_event(analytics::Event::ManuallyAddGame, &game_name).await;

	Ok(())
}

#[tauri::command]
#[specta::specta]
async fn remove_game(game_id: GameId, handle: AppHandle) -> Result {
	let game = handle.app_state().database.get_game(&game_id)?;

	manual_provider::remove_game(&game)?;

	refresh_games(handle, ProviderId::Manual).await
}

#[tauri::command]
#[specta::specta]
async fn run_provider_command(provider_command: ProviderCommand, handle: AppHandle) -> Result {
	provider_command.run()?;

	handle.emit_safe(events::ExecutedProviderCommand);

	Ok(())
}

#[tauri::command]
#[specta::specta]
async fn reset_steam_cache(handle: AppHandle) -> Result {
	Steam::delete_cache()?;

	refresh_games(handle, ProviderId::Steam).await?;

	Ok(())
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

#[tauri::command]
#[specta::specta]
async fn get_game_ids(handle: AppHandle, query: Option<GamesQuery>) -> Result<GameIdsResponse> {
	let state = handle.app_state();
	Ok(state.database.get_game_ids(query)?)
}

#[tauri::command]
#[specta::specta]
async fn get_game(handle: AppHandle, game_id: GameId) -> Result<DbGame> {
	let state = handle.app_state();
	Ok(state.database.get_game(&game_id)?)
}

#[tauri::command]
#[specta::specta]
async fn get_app_settings() -> Result<AppSettings> {
	Ok(AppSettings::read())
}

#[tauri::command]
#[specta::specta]
async fn save_app_settings(settings: AppSettings) -> Result {
	settings.try_write()
}

#[tauri::command]
#[specta::specta]
async fn get_installed_mod_versions(
	game_id: GameId,
	app_handle: AppHandle,
) -> Result<HashMap<String, String>> {
	let game = get_game(app_handle.clone(), game_id.clone()).await?;
	Ok(game.get_installed_mod_versions())
}

fn main() {
	// Since I'm making all exposed functions async, panics won't crash anything important, I think.
	// So I can just catch panics here and show a system message with the error.
	std::panic::set_hook(Box::new(|info| {
		println!("Panic: {info}");

		#[cfg(target_os = "windows")]
		windows::error_dialog(&format!("I found a panic!!!: {info}"));
	}));

	let builder = Builder::<tauri::Wry>::new()
		.commands(tauri_specta::collect_commands![
			add_game,
			configure_mod,
			delete_mod,
			download_mod,
			frontend_ready,
			get_app_settings,
			get_game_ids,
			get_game,
			get_installed_mod_versions,
			get_local_mods,
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
			refresh_games,
			refresh_mods,
			refresh_remote_games,
			remove_game,
			reset_steam_cache,
			run_provider_command,
			run_runnable_without_game,
			save_app_settings,
			uninstall_all_mods,
			uninstall_mod,
		])
		.events(events::collect_events())
		.constant("PROVIDER_IDS", ProviderId::iter().collect::<Vec<_>>())
		.error_handling(tauri_specta::ErrorHandlingMode::Throw);

	#[cfg(debug_assertions)]
	typescript::export(&builder);

	tauri::Builder::default()
		.plugin(tauri_plugin_shell::init())
		.plugin(tauri_plugin_os::init())
		.plugin(tauri_plugin_store::Builder::new().build())
		.plugin(
			tauri_plugin_window_state::Builder::default()
				.with_state_flags(StateFlags::POSITION | StateFlags::SIZE)
				.build(),
		)
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
		.manage(AppState::new().unwrap())
		.invoke_handler(builder.invoke_handler())
		.setup(move |app| {
			builder.mount_events(app);

			if let Some(window) = app.get_webview_window("main") {
				let mut title = format!("Rai Pal {}", env!("CARGO_PKG_VERSION"));
				if cfg!(debug_assertions) {
					title += " DEV";
				}
				window.set_title(&title)?;

				// Window is created hidden in tauri.conf.json.
				// We show it here once everything is ready, which reduces the jumping around
				// that happens while waiting for tauri_plugin_window_state to do its thing.
				// We could also trigger this on the frontend to reduce the white flash,
				// but it never seems to go away, and that introduces an extra delay
				// until something is visible, so I figure I'd just show it here.
				window.show()?;

				window.on_window_event(|event| {
					if let tauri::WindowEvent::Destroyed { .. } = event {
						// Once the window is closed, we don't need to report panics anymore.
						// I'm doing this because closing the window abruptly while events are being sent
						// causes panics, so it was easy to trigger those messages by just closing while loading data.
						let _ = std::panic::take_hook();
					}
				});
			}

			let app_handle = app.app_handle().clone();

			tauri::async_runtime::spawn(async move {
				let state = app_handle.app_state();
				let database_connection = state.database.lock().unwrap();
				let cloned_handle = app_handle.clone();
				database_connection.update_hook(Some({
					move |_, _: &str, _: &str, _| {
						cloned_handle.emit_safe(events::GamesChanged());
					}
				}));
			});

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
			#[cfg(target_os = "macos")]
			log::error!("Error: {error}");
		});
}
