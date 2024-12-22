// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
// Command stuff needs to be async so I can spawn tasks.
#![allow(clippy::unused_async)]

use std::sync::{Arc, RwLock};
use std::{collections::HashMap, path::PathBuf, sync::Mutex};

use crate::result::{Error, Result};
use app_state::{AppState, DataValue, GameId, StateData, StatefulHandle};
use events::EventEmitter;
use rai_pal_core::game::Game;
use rai_pal_core::game_executable::GameExecutable;
use rai_pal_core::games_query::GamesQuery;
use rai_pal_core::installed_game::InstalledGame;
use rai_pal_core::local_mod::{self, LocalMod};
use rai_pal_core::maps::TryGettable;
use rai_pal_core::mod_loaders::mod_loader::{self, ModLoaderActions};
use rai_pal_core::paths::{self, normalize_path};
use rai_pal_core::providers::provider::ProviderId;
use rai_pal_core::providers::provider_cache::ProviderCache;
use rai_pal_core::providers::{
	manual_provider,
	provider::{self, ProviderActions},
	provider_command::ProviderCommandAction,
};
use rai_pal_core::remote_games::{self, IdKind, RemoteGame};
#[cfg(target_os = "windows")]
use rai_pal_core::windows;
use rai_pal_core::{analytics, remote_mod, steam};
use tauri::path::BaseDirectory;
use tauri::{AppHandle, Manager};
use tauri_plugin_log::{Target, TargetKind};
use tauri_plugin_window_state::StateFlags;
use tauri_specta::Builder;

mod app_state;
mod events;
mod result;
#[cfg(debug_assertions)]
mod typescript;

#[tauri::command]
#[specta::specta]
async fn get_mod_loaders(handle: AppHandle) -> Result<mod_loader::DataMap> {
	Ok(mod_loader::get_data_map(
		&handle.app_state().mod_loaders.get_data()?,
	)?)
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
	Ok(installed_game.open_game_folder()?)
}

#[tauri::command]
#[specta::specta]
async fn open_game_mods_folder(installed_game: InstalledGame) -> Result {
	Ok(installed_game.open_mods_folder()?)
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
		.try_get(mod_id)?
		.open_folder()?)
}

#[tauri::command]
#[specta::specta]
async fn open_mod_loader_folder(mod_loader_id: &str, handle: AppHandle) -> Result {
	Ok(handle
		.app_state()
		.mod_loaders
		.try_get(mod_loader_id)?
		.open_folder()?)
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
	Ok(installed_game.start_exe()?)
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
	// TODO

	// let mut refreshed_game = installed_game.clone();
	// refreshed_game.refresh_installed_mods();
	// refreshed_game.refresh_executable()?;

	// if let Some(installed_games_state) = handle
	// 	.app_state()
	// 	.installed_games
	// 	.get(&installed_game.provider)
	// {
	// 	let mut installed_games = installed_games_state.get_data()?;
	// 	installed_games.insert(refreshed_game.id.clone(), refreshed_game.clone());
	// 	update_installed_games_state(handle, &installed_game.provider, &installed_games);
	// }

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
		.filter_map(|mod_loader| match mod_loader.get_local_mods() {
			Ok(local_mods) => Some(local_mods),
			Err(err) => {
				log::error!("Failed to get local mods: {err}");
				None
			}
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

	Ok(local_mods.try_get(mod_id).cloned()?)
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
async fn fetch_remote_games(handle: AppHandle) -> Result {
	let state = handle.app_state();
	let remote_games = remote_games::get().await?;
	let remote_games_by_provider: HashMap<IdKind, HashMap<String, RemoteGame>> = remote_games
		.iter()
		.flat_map(|remote_game| {
			remote_game.ids.iter().map(move |(id_kind, ids)| {
				ids.iter()
					.map(move |id| (*id_kind, id.clone(), remote_game.clone()))
			})
		})
		.flatten()
		.fold(HashMap::new(), |mut map, (id_kind, id, remote_game)| {
			map.entry(id_kind).or_default().insert(id, remote_game);
			map
		});

	let mut games = state.games.write().unwrap();
	games.iter_mut().for_each(|(_provider_id, provider_games)| {
		provider_games.iter_mut().for_each(|game| {
			// Assign remote game to any existing game.
			// This is for when the remote games are fetched *after* games are found locally.
			game.remote_game = remote_games_by_provider
				.get(&IdKind::Steam)
				.and_then(|provider_remote_games| provider_remote_games.get(&game.id))
				.cloned()
		})
	});
	let mut remote_games_write_lock = state.remote_games.write().unwrap();
	*remote_games_write_lock = remote_games_by_provider;

	Ok(())
}

#[tauri::command]
#[specta::specta]
async fn get_provider_games(handle: AppHandle, provider_id: ProviderId) -> Result {
	let state = handle.app_state();

	// let mut cache = ProviderCache::new(provider_id)?;

	// if let Err(err) = cache.load() {
	// 	log::warn!("Failed to load cache for provider {provider_id}: {err}");
	// } else {
	// 	installed_games = cache.data.installed_games.clone();
	// 	owned_games = cache.data.owned_games.clone();
	// 	update_installed_games_state(&handle, &provider_id, &installed_games);
	// 	update_owned_games_state(&handle, &provider_id, &owned_games);
	// }

	let provider = provider::get_provider(provider_id)?;

	state.games.write().unwrap().clear();

	let remote_games = state.remote_games.read().unwrap().clone();
	provider.get_games_new(|mut game: Game| {
			// Assign the remote game here as we find the new game.
			// This is for when the remote games are fetched *before* games are found locally.
		game.remote_game = remote_games
			.get(&IdKind::Steam)
			.and_then(|provider_remote_games| provider_remote_games.get(&game.id))
			.cloned();

		handle
			.app_state()
			.games
			.write()
			.unwrap()
			.entry(provider_id)
			.or_default()
			.push(game);
		handle.emit_safe(events::FoundGame());
	}).await.unwrap_or_else(|err| {
		// It's normal for a provider to fail here if that provider is just missing.
		// So we log those errors here instead of throwing them up.
		log::warn!("Failed to get games for provider {provider_id}. User might just not have it. Error: {err}");
	});

	// update_games_state(&handle, &provider_id, &games);

	// cache
	// 	.set_data(ProviderData {
	// 		installed_games: installed_games_without_cache.clone(),
	// 		owned_games: owned_games_without_cache.clone(),
	// 	})
	// 	.save()?;

	Ok(())
}

#[tauri::command]
#[specta::specta]
async fn get_provider_ids() -> Result<Vec<ProviderId>> {
	Ok(provider::get_provider_ids())
}

#[tauri::command]
#[specta::specta]
async fn add_game(path: PathBuf, handle: AppHandle) -> Result {
	// let normalized_path = normalize_path(&path);

	// let installed_game = manual_provider::add_game(&normalized_path)?;
	// let game_name = installed_game.title.display.clone();

	// if let Some(installed_games_state) = handle.app_state().installed_games.get(&ProviderId::Manual)
	// {
	// 	let mut installed_games = installed_games_state.get_data()?;
	// 	installed_games.insert(installed_game.id.clone(), installed_game.clone());
	// 	update_installed_games_state(&handle, &installed_game.provider, &installed_games);
	// }

	// handle.emit_safe(events::SelectInstalledGame(
	// 	installed_game.provider,
	// 	installed_game.id.clone(),
	// ));

	// analytics::send_event(analytics::Event::ManuallyAddGame, &game_name).await;

	Ok(())
}

#[tauri::command]
#[specta::specta]
async fn remove_game(installed_game: InstalledGame, handle: AppHandle) -> Result {
	// manual_provider::remove_game(&installed_game.executable.path)?;

	// if let Some(installed_games_state) = handle
	// 	.app_state()
	// 	.installed_games
	// 	.get(&installed_game.provider)
	// {
	// 	let mut installed_games = installed_games_state.get_data()?;
	// 	installed_games.remove(&installed_game.id);
	// 	update_installed_games_state(&handle, &installed_game.provider, &installed_games);
	// }

	Ok(())
}

#[tauri::command]
#[specta::specta]
async fn run_provider_command(
	game: Game,
	command_action: ProviderCommandAction,
	handle: AppHandle,
) -> Result {
	game.provider_commands.try_get(&command_action)?.run()?;

	handle.emit_safe(events::ExecutedProviderCommand);

	Ok(())
}

#[tauri::command]
#[specta::specta]
async fn delete_steam_appinfo_cache() -> Result {
	Ok(steam::appinfo::delete()?)
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
async fn get_data(handle: AppHandle, data_query: Option<GamesQuery>) -> Result<Vec<GameId>> {
	let state = handle.app_state();

	let read_guard = state.games.read().unwrap();

	let games_iter = read_guard
		.values()
		.flat_map(|games| games.iter())
		.enumerate();

	let games: Vec<_> = if let Some(query) = data_query.as_ref() {
		let mut games: Vec<_> = games_iter
			.filter(|(_index, game)| query.matches(game))
			.collect();
		games.sort_by(|(_index_a, game_a), (_index_b, game_b)| query.sort(game_a, game_b));

		games
	} else {
		let games: Vec<_> = games_iter.collect();
		games
	};

	Ok(games
		.into_iter()
		.map(|(index, game)| GameId {
			index,
			provider_id: game.provider_id,
		})
		.collect())
}

#[tauri::command]
#[specta::specta]
async fn get_game(
	handle: AppHandle,
	provider_id: ProviderId,
	index: usize,
) -> Result<Option<Game>> {
	Ok(handle
		.app_state()
		.games
		.read()
		.unwrap()
		.try_get(&provider_id)?
		.get(index)
		.cloned())
}

#[tauri::command]
#[specta::specta]
async fn clear_cache() -> Result {
	ProviderCache::clear_all()?;
	Ok(())
}

fn main() {
	// Since I'm making all exposed functions async, panics won't crash anything important, I think.
	// So I can just catch panics here and show a system message with the error.
	std::panic::set_hook(Box::new(|info| {
		#[cfg(target_os = "windows")]
		windows::error_dialog(&format!("I found a panic!!!: {info}"));

		#[cfg(target_os = "linux")]
		log::error!("Panic: {info}");
	}));

	let builder = Builder::<tauri::Wry>::new()
		.commands(tauri_specta::collect_commands![
			add_game,
			clear_cache,
			configure_mod,
			delete_mod,
			delete_steam_appinfo_cache,
			download_mod,
			fetch_remote_games,
			frontend_ready,
			get_local_mods,
			get_mod_loaders,
			get_data,
			get_provider_games,
			get_provider_ids,
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
			get_game,
		])
		.events(events::collect_events());

	#[cfg(debug_assertions)]
	typescript::export(&builder);

	tauri::Builder::default()
		.plugin(tauri_plugin_shell::init())
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
		.manage(AppState {
			mod_loaders: Mutex::default(),
			local_mods: Mutex::default(),
			remote_mods: Mutex::default(),
			remote_games: RwLock::default(),
			games: RwLock::default(),
		})
		.invoke_handler(builder.invoke_handler())
		.setup(move |app| {
			builder.mount_events(app);

			if let Some(window) = app.get_webview_window("main") {
				window.set_title(&format!("Rai Pal {}", env!("CARGO_PKG_VERSION")))?;

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
