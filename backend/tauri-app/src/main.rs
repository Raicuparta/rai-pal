// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
// Command stuff needs to be async so I can spawn tasks.
#![allow(clippy::unused_async)]

use std::sync::RwLock;
use std::{collections::HashMap, path::PathBuf};

use crate::result::{Error, Result};
use app_state::{AppState, StateData, StatefulHandle};
use events::EventEmitter;
use rai_pal_core::game::{Game, GameId};
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
#[cfg(target_os = "windows")]
use rai_pal_core::windows;
use rai_pal_core::{analytics, remote_game, remote_mod, steam};
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
	let state = handle.app_state();
	let data_map = mod_loader::get_data_map(&state.mod_loaders.read_state()?.clone())?;

	Ok(data_map)
}

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
	let state = handle.app_state();
	let games = state.games.try_get(&game_id.provider_id)?.read_state()?;
	let installed_game = games.try_get(&game_id.game_id)?.try_get_installed_game()?;

	Ok(installed_game.open_game_folder()?)
}

#[tauri::command]
#[specta::specta]
async fn open_game_mods_folder(handle: AppHandle, game_id: GameId) -> Result {
	let state = handle.app_state();
	let games = state.games.try_get(&game_id.provider_id)?.read_state()?;
	let installed_game = games.try_get(&game_id.game_id)?.try_get_installed_game()?;

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

fn refresh_game_mods(game_id: &GameId, handle: &AppHandle) -> Result {
	let state = handle.app_state();
	let mut games = state.games.try_get(&game_id.provider_id)?.write_state()?;
	let game = games.try_get_mut(&game_id.game_id)?;
	let installed_game = game.try_get_installed_game_mut()?;
	installed_game.refresh_installed_mods();

	handle.emit_safe(events::FoundGame(game_id.clone()));

	Ok(())
}

#[tauri::command]
#[specta::specta]
async fn install_mod(game_id: GameId, mod_id: &str, handle: AppHandle) -> Result {
	let state = handle.app_state();

	let mod_loaders = state.mod_loaders.read_state()?.clone();

	let local_mod = refresh_and_get_local_mod(mod_id, &mod_loaders, &handle).await?;

	let mod_loader = mod_loaders.try_get(&local_mod.common.loader_id)?;

	let installed_game = {
		let games = state.games.try_get(&game_id.provider_id)?.read_state()?;
		let game = games.try_get(&game_id.game_id)?;
		game.try_get_installed_game()?.clone()
	};

	// Uninstall mod if it already exists, in case there are conflicting leftover files when updating.
	mod_loader
		.uninstall_mod(&installed_game, &local_mod)
		.await?;

	mod_loader.install_mod(&installed_game, &local_mod).await?;

	// TODO figure this out, I want this to happen only at the very end.
	analytics::send_event(analytics::Event::InstallOrRunMod, mod_id).await;

	refresh_game_mods(&game_id, &handle)?;

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

	let installed_game = {
		let games = state.games.try_get(&game_id.provider_id)?.read_state()?;
		let game = games.try_get(&game_id.game_id)?;
		game.try_get_installed_game()?.clone()
	};

	let mod_loaders = state.mod_loaders.read_state()?.clone();
	let local_mod = refresh_and_get_local_mod(mod_id, &mod_loaders, &handle).await?;

	let mod_loader = mod_loaders.try_get(&local_mod.common.loader_id)?;

	mod_loader.configure_mod(&installed_game, &local_mod)?;

	Ok(())
}

#[tauri::command]
#[specta::specta]
async fn open_installed_mod_folder(game_id: GameId, mod_id: &str, handle: AppHandle) -> Result {
	let state = handle.app_state();

	let mod_loaders = state.mod_loaders.read_state()?.clone();
	let local_mod = refresh_and_get_local_mod(mod_id, &mod_loaders, &handle).await?;

	let mod_loader = mod_loaders.try_get(&local_mod.common.loader_id)?;

	let installed_game = {
		let games = state.games.try_get(&game_id.provider_id)?.read_state()?;
		let game = games.try_get(&game_id.game_id)?;
		game.try_get_installed_game()?.clone()
	};
	mod_loader.open_installed_mod_folder(&installed_game, &local_mod)?;

	Ok(())
}

#[tauri::command]
#[specta::specta]
async fn refresh_game(game_id: GameId, handle: AppHandle) -> Result {
	let state = handle.app_state();
	let mut games = state.games.try_get(&game_id.provider_id)?.write_state()?;

	let game = games.try_get_mut(&game_id.game_id)?;

	if let Some(installed_game) = game.installed_game.as_mut() {
		installed_game.refresh_installed_mods();
		installed_game.refresh_executable()?;
	}

	handle.emit_safe(events::FoundGame(game_id));

	Ok(())
}

#[tauri::command]
#[specta::specta]
async fn uninstall_mod(game_id: GameId, mod_id: &str, handle: AppHandle) -> Result {
	let state = handle.app_state();

	let mod_loaders = state.mod_loaders.read_state()?.clone();

	let local_mod = refresh_and_get_local_mod(mod_id, &mod_loaders, &handle).await?;

	let mod_loader = mod_loaders.try_get(&local_mod.common.loader_id)?;

	let installed_game = {
		let games = state.games.try_get(&game_id.provider_id)?.read_state()?;
		let game = games.try_get(&game_id.game_id)?;
		game.try_get_installed_game()?.clone()
	};

	// Uninstall mod if it already exists, in case there are conflicting leftover files when updating.
	mod_loader
		.uninstall_mod(&installed_game, &local_mod)
		.await?;

	refresh_game_mods(&game_id, &handle)?;

	Ok(())
}

#[tauri::command]
#[specta::specta]
async fn uninstall_all_mods(game_id: GameId, handle: AppHandle) -> Result {
	let state = handle.app_state();
	let installed_game = {
		let games = state.games.try_get(&game_id.provider_id)?.read_state()?;
		let game = games.try_get(&game_id.game_id)?;
		game.try_get_installed_game()?.clone()
	};

	installed_game.uninstall_all_mods()?;

	refresh_game_mods(&game_id, &handle)?;

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
async fn update_local_mods(handle: AppHandle) -> Result {
	let resources_path = handle
		.path()
		.resolve("resources", BaseDirectory::Resource)
		.map_err(|err| Error::FailedToGetResourcesPath(err.to_string()))?;

	let mod_loaders = mod_loader::get_map(&resources_path);

	handle.emit_safe(events::SyncModLoaders(mod_loader::get_data_map(
		&mod_loaders,
	)?));

	handle
		.app_state()
		.mod_loaders
		.write_state_value(mod_loaders.clone())?;

	refresh_local_mods(&mod_loaders, &handle)?;
	refresh_remote_mods(&mod_loaders, &handle).await?;

	Ok(())
}

#[tauri::command]
#[specta::specta]
async fn fetch_remote_games(handle: AppHandle) -> Result {
	let state = handle.app_state();
	let remote_games = remote_game::get().await?;

	state
		.games
		.iter()
		.for_each(|(_provider_id, provider_games)| {
			match provider_games.write_state() {
				Ok(mut provider_games_write) => {
					provider_games_write.iter_mut().for_each(|(_, game)| {
						// Assign remote game to any existing game.
						// This is for when the remote games are fetched *after* games are found locally.
						game.remote_game = remote_games
							.get(&game.id.provider_id)
							.and_then(|provider_remote_games| {
								provider_remote_games.get(&game.external_id)
							})
							.or_else(|| {
								remote_games.get(&ProviderId::Manual).and_then(
									|provider_remote_games| {
										game.title.normalized.first().and_then(|normalized_title| {
											provider_remote_games.get(normalized_title)
										})
									},
								)
							})
							.cloned()
					})
				}
				Err(err) => {
					log::error!("Failed to write provider games state: {err}");
				}
			}
		});

	handle.emit_safe(events::GamesChanged());

	state.remote_games.write_state_value(remote_games)?;

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

	let remote_games = state.remote_games.read_state()?.clone();

	let mut fresh_games: HashMap<String, Game> = HashMap::default();

	provider.get_games(|mut game: Game| {
		// Assign the remote game here as we find the new game.
		// This is for when the remote games are fetched *before* games are found locally.
		game.remote_game = remote_games
			.get(&game.id.provider_id)
			.and_then(|provider_remote_games| provider_remote_games.get(&game.external_id))
			.or_else(|| {
				remote_games
					.get(&ProviderId::Manual)
					.and_then(|provider_remote_games| {
						game.title.normalized.first().and_then(|normalized_title| {
							provider_remote_games.get(normalized_title)
						})
					})
			})
			.cloned();

		handle
			.app_state()
			.games
			.get(&provider_id)
			.unwrap()
			.write()
			.unwrap()
			.insert(game.id.game_id.clone(), game.clone());

		handle.emit_safe(events::FoundGame(game.id.clone()));

		fresh_games.insert(game.id.game_id.clone(), game);
	}).await.unwrap_or_else(|err| {
		// It's normal for a provider to fail here if that provider is just missing.
		// So we log those errors here instead of throwing them up.
		log::warn!("Failed to get games for provider {provider_id}. User might just not have it. Error: {err}");
	});

	// After all is done, write again with fresh games, to get rid of the stale ones.
	let mut games_write = state.games.try_get(&provider_id)?.write_state()?;
	*games_write = fresh_games;

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
	let normalized_path = normalize_path(&path);

	let game = manual_provider::add_game(&normalized_path)?;
	let game_name = game.title.display.clone();

	let state = handle.app_state();

	state
		.games
		.get(&ProviderId::Manual)
		.unwrap()
		.write()
		.unwrap()
		.insert(game.id.game_id.clone(), game.clone());

	handle.emit_safe(events::SelectInstalledGame(
		ProviderId::Manual,
		game.id.game_id.clone(),
	));

	analytics::send_event(analytics::Event::ManuallyAddGame, &game_name).await;

	Ok(())
}

#[tauri::command]
#[specta::specta]
async fn remove_game(path: PathBuf, handle: AppHandle) -> Result {
	manual_provider::remove_game(&path)?;

	get_provider_games(handle, ProviderId::Manual).await
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

	let games_iter = state
		.games
		.values()
		.flat_map(|provider_games| provider_games.read().unwrap().clone().into_values());

	let games: Vec<_> = if let Some(query) = data_query.as_ref() {
		let mut games: Vec<_> = games_iter.filter(|game| query.matches(game)).collect();
		games.sort_by(|game_a, game_b| query.sort(game_a, game_b));

		games
	} else {
		let games: Vec<_> = games_iter.collect();
		games
	};

	Ok(games.into_iter().map(|game| game.id).collect())
}

#[tauri::command]
#[specta::specta]
async fn get_game(id: GameId, handle: AppHandle) -> Result<Game> {
	Ok(handle
		.app_state()
		.games
		.try_get(&id.provider_id)?
		.read_state()?
		.try_get(&id.game_id)?
		.clone())
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
			mod_loaders: RwLock::default(),
			local_mods: RwLock::default(),
			remote_mods: RwLock::default(),
			remote_games: RwLock::default(),
			games: HashMap::from_iter(
				provider::get_provider_ids()
					.iter()
					.map(|&id| (id, RwLock::default())),
			),
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
