// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
// Command stuff needs to be async so I can spawn tasks.
#![allow(clippy::unused_async)]

use std::path::Path;
use std::sync::RwLock;
use std::{collections::HashMap, path::PathBuf};

use crate::result::{Error, Result};
use app_settings::AppSettings;
use app_state::{AppState, StateData, StatefulHandle};
use events::EventEmitter;
use rai_pal_core::game::{self, DbGame, Game, GameId};
use rai_pal_core::game_engines::game_engine::{EngineVersion, EngineVersionNumbers, GameEngine};
use rai_pal_core::game_executable::GameExecutable;
use rai_pal_core::games_query::{GamesQuery, GamesSortBy, InstallState};
use rai_pal_core::installed_game::{self, InstalledGame, InstalledModVersions};
use rai_pal_core::local_mod::{self, LocalMod};
use rai_pal_core::maps::TryGettable;
use rai_pal_core::mod_loaders::mod_loader::{self, ModLoaderActions};
use rai_pal_core::paths::{self, normalize_path, AsValidStr};
use rai_pal_core::providers::provider::ProviderId;
use rai_pal_core::providers::provider_cache;
use rai_pal_core::providers::provider_command::ProviderCommand;
use rai_pal_core::providers::steam::steam_provider::Steam;
use rai_pal_core::providers::{
	manual_provider,
	provider::{self, ProviderActions},
	provider_command::ProviderCommandAction,
};
use rai_pal_core::remote_game::{self, RemoteGame};
#[cfg(target_os = "windows")]
use rai_pal_core::windows;
use rai_pal_core::{analytics, remote_mod};
use rai_pal_proc_macros::serializable_struct;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::{Executor, Pool, Row, Sqlite};
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

#[serializable_struct]
struct GameIdsResponse {
	game_ids: Vec<GameId>,
	total_count: i64,
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

	refresh_game_mods(&game_id, &handle)?;

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
	let pool = state.database_pool.read_state()?.clone();
	let path = remote_game::download_database().await?;
	attach_remote_database(&pool, &path).await?;
	handle.emit_safe(events::GamesChanged());

	Ok(())
}

async fn attach_remote_database(pool: &Pool<Sqlite>, path: &Path) -> Result {
	if !path.is_file() {
		return Ok(());
	}

	sqlx::query(&format!(r#"
    ATTACH DATABASE 'file:{}?mode=ro' AS 'remote';
    INSERT OR IGNORE INTO main.remote_games
    SELECT * FROM remote.games;
    DETACH DATABASE remote;
	"#, path.try_to_str()?))
		.execute(pool)
		.await?;

	Ok(())
}

pub async fn setup_database() -> Result<Pool<Sqlite>> {
	let path = paths::app_data_path()?.join("db.sqlite");
	if path.is_file() {
		std::fs::remove_file(&path)?;
	}

	let config = sqlx::sqlite::SqliteConnectOptions::new()
		.filename(path)
		.create_if_missing(true)
		.journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
		.synchronous(sqlx::sqlite::SqliteSynchronous::Off)
		.in_memory(false)
		.shared_cache(true);

	let pool = SqlitePoolOptions::new()
		.connect_with(config)
		.await?;

	// Run the initial migration
	pool.execute(r#"
		CREATE TABLE IF NOT EXISTS games (
				provider_id TEXT NOT NULL,
				game_id TEXT NOT NULL,
				external_id TEXT NOT NULL,
				display_title TEXT NOT NULL,
				normalized_titles TEXT NOT NULL,
				title_discriminator TEXT,
				thumbnail_url TEXT,
				tags TEXT,
				release_date INTEGER,
				installed_game TEXT,
				extra_data TEXT,
				FOREIGN KEY(installed_game) REFERENCES installed_games(id),
				PRIMARY KEY (provider_id, game_id)
		);

		CREATE INDEX IF NOT EXISTS idx_games_external_id ON games(provider_id, external_id);
		CREATE INDEX IF NOT EXISTS idx_games_installed_game ON games(installed_game);

		CREATE TABLE IF NOT EXISTS installed_games (
				id TEXT NOT NULL PRIMARY KEY,
				exe_path TEXT NOT NULL,
				engine_brand TEXT,
				engine_version TEXT,
				unity_backend TEXT,
				architecture TEXT
		);

		CREATE TABLE IF NOT EXISTS remote_games (
        provider_id TEXT NOT NULL,
        external_id TEXT NOT NULL,
        engine_brand TEXT,
        engine_version TEXT,
        subscriptions TEXT,
        PRIMARY KEY (provider_id, external_id)
		);
		"#,
	)
	.await?;

	attach_remote_database(&pool, &remote_game::get_database_file_path()?).await?;

	Ok(pool)
}

#[tauri::command]
#[specta::specta]
async fn refresh_games(handle: AppHandle, provider_id: ProviderId) -> Result {
	let state = handle.app_state();

	let provider = provider::get_provider(provider_id)?;

	let mut fresh_games = game::Map::default();

	provider
		.get_games(|mut game: Game| {
			
		match state.database_pool.read_state() {
			Ok(pool) => {
				let pool = pool.clone();
				let game_clone = game.clone();
				tauri::async_runtime::spawn_blocking(move || {
					let game_query = sqlx::query::<Sqlite>(
						"INSERT OR REPLACE INTO games (provider_id, game_id, external_id, display_title, normalized_titles, thumbnail_url, release_date, installed_game, tags, title_discriminator, extra_data) 
						 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
					)
					.bind(game_clone.id.provider_id)
					.bind(game_clone.id.game_id.clone())
					.bind(game_clone.external_id.clone())
					.bind(game_clone.title.display.clone())
					.bind(serde_json::to_string(&game_clone.title.normalized).ok()) // TODO log error.
					.bind(game_clone.thumbnail_url.clone())
					.bind(game_clone.release_date)
					.bind(game_clone.installed_game.as_ref().map(|installed_game| {
						installed_game.id.clone()
					}))
					.bind(serde_json::to_string(&game_clone.tags).ok()) // TODO log error.
					.bind(game_clone.installed_game.as_ref().map(|installed_game| {
						installed_game.discriminator.clone()
					}))
					.bind(serde_json::to_string(&game::ExtraData {
						provider_commands: game_clone.provider_commands,
						installed_mod_versions: game_clone.installed_game.as_ref().map(|installed_game| {
							installed_game.installed_mod_versions.clone()
						}).unwrap_or_default(),
					}).ok()); // TODO log error;

					let installed_game_query = game_clone.installed_game.as_ref().map(|installed_game| sqlx::query::<Sqlite>(
						"INSERT OR REPLACE INTO installed_games (id, exe_path, engine_brand, engine_version, unity_backend) 
						 VALUES (?, ?, ?, ?, ?)"
					)
						.bind(installed_game.id.clone())
						.bind(installed_game.executable.path.display().to_string()) // TODO lossy path guy.
						// .bind(installed_game.executable.name.clone())
						.bind(installed_game.executable.engine.as_ref().map(|engine| engine.brand))
						.bind(installed_game.executable.engine.as_ref().map(|engine| engine.version.as_ref().map(|version| version.display.clone()))) // TODO lossy version.
						.bind(installed_game.executable.scripting_backend.as_ref().map(|unity_backend| unity_backend.to_string()))
					);

					tauri::async_runtime::block_on(async {
						// TODO transaction
						if let Some(installed_game_query) = installed_game_query {
							installed_game_query.execute(&pool).await;
						}
						if let Err(err) = pool.execute(game_query).await {
							log::error!("Failed to execute query: {err}");
						}
					});
				});
			}
			Err(err) => {
				log::error!("Failed to read database pool: {err}");
			}
		}

			// match state.remote_games.read() {
			// 	Ok(remote_games) => {
			// 		// Assign the remote game here as we find the new game.
			// 		// This is for when the remote games are fetched *before* games are found locally.
			// 		game.remote_game = remote_games
			// 			.get(&game.id.provider_id)
			// 			.and_then(|provider_remote_games| {
			// 				provider_remote_games.get(&game.external_id)
			// 			})
			// 			.or_else(|| {
			// 				remote_games.get(&ProviderId::Manual).and_then(
			// 					|provider_remote_games| {
			// 						game.title.normalized.first().and_then(|normalized_title| {
			// 							provider_remote_games.get(normalized_title)
			// 						})
			// 					},
			// 				)
			// 			})
			// 			.cloned();
			// 	}
			// 	Err(err) => {
			// 		log::error!("Failed to read remote games state: {err}");
			// 	}
			// }

			if let Some(provider_games) = handle.app_state().games.get(&provider_id) {
				match provider_games.write() {
					Ok(mut provider_games_write) => {
						provider_games_write.insert(game.id.game_id.clone(), game.clone());
						handle.emit_safe(events::FoundGame(game.id.clone()));
						handle.emit_safe(events::GamesChanged());
						fresh_games.insert(game.id.game_id.clone(), game);
					}
					Err(err) => {
						log::error!("Failed to write games state: {err}");
					}
				}
			}
		})
		.await
		.unwrap_or_else(|err| {
			// It's normal for a provider to fail here if that provider is just missing.
			// So we log those errors here instead of throwing them up.
			log::warn!(
				"Failed to get games for provider {provider_id}. User might just not have it. Error: {err}"
			);
		});

	// After all is done, write again with fresh games, to get rid of the stale ones.
	let mut games_write = state.games.try_get(&provider_id)?.write_state()?;
	*games_write = fresh_games.clone();

	provider_cache::write(provider_id, &fresh_games);

	Ok(())
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
		.try_get(&ProviderId::Manual)?
		.write_state()?
		.insert(game.id.game_id.clone(), game.clone());

	handle.emit_safe(events::FoundGame(game.id.clone()));

	handle.emit_safe(events::GamesChanged());

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

	refresh_games(handle, ProviderId::Manual).await
}

#[tauri::command]
#[specta::specta]
async fn run_provider_command(
	provider_command: ProviderCommand,
	handle: AppHandle,
) -> Result {
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
async fn get_game_ids(
    handle: AppHandle,
    query: Option<GamesQuery>,
) -> Result<GameIdsResponse> {
    let state = handle.app_state();
    let pool = state.database_pool.read_state()?.clone();
    let search = query.as_ref().map(|q| q.search.clone()).unwrap_or_default();

    // Build sorting logic
    let sort_columns = match query.as_ref().map(|q| q.sort_by) {
        Some(GamesSortBy::Title) => vec!["g.display_title"],
        Some(GamesSortBy::ReleaseDate) => vec!["g.release_date"],
        Some(GamesSortBy::Engine) => vec!["COALESCE(ig.engine_brand, rg.engine_brand)", "COALESCE(ig.engine_version, rg.engine_version)"],
        _ => vec!["g.display_title"],
    };

    let sort_order = if query.as_ref().is_some_and(|q| q.sort_descending) {
        "DESC"
    } else {
        "ASC"
    };

    // Build filtering logic dynamically
    let mut filters = Vec::<String>::new();

    if let Some(filter) = query.as_ref().map(|q| &q.filter) {
        // Installed filter
        if filter.installed.contains(&Some(InstallState::Installed)) {
            filters.push("g.installed_game IS NOT NULL".to_string());
        } else if filter.installed.contains(&Some(InstallState::NotInstalled)) {
            filters.push("g.installed_game IS NULL".to_string());
        }

        if !filter.providers.is_empty() {
            let provider_conditions: Vec<String> = filter
                .providers
                .iter()
                .filter_map(|provider| provider.as_ref().map(|p| format!("g.provider_id = '{}'", p)))
                .collect();
            if !provider_conditions.is_empty() {
                filters.push(format!("({})", provider_conditions.join(" OR ")));
            }
        }

				// TODO: tag filtering expectation is weird, probably want to make sure disabled tags never show up.
        if !filter.tags.is_empty() {
            let tag_conditions: Vec<String> = filter
                .tags
                .iter()
                .map(|tag| if let Some(t) = tag {
									format!("g.tags LIKE '%\"{}\"%'", t)
								} else {
									format!("g.tags = '[]'")
								})
                .collect();
            if !tag_conditions.is_empty() {
                filters.push(format!("({})", tag_conditions.join(" OR ")));
            }
        }

				if !filter.engines.is_empty() {
					let mut engine_conditions = Vec::new();
			
					// Check if None is in the filter.engines
					if filter.engines.contains(&None) {
							engine_conditions.push("COALESCE(ig.engine_brand, rg.engine_brand) IS NULL".to_string());
					}
			
					// Collect all non-None values and use the IN clause
					let engine_values: Vec<String> = filter
							.engines
							.iter()
							.filter_map(|engine| engine.as_ref().map(|e| format!("'{}'", e)))
							.collect();
			
					if !engine_values.is_empty() {
							engine_conditions.push(format!(
									"COALESCE(ig.engine_brand, rg.engine_brand) IN ({})",
									engine_values.join(", ")
							));
					}
			
					if !engine_conditions.is_empty() {
							filters.push(format!("({})", engine_conditions.join(" OR ")));
					}
			}

				if !filter.unity_scripting_backends.is_empty() {
						let backend_conditions: Vec<String> = filter
								.unity_scripting_backends
								.iter()
								.filter_map(|backend| backend.as_ref().map(|b| format!("ig.unity_backend = '{}'", b)))
								.collect();
						if !backend_conditions.is_empty() {
								filters.push(format!("({})", backend_conditions.join(" OR ")));
						}
				}
    }

    // Add search filter
    if !search.trim().is_empty() {
        filters.push("(g.display_title LIKE '%' || $1 || '%' OR g.normalized_titles LIKE '%' || $1 || '%')".to_string());
    }

    // Combine all filters into a single WHERE clause
    let where_clause = if filters.is_empty() {
        "1=1".to_string() // No filters, match all rows
    } else {
        filters.join(" AND ")
    };

		log::info!("#### WHERE clause: {where_clause}");

		let query = &format!(
			r#"
			SELECT DISTINCT
					g.provider_id as provider_id,
					g.game_id as game_id
			FROM main.games g
			LEFT JOIN main.installed_games ig ON g.installed_game = ig.id
			LEFT JOIN remote_games rg ON (
					g.provider_id = rg.provider_id AND g.external_id = rg.external_id
			) OR (
					rg.provider_id = 'Manual' AND g.normalized_titles LIKE '%"' || rg.external_id || '"%'
			)
			WHERE {where_clause}
			ORDER BY {}
			"#,
			sort_columns
					.iter()
					.map(|col| format!("{col} {sort_order}"))
					.collect::<Vec<_>>()
					.join(", ")
	);

	log::info!("#### Query: {query}");

    let game_ids: Vec<GameId> = sqlx::query_as(
			query
    )
    .bind(search.trim())
    .fetch_all(&pool)
    .await?;

    let total_count: i64 = sqlx::query(r#"
			SELECT COUNT(*)
			FROM main.games g
		"#)
    .fetch_one(&pool)
    .await?
    .try_get(0)?;

    Ok(GameIdsResponse {
        game_ids,
        total_count,
    })
}

#[tauri::command]
#[specta::specta]
async fn get_game(id: GameId, handle: AppHandle) -> Result<DbGame> {
	let state = handle.app_state();
	let pool = state.database_pool.read_state()?.clone();

	// TODO take into account all normalized titles
	let db_game: DbGame = sqlx::query_as(r#"
		SELECT
			g.*,
			ig.*,
			COALESCE(rg.engine_brand, ig.engine_brand) AS engine_brand,
			COALESCE(rg.engine_version, ig.engine_version) AS engine_version
		FROM main.games g
		LEFT JOIN main.installed_games ig ON g.installed_game = ig.id
		LEFT JOIN remote_games rg ON (
				g.provider_id = rg.provider_id AND g.external_id = rg.external_id
		) OR (
				rg.provider_id = 'Manual' AND g.normalized_titles LIKE '%"' || rg.external_id || '"%'
		)
		WHERE g.provider_id = $1 AND g.game_id = $2
		LIMIT 1
	"#)
	.bind(id.provider_id)
	.bind(&id.game_id)
	.fetch_one(&pool)
	.await?;

	// let mut game = Game::new(id.clone(), &db_game.display_title);
	// game.release_date = db_game.release_date;
	// game.thumbnail_url = db_game.thumbnail_url;
	// game.remote_game = db_game.engine_brand.map(|engine_brand| RemoteGame {
	// 	title: None,
	// 	engine: Some(GameEngine {
	// 		version: None,
	// 		brand: engine_brand,
	// 	}),
	// 	ids: HashMap::default(),
	// 	subscriptions: None,
	// });
	// game.installed_game = db_game.exe_path.map(|exe_path| InstalledGame {
	// 	id: exe_path.clone(),
	// 	installed_mod_versions: HashMap::default(),
	// 	discriminator: None,
	// 	start_command: None,
	// 	executable: GameExecutable {
	// 		architecture: None,
	// 		engine: db_game.engine_brand.map(|engine_brand| GameEngine {
	// 			brand: engine_brand,
	// 			// TODO: properly store entire version information.
	// 			version: db_game.engine_version.map(|engine_version| EngineVersion {
	// 				display: engine_version.to_string(),
	// 				suffix: None,
	// 				numbers: EngineVersionNumbers {
	// 					major: 0,
	// 					minor: None,
	// 					patch: None,
	// 				},
	// 			}),
	// 		}),
	// 		path: PathBuf::from(exe_path),
	// 		scripting_backend: None,
	// 	}
	// });
	// game.tags = db_game.tags.0;
	// let tags_str: &str = row.get("tags");
	// game.tags = tags_str.split(',').map(|s| s.trim().to_string()).collect();

	Ok(db_game)
}

#[tauri::command]
#[specta::specta]
async fn clear_cache() -> Result {
	provider_cache::clear()?;
	Ok(())
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

fn main() {
	// Since I'm making all exposed functions async, panics won't crash anything important, I think.
	// So I can just catch panics here and show a system message with the error.
	std::panic::set_hook(Box::new(|info| {
		#[cfg(target_os = "windows")]
		windows::error_dialog(&format!("I found a panic!!!: {info}"));

		log::error!("Panic: {info}");
	}));

	let database_pool = tauri::async_runtime::block_on(setup_database()).unwrap();

	let builder = Builder::<tauri::Wry>::new()
		.commands(tauri_specta::collect_commands![
			add_game,
			clear_cache,
			configure_mod,
			delete_mod,
			download_mod,
			frontend_ready,
			get_app_settings,
			get_game_ids,
			get_game,
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
			start_game_exe,
			start_game,
			uninstall_all_mods,
			uninstall_mod,
		])
		.events(events::collect_events())
		.constant("PROVIDER_IDS", ProviderId::variants())
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
		.manage(AppState {
			mod_loaders: RwLock::default(),
			local_mods: RwLock::default(),
			remote_mods: RwLock::default(),
			games: HashMap::from_iter(ProviderId::variants().iter().map(|&id| {
				(
					id,
					RwLock::new(provider_cache::read(id).unwrap_or_default()),
				)
			})),
			database_pool: RwLock::new(database_pool),
		})
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
