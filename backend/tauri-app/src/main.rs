// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
// Command stuff needs to be async so I can spawn tasks.
#![allow(clippy::unused_async)]

use std::ops::Deref;
use std::path::Path;
use std::sync::RwLock;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{collections::HashMap, path::PathBuf};

use crate::result::{Error, Result};
use app_settings::AppSettings;
use app_state::{AppState, StateData, StatefulHandle};
use events::EventEmitter;
use futures::{StreamExt, TryStreamExt};
use rai_pal_core::game::{self, DbGame, GameId, InsertGame};
use rai_pal_core::game_engines::game_engine::{
	EngineBrand, EngineVersion, EngineVersionNumbers, GameEngine,
};
use rai_pal_core::game_executable::GameExecutable;
use rai_pal_core::game_title::get_normalized_titles;
use rai_pal_core::games_query::{GamesQuery, GamesSortBy, InstallState};
use rai_pal_core::local_mod::{self, LocalMod};
use rai_pal_core::maps::TryGettable;
use rai_pal_core::mod_loaders::mod_loader::{self, ModLoaderActions};
use rai_pal_core::paths::{self, AsValidStr, normalize_path};
use rai_pal_core::providers::provider::ProviderId;
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
use rusqlite::OpenFlags;
use strum::IntoEnumIterator;
use tauri::async_runtime::Mutex;
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
	let game = get_game(game_id, handle.clone()).await?;
	game.open_game_folder()?;
	Ok(())
}

#[tauri::command]
#[specta::specta]
async fn open_game_mods_folder(handle: AppHandle, game_id: GameId) -> Result {
	let game = get_game(game_id, handle.clone()).await?;
	game.open_mods_folder()?;
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

fn refresh_game_mods(game_id: &GameId, handle: &AppHandle) -> Result {
	// TODO: refresh game mods
	// let state = handle.app_state();
	// let mut games = state.games.try_get(&game_id.provider_id)?.write_state()?;
	// let game = games.try_get_mut(&game_id.game_id)?;
	// let installed_game = game.try_get_installed_game_mut()?;
	// installed_game.refresh_installed_mods();

	// handle.emit_safe(events::FoundGame(game_id.clone()));

	Ok(())
}

#[tauri::command]
#[specta::specta]
async fn install_mod(game_id: GameId, mod_id: &str, handle: AppHandle) -> Result {
	let state = handle.app_state();
	let game = get_game(game_id.clone(), handle.clone()).await?;

	let mod_loaders = state.mod_loaders.read_state()?.clone();

	let local_mod = refresh_and_get_local_mod(mod_id, &mod_loaders, &handle).await?;

	let mod_loader = mod_loaders.try_get(&local_mod.common.loader_id)?;

	// Uninstall mod if it already exists, in case there are conflicting leftover files when updating.
	mod_loader.uninstall_mod(&game, &local_mod).await?;

	mod_loader.install_mod(&game, &local_mod).await?;

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
	let game = get_game(game_id.clone(), handle.clone()).await?;

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
	let game = get_game(game_id.clone(), handle.clone()).await?;

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
	let game = get_game(game_id.clone(), handle.clone()).await?;

	// TODO refresh game.
	// if let Some(installed_game) = game.installed_game.as_mut() {
	// 	installed_game.refresh_installed_mods();
	// 	installed_game.refresh_executable()?;
	// }

	// handle.emit_safe(events::FoundGame(game_id));

	Ok(())
}

#[tauri::command]
#[specta::specta]
async fn uninstall_mod(game_id: GameId, mod_id: &str, handle: AppHandle) -> Result {
	let state = handle.app_state();
	let game = get_game(game_id.clone(), handle.clone()).await?;

	let mod_loaders = state.mod_loaders.read_state()?.clone();

	let local_mod = refresh_and_get_local_mod(mod_id, &mod_loaders, &handle).await?;

	let mod_loader = mod_loaders.try_get(&local_mod.common.loader_id)?;

	// Uninstall mod if it already exists, in case there are conflicting leftover files when updating.
	mod_loader.uninstall_mod(&game, &local_mod).await?;

	refresh_game_mods(&game_id, &handle)?;

	Ok(())
}

#[tauri::command]
#[specta::specta]
async fn uninstall_all_mods(game_id: GameId, handle: AppHandle) -> Result {
	let state = handle.app_state();
	let game = get_game(game_id.clone(), handle.clone()).await?;

	game.uninstall_all_mods()?;

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
	let path = remote_game::download_database().await?;
	attach_remote_database(state.database_connection.lock().unwrap(), &path)?;

	Ok(())
}

fn attach_remote_database<TConnection: Deref<Target = rusqlite::Connection>>(
	local_database_connection: TConnection,
	path: &Path,
) -> Result {
	println!("Attaching remote database...");

	if !path.is_file() {
		return Ok(());
	}

	let remote_database_connection =
		rusqlite::Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;

	let mut insert_into_local = local_database_connection.prepare(
		r#"
		INSERT OR IGNORE INTO main.remote_games (
				provider_id, external_id, engine_brand, engine_version_major,
				engine_version_minor, engine_version_patch, engine_version_display, subscriptions
		)
		VALUES (?, ?, ?, ?, ?, ?, ?, ?);
		"#,
	)?;

	remote_database_connection
		.prepare("SELECT * FROM games;")?
		.query_map([], |row| {
			let provider_id: ProviderId = row.get("provider_id")?;
			let external_id: String = row.get("external_id")?;
			let engine_brand: Option<EngineBrand> = row.get("engine_brand")?;
			let engine_version_display: Option<String> = row.get("engine_version")?;
			let subscriptions: Option<String> = row.get("subscriptions")?;
			let engine_version_string: Option<String> = row.get("engine_version")?;

			let engine_version = if let Some(engine_version) = engine_version_string {
				remote_game::parse_version(&engine_version)
			} else {
				None
			};

			// Insert the processed game into main.remote_games
			insert_into_local.execute(rusqlite::params![
				provider_id,
				external_id,
				engine_brand,
				engine_version.as_ref().map(|v| v.numbers.major),
				engine_version.as_ref().map(|v| v.numbers.minor),
				engine_version.as_ref().map(|v| v.numbers.patch),
				engine_version_display,
				subscriptions,
			])?;

			Ok(())
		})?
		.for_each(|result| {
			if let Err(err) = result {
				log::error!("Error processing row: {err}");
			}
		});

	println!("Remote database attached!");

	Ok(())
}

pub fn setup_rusqlite() -> Result<rusqlite::Connection> {
	let path = paths::app_data_path()?.join("db.sqlite");
	if path.is_file() {
		std::fs::remove_file(&path)?;
	}

	let connection = rusqlite::Connection::open_with_flags(
		path,
		OpenFlags::SQLITE_OPEN_CREATE
			| OpenFlags::SQLITE_OPEN_READ_WRITE
			| OpenFlags::SQLITE_OPEN_SHARED_CACHE,
	)?;

	connection.execute_batch(
		r#"
		PRAGMA journal_mode = WAL;
		PRAGMA synchronous = OFF;

		CREATE TABLE IF NOT EXISTS games (
			provider_id TEXT NOT NULL,
			game_id TEXT NOT NULL,
			external_id TEXT NOT NULL,
			display_title TEXT NOT NULL,
			title_discriminator TEXT,
			thumbnail_url TEXT,
			tags TEXT,
			release_date INTEGER,
			provider_commands TEXT,
			created_at INTEGER,
			PRIMARY KEY (provider_id, game_id)
		);

		CREATE INDEX IF NOT EXISTS idx_games_external_id ON games(provider_id, external_id);

		CREATE TABLE IF NOT EXISTS normalized_titles (
			provider_id TEXT NOT NULL,
			game_id TEXT NOT NULL,
			normalized_title TEXT NOT NULL,
			FOREIGN KEY (provider_id, game_id) REFERENCES games(provider_id, game_id) ON DELETE CASCADE,
			PRIMARY KEY (provider_id, game_id, normalized_title)
		);

		CREATE INDEX IF NOT EXISTS idx_normalized_titles ON normalized_titles(provider_id, game_id);

		CREATE TABLE IF NOT EXISTS installed_games (
			provider_id TEXT NOT NULL,
			game_id TEXT NOT NULL,
			exe_path TEXT NOT NULL,
			engine_brand TEXT,
			engine_version_major INTEGER,
			engine_version_minor INTEGER,
			engine_version_patch INTEGER,
			engine_version_display TEXT,
			unity_backend TEXT,
			architecture TEXT,
			FOREIGN KEY(provider_id, game_id) REFERENCES games(provider_id, game_id) ON DELETE CASCADE,
			PRIMARY KEY (provider_id, game_id)
		);

		CREATE TABLE IF NOT EXISTS remote_games (
			provider_id TEXT NOT NULL,
			external_id TEXT NOT NULL,
			engine_brand TEXT,
			engine_version_major INTEGER,
			engine_version_minor INTEGER,
			engine_version_patch INTEGER,
			engine_version_display TEXT,
			subscriptions TEXT,
			PRIMARY KEY (provider_id, external_id)
		);
	"#,
	)?;
	attach_remote_database(&connection, &remote_game::get_database_file_path()?)?;

	Ok(connection)
}

#[tauri::command]
#[specta::specta]
async fn refresh_games(handle: AppHandle, provider_id: ProviderId) -> Result {
	let state = handle.app_state();

	let now = SystemTime::now()
		.duration_since(UNIX_EPOCH)
		.unwrap()
		.as_secs();

	let provider = provider::get_provider(provider_id)?;

	provider.insert_games(&state.database_connection).await?;

	// Remove stale games
	state
		.database_connection
		.lock()
		.unwrap()
		.prepare("DELETE FROM main.games WHERE provider_id = $1 AND created_at < $2;")?
		.execute(rusqlite::params![provider_id, now])?;

	Ok(())
}

#[tauri::command]
#[specta::specta]
async fn add_game(path: PathBuf, handle: AppHandle) -> Result {
	let normalized_path = normalize_path(&path);

	let game = manual_provider::add_game(&normalized_path)?;
	let game_name = game.display_title.clone();

	let state = handle.app_state();

	state.database_connection.lock().unwrap().insert_game(&game);

	handle.emit_safe(events::FoundGame(GameId {
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
async fn remove_game(path: PathBuf, handle: AppHandle) -> Result {
	manual_provider::remove_game(&path)?;

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
	let database_connection = state.database_connection.lock().unwrap();
	let search = query.as_ref().map(|q| q.search.clone()).unwrap_or_default();

	// Build sorting logic
	let sort_columns = match query.as_ref().map(|q| q.sort_by) {
		Some(GamesSortBy::Title) => vec!["g.display_title"],
		Some(GamesSortBy::ReleaseDate) => vec!["g.release_date"],
		Some(GamesSortBy::Engine) => vec![
			"COALESCE(ig.engine_brand, rg.engine_brand)",
			"COALESCE(ig.engine_version_major, rg.engine_version_major)",
			"COALESCE(ig.engine_version_minor, rg.engine_version_minor)",
			"COALESCE(ig.engine_version_patch, rg.engine_version_patch)",
		],
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
			filters.push("ig.exe_path IS NOT NULL".to_string());
		} else if filter.installed.contains(&Some(InstallState::NotInstalled)) {
			filters.push("ig.exe_path IS NULL".to_string());
		}

		if !filter.providers.is_empty() {
			let provider_conditions: Vec<String> = filter
				.providers
				.iter()
				.filter_map(|provider| {
					provider
						.as_ref()
						.map(|p| format!("g.provider_id = '{}'", p))
				})
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
				.map(|tag| {
					if let Some(t) = tag {
						format!("g.tags LIKE '%\"{}\"%'", t)
					} else {
						format!("g.tags = '[]'")
					}
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
				engine_conditions
					.push("COALESCE(ig.engine_brand, rg.engine_brand) IS NULL".to_string());
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

		if !filter.unity_backends.is_empty() {
			let backend_conditions: Vec<String> = filter
				.unity_backends
				.iter()
				.filter_map(|backend| {
					backend
						.as_ref()
						.map(|b| format!("ig.unity_backend = '{}'", b))
				})
				.collect();
			if !backend_conditions.is_empty() {
				filters.push(format!("({})", backend_conditions.join(" OR ")));
			}
		}
	}

	let trimmed_search = search.trim();
	// Add search filter
	if !trimmed_search.is_empty() {
		filters.push(format!(
			"(g.display_title LIKE '%{}%' OR nt.normalized_title LIKE '%{}%')",
			trimmed_search, trimmed_search
		));
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
			LEFT JOIN main.installed_games ig ON g.provider_id = ig.provider_id AND g.game_id = ig.game_id
			LEFT JOIN main.normalized_titles nt ON g.provider_id = nt.provider_id AND g.game_id = nt.game_id
			LEFT JOIN remote_games rg ON (
					g.provider_id = rg.provider_id AND g.external_id = rg.external_id
			) OR (
					rg.provider_id = 'Manual' AND nt.normalized_title = rg.external_id
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

	let game_ids = database_connection
		.prepare(query)?
		.query_map([], |row| {
			Ok(GameId {
				provider_id: row.get(0)?,
				game_id: row.get(1)?,
			})
		})?
		.filter_map(|game_id| game_id.ok()) // TODO log errors.
		.collect();

	let total_count = database_connection
		.prepare(
			r#"
			SELECT COUNT(*)
			FROM main.games g
		"#,
		)?
		.query_row([], |row| Ok(row.get::<_, i64>(0)?))?;

	Ok(GameIdsResponse {
		game_ids,
		total_count,
	})
}

#[tauri::command]
#[specta::specta]
async fn get_game(id: GameId, handle: AppHandle) -> Result<DbGame> {
	let state = handle.app_state();
	let database_connection = state.database_connection.lock().unwrap();

	// TODO take into account all normalized titles
	let db_game = database_connection
		.prepare(
			r#"
		SELECT
			g.provider_id,
			g.game_id,
			g.external_id,
			g.display_title,
			g.title_discriminator,
			g.thumbnail_url,
			g.release_date,
			g.tags,
			g.provider_commands,
			ig.exe_path,
			ig.unity_backend,
			ig.architecture,
			COALESCE(ig.engine_brand, rg.engine_brand) AS engine_brand,
			COALESCE(ig.engine_version_major, rg.engine_version_major) AS engine_version_major,
			COALESCE(ig.engine_version_minor, rg.engine_version_minor) AS engine_version_minor,
			COALESCE(ig.engine_version_patch, rg.engine_version_patch) AS engine_version_patch,
			COALESCE(ig.engine_version_display, rg.engine_version_display) AS engine_version_display
		FROM main.games g
		LEFT JOIN main.installed_games ig ON g.provider_id = ig.provider_id AND g.game_id = ig.game_id
		LEFT JOIN main.normalized_titles nt ON g.provider_id = nt.provider_id AND g.game_id = nt.game_id
		LEFT JOIN remote_games rg ON (
				g.provider_id = rg.provider_id AND g.external_id = rg.external_id
		) OR (
				rg.provider_id = 'Manual' AND nt.normalized_title = rg.external_id
		)
		WHERE g.provider_id = $1 AND g.game_id = $2
		LIMIT 1
	"#,
		)?
		.query_row([id.provider_id.to_string(), id.game_id], |row| {
			Ok(DbGame {
				provider_id: row.get(0)?,
				game_id: row.get(1)?,
				external_id: row.get(2)?,
				display_title: row.get(3)?,
				title_discriminator: row.get(4)?,
				thumbnail_url: row.get(5)?,
				release_date: row.get(6)?,
				tags: row.get(7)?,
				provider_commands: row.get(8)?,
				exe_path: row.get(9)?,
				unity_backend: row.get(10)?,
				architecture: row.get(11)?,
				engine_brand: row.get(12)?,
				engine_version_major: row.get(13)?,
				engine_version_minor: row.get(14)?,
				engine_version_patch: row.get(15)?,
				engine_version_display: row.get(16)?,
			})
		})?;

	Ok(db_game)
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
	let game = get_game(game_id.clone(), app_handle.clone()).await?;
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

	let database_connection = setup_rusqlite().unwrap(); // TODO handle error

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
		.manage(AppState {
			mod_loaders: RwLock::default(),
			local_mods: RwLock::default(),
			remote_mods: RwLock::default(),
			database_connection: std::sync::Mutex::new(database_connection),
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

			let app_handle = app.app_handle().clone();

			tauri::async_runtime::spawn(async move {
				let state = app_handle.app_state();
				let database_connection = state.database_connection.lock().unwrap();
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
