use std::{
	collections::HashMap,
	fs,
	path::PathBuf,
	time::{
		SystemTime,
		UNIX_EPOCH,
	},
};

use rusqlite::OpenFlags;
use serde::Serialize;

use crate::{
	app_paths,
	local_database::game_database::{
		DbMutex,
		GameDatabase,
	},
	local_database::rusqlite_extensions::RowExt,
	mods::game_mod::GameMod,
	result::Result,
};

pub trait ModDatabase {
	fn insert_mod(&self, game_mod: &GameMod);
	fn get_mod(&self, mod_id: &str) -> Result<GameMod>;
	fn get_mod_map(&self) -> Result<HashMap<String, GameMod>>;
	fn remove_stale_mods(&self, max_time: u64) -> Result;
}

impl ModDatabase for DbMutex {
	fn insert_mod(&self, game_mod: &GameMod) {
		if let Err(err) = try_insert_mod(self, game_mod) {
			log::error!(
				"Failed to insert mod ({}) into local database: {}",
				game_mod.id,
				err
			);
		}
	}

	fn get_mod(&self, mod_id: &str) -> Result<GameMod> {
		Ok(self
			.lock_db()?
			.prepare_cached(
				r"
			SELECT
				id,
				title,
				author,
				source_code,
				description,
				download,
				engine,
				engine_version_range,
				unity_backend,
				architecture,
				game_os,
				host_os,
				deprecated,
				config,
				dependencies,
				install,
				run_for_game,
				hash
			FROM main.mods
			WHERE id = $1
			LIMIT 1
		",
			)?
			.query_row([mod_id], |row| {
				Ok(GameMod {
					id: row.get(0)?,
					title: row.get(1)?,
					author: row.get(2)?,
					source_code: row.get(3)?,
					description: row.get(4)?,
					download: row.get_json(5)?,
					engine: row.get_json(6)?,
					engine_version_range: row.get_json(7)?,
					unity_backend: row.get_json(8)?,
					architecture: row.get_json(9)?,
					game_os: row.get_json(10)?,
					host_os: row.get_json(11)?,
					deprecated: row.get(12)?,
					config: row.get_json(13)?,
					dependencies: row.get_json(14)?,
					install: row.get_json(15)?,
					run_for_game: row.get_json(16)?,
					hash: row.get(17)?,
				})
			})?)
	}

	fn get_mod_map(&self) -> Result<HashMap<String, GameMod>> {
		Ok(self
			.lock_db()?
			.prepare_cached(
				r"
			SELECT
				id,
				title,
				author,
				source_code,
				description,
				download,
				engine,
				engine_version_range,
				unity_backend,
				architecture,
				game_os,
				host_os,
				deprecated,
				config,
				dependencies,
				install,
				run_for_game,
				hash
			FROM main.mods
		",
			)?
			.query_map([], |row| {
				Ok(GameMod {
					id: row.get(0)?,
					title: row.get(1)?,
					author: row.get(2)?,
					source_code: row.get(3)?,
					description: row.get(4)?,
					download: row.get_json(5)?,
					engine: row.get_json(6)?,
					engine_version_range: row.get_json(7)?,
					unity_backend: row.get_json(8)?,
					architecture: row.get_json(9)?,
					game_os: row.get_json(10)?,
					host_os: row.get_json(11)?,
					deprecated: row.get(12)?,
					config: row.get_json(13)?,
					dependencies: row.get_json(14)?,
					install: row.get_json(15)?,
					run_for_game: row.get_json(16)?,
					hash: row.get(17)?,
				})
			})?
			.filter_map(|game_mod| match game_mod {
				Ok(parsed) => Some((parsed.id.clone(), parsed)),
				Err(err) => {
					log::warn!("Failed to read mod from local database: {err}");
					None
				}
			})
			.collect())
	}

	fn remove_stale_mods(&self, max_time: u64) -> Result {
		self.lock_db()?
			.prepare_cached("DELETE FROM main.mods WHERE created_at < $1;")?
			.execute(rusqlite::params![max_time.cast_signed()])?;

		Ok(())
	}
}

fn serialize_json_option<T: Serialize>(value: &Option<T>) -> Result<String> {
	serde_json::to_string(value).map_err(Into::into)
}

fn try_insert_mod(connection_mutex: &DbMutex, game_mod: &GameMod) -> Result {
	let download = serialize_json_option(&game_mod.download)?;
	let engine = serialize_json_option(&game_mod.engine)?;
	let engine_version_range = serialize_json_option(&game_mod.engine_version_range)?;
	let unity_backend = serialize_json_option(&game_mod.unity_backend)?;
	let architecture = serialize_json_option(&game_mod.architecture)?;
	let game_os = serialize_json_option(&game_mod.game_os)?;
	let host_os = serialize_json_option(&game_mod.host_os)?;
	let config = serialize_json_option(&game_mod.config)?;
	let dependencies = serialize_json_option(&game_mod.dependencies)?;
	let install = serialize_json_option(&game_mod.install)?;
	let run_for_game = serialize_json_option(&game_mod.run_for_game)?;

	connection_mutex
		.lock_db()?
		.prepare_cached(
			"INSERT OR REPLACE INTO mods (
				id,
				title,
				author,
				source_code,
				description,
				download,
				engine,
				engine_version_range,
				unity_backend,
				architecture,
				game_os,
				host_os,
				deprecated,
				config,
				dependencies,
				install,
				run_for_game,
				hash,
				created_at
			) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19)",
		)?
		.execute(rusqlite::params![
			game_mod.id,
			game_mod.title,
			game_mod.author,
			game_mod.source_code,
			game_mod.description,
			download,
			engine,
			engine_version_range,
			unity_backend,
			architecture,
			game_os,
			host_os,
			game_mod.deprecated,
			config,
			dependencies,
			install,
			run_for_game,
			game_mod.hash,
			SystemTime::now()
				.duration_since(UNIX_EPOCH)?
				.as_secs()
				.cast_signed()
		])?;

	Ok(())
}

pub fn try_create() -> Result<DbMutex> {
	match create() {
		Ok(db) => Ok(db),
		Err(initial_error) => {
			log::error!(
				"Failed to set up mod database. Deleting it and retrying once. Error: {initial_error}"
			);
			cleanup_database_file();

			match create() {
				Ok(db) => Ok(db),
				Err(retry_error) => {
					log::error!(
						"Failed to set up mod database after cleanup retry. Giving up. Error: {retry_error}"
					);
					Err(retry_error)
				}
			}
		}
	}
}

fn cleanup_database_file() {
	let path = match db_file_path() {
		Ok(path) => path,
		Err(path_error) => {
			log::warn!("Failed to resolve mod database path for cleanup: {path_error}");
			return;
		}
	};

	match fs::remove_file(&path) {
		Ok(()) => {}
		Err(remove_error) if remove_error.kind() == std::io::ErrorKind::NotFound => {}
		Err(remove_error) => {
			log::warn!(
				"Failed to delete mod database after create failure ({}): {remove_error}",
				path.display()
			);
		}
	}
}

fn create() -> Result<DbMutex> {
	let path = db_file_path()?;
	if let Some(parent) = path.parent() {
		std::fs::create_dir_all(parent)?;
	}

	let connection = rusqlite::Connection::open_with_flags(
		path,
		OpenFlags::SQLITE_OPEN_CREATE
			| OpenFlags::SQLITE_OPEN_READ_WRITE
			| OpenFlags::SQLITE_OPEN_SHARED_CACHE,
	)?;

	connection.execute_batch(
		r"
		PRAGMA journal_mode = WAL;
		PRAGMA synchronous = OFF;

		CREATE TABLE IF NOT EXISTS mods (
			id TEXT NOT NULL,
			title TEXT NOT NULL,
			author TEXT NOT NULL,
			source_code TEXT NOT NULL,
			description TEXT NOT NULL,
			download TEXT,
			engine TEXT,
			engine_version_range TEXT,
			unity_backend TEXT,
			architecture TEXT,
			game_os TEXT,
			host_os TEXT,
			deprecated INTEGER,
			config TEXT,
			dependencies TEXT,
			install TEXT,
			run_for_game TEXT,
			hash TEXT,
			created_at INTEGER,
			PRIMARY KEY (id)
		);

		CREATE INDEX IF NOT EXISTS idx_mods_created_at ON mods(created_at);
	",
	)?;

	Ok(DbMutex::new(connection))
}

fn db_file_path() -> Result<PathBuf> {
	app_paths::database_path("mods")
}