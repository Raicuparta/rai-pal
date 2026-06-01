use std::{
	collections::BTreeMap,
	ffi::OsStr,
	path::Path,
	string,
	time::{
		SystemTime,
		UNIX_EPOCH,
	},
};

use rai_pal_proc_macros::serializable_struct;
use serde::Serialize;

use crate::{
	app_paths,
	game_providers::game_provider::GameProviderId,
	local_database::{
		game_database::{
			DbMutex,
			GameDatabase,
		},
		rusqlite_extensions::RowExt,
	},
	mods::game_mod::GameMod,
	path_extensions::PathExt,
	result::Result,
};

#[serializable_struct]
pub struct GameModInfo {
	pub mod_id: String,
	pub installed_version: Option<String>,
	pub installed_hash: Option<String>,
	pub compatible: bool,
}

pub trait ModDatabase {
	fn setup_mod_tables(&self) -> Result;
	fn insert_mod(&self, game_mod: &GameMod);
	fn get_mod(&self, mod_id: &str) -> Result<GameMod>;
	fn get_mod_map(&self) -> Result<BTreeMap<String, GameMod>>;
	fn refresh_installed_mods(&self) -> Result;
	fn get_game_mods(
		&self,
		provider_id: &GameProviderId,
		game_id: &str,
	) -> Result<Vec<GameModInfo>>;
	fn remove_stale_mods(&self, max_time: u64) -> Result;
}

impl ModDatabase for DbMutex {
	fn setup_mod_tables(&self) -> Result {
		self.lock_db()?.execute_batch(
			r"
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

			DROP TABLE IF EXISTS installed_mods;

			CREATE TABLE IF NOT EXISTS installed_mods (
				exe_path_hash TEXT NOT NULL,
				mod_id TEXT NOT NULL,
				installed_version TEXT,
				installed_hash TEXT,
				created_at INTEGER,
				PRIMARY KEY (exe_path_hash, mod_id)
			);

			CREATE INDEX IF NOT EXISTS idx_installed_mods_hash ON installed_mods(exe_path_hash);
			CREATE INDEX IF NOT EXISTS idx_installed_mods_mod_id ON installed_mods(mod_id);
		",
		)?;

		Ok(())
	}

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

	fn get_mod_map(&self) -> Result<BTreeMap<String, GameMod>> {
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

	fn refresh_installed_mods(&self) -> Result {
		let installed_mods_root = app_paths::installed_mods_path()?;
		let manifests_glob = installed_mods_root
			.join("*")
			.join("manifests")
			.join("*.json");

		let created_at = SystemTime::now()
			.duration_since(UNIX_EPOCH)?
			.as_secs()
			.cast_signed();

		{
			let mut connection = self.lock_db()?;
			let transaction = connection.transaction()?;

			transaction.execute("DELETE FROM main.installed_mods;", [])?;

			{
				let mut statement = transaction.prepare_cached(
					"INSERT OR REPLACE INTO main.installed_mods (
						exe_path_hash,
						mod_id,
						installed_version,
						installed_hash,
						created_at
					) VALUES ($1, $2, $3, $4, $5)",
				)?;

				for manifest_path in manifests_glob.glob() {
					let Some(exe_path_hash) = manifest_path
						.parent()
						.and_then(Path::parent)
						.and_then(Path::file_name)
						.and_then(OsStr::to_str)
						.map(string::ToString::to_string)
					else {
						continue;
					};

					let Some(manifest) = GameMod::from_file(&manifest_path) else {
						continue;
					};

					statement.execute(rusqlite::params![
						exe_path_hash,
						manifest.id,
						manifest
							.download
							.as_ref()
							.map(|download| download.id.clone()),
						manifest.hash,
						created_at,
					])?;
				}
			}

			transaction.commit()?;

			// tbh only here due to a clippy warning,
			// Clippy does't seem to realize connection is needed as long as transaction.
			drop(connection);
		}

		Ok(())
	}

	fn get_game_mods(
		&self,
		provider_id: &GameProviderId,
		game_id: &str,
	) -> Result<Vec<GameModInfo>> {
		Ok(self
			.lock_db()?
			.prepare_cached(
				r"
			WITH candidate_mods AS (
				SELECT DISTINCT
					m.id AS mod_id,
					im.installed_version AS installed_version,
					im.installed_hash AS installed_hash,
					COALESCE(ig.engine_version_major, rg.engine_version_major) AS game_major,
					COALESCE(ig.engine_version_minor, rg.engine_version_minor) AS game_minor,
					COALESCE(ig.engine_version_patch, rg.engine_version_patch) AS game_patch,
					json_extract(m.engine_version_range, '$.minimum.major') AS min_major,
					json_extract(m.engine_version_range, '$.minimum.minor') AS min_minor,
					json_extract(m.engine_version_range, '$.minimum.patch') AS min_patch,
					json_extract(m.engine_version_range, '$.maximum.major') AS max_major,
					json_extract(m.engine_version_range, '$.maximum.minor') AS max_minor,
					json_extract(m.engine_version_range, '$.maximum.patch') AS max_patch
				FROM main.games g
				LEFT JOIN main.installed_games ig ON g.provider_id = ig.provider_id AND g.game_id = ig.game_id
				LEFT JOIN main.normalized_titles nt ON g.provider_id = nt.provider_id AND g.game_id = nt.game_id
				LEFT JOIN remote_games rg ON (
						g.provider_id = rg.provider_id AND g.external_id = rg.external_id
				) OR (
						rg.provider_id = 'Manual' AND nt.normalized_title = rg.external_id
				)
				INNER JOIN main.mods m ON 1=1
				LEFT JOIN main.installed_mods im ON
					g.exe_path_hash = im.exe_path_hash
					AND m.id = im.mod_id
				WHERE g.provider_id = $1
					AND g.game_id = $2
					AND (
						m.deprecated IS NULL
						OR m.deprecated = 0
						OR im.mod_id IS NOT NULL
					)
					AND (
						json_extract(m.engine, '$') IS NULL
						OR json_extract(m.engine, '$') = COALESCE(ig.engine_brand, rg.engine_brand)
					)
					AND (
						json_extract(m.unity_backend, '$') IS NULL
						OR ig.unity_backend IS NULL
						OR json_extract(m.unity_backend, '$') = ig.unity_backend
					)
					AND (
						json_extract(m.architecture, '$') IS NULL
						OR ig.architecture IS NULL
						OR json_extract(m.architecture, '$') = ig.architecture
					)
			)
			SELECT
				mod_id,
				installed_version,
				installed_hash,
				CASE
					WHEN game_major IS NULL THEN 1
					WHEN min_major IS NOT NULL AND (
						min_major > game_major
						OR (
							min_major = game_major
							AND min_minor IS NOT NULL
							AND game_minor IS NOT NULL
							AND (
								min_minor > game_minor
								OR (
									min_minor = game_minor
									AND min_patch IS NOT NULL
									AND game_patch IS NOT NULL
									AND min_patch > game_patch
								)
							)
						)
					) THEN 0
					WHEN max_major IS NOT NULL AND (
						max_major < game_major
						OR (
							max_major = game_major
							AND max_minor IS NOT NULL
							AND game_minor IS NOT NULL
							AND (
								max_minor < game_minor
								OR (
									max_minor = game_minor
									AND max_patch IS NOT NULL
									AND game_patch IS NOT NULL
									AND max_patch < game_patch
								)
							)
						)
					) THEN 0
					ELSE 1
				END AS compatible
			FROM candidate_mods
		",
			)?
			.query_map([provider_id.to_string(), game_id.to_string()], |row| {
				let mod_id = row.get::<_, String>(0)?;
				let installed_version = row.get::<_, Option<String>>(1)?;
				let installed_hash = row.get::<_, Option<String>>(2)?;
				let compatible = row.get::<_, bool>(3)?;

				Ok(GameModInfo {
					mod_id,
					installed_version,
					installed_hash,
					compatible,
				})
			})?
			.collect::<rusqlite::Result<Vec<GameModInfo>>>()?)
	}

	fn remove_stale_mods(&self, max_time: u64) -> Result {
		self.lock_db()?
			.prepare_cached("DELETE FROM main.mods WHERE created_at < $1;")?
			.execute(rusqlite::params![max_time.cast_signed()])?;

		Ok(())
	}
}

fn serialize_json_option<T: Serialize>(value: Option<&T>) -> Result<String> {
	serde_json::to_string(&value).map_err(Into::into)
}

fn try_insert_mod(connection_mutex: &DbMutex, game_mod: &GameMod) -> Result {
	let download = serialize_json_option(game_mod.download.as_ref())?;
	let engine = serialize_json_option(game_mod.engine.as_ref())?;
	let engine_version_range = serialize_json_option(game_mod.engine_version_range.as_ref())?;
	let unity_backend = serialize_json_option(game_mod.unity_backend.as_ref())?;
	let architecture = serialize_json_option(game_mod.architecture.as_ref())?;
	let game_os = serialize_json_option(game_mod.game_os.as_ref())?;
	let host_os = serialize_json_option(game_mod.host_os.as_ref())?;
	let config = serialize_json_option(game_mod.config.as_ref())?;
	let dependencies = serialize_json_option(game_mod.dependencies.as_ref())?;
	let install = serialize_json_option(game_mod.install.as_ref())?;
	let run_for_game = serialize_json_option(game_mod.run_for_game.as_ref())?;

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
