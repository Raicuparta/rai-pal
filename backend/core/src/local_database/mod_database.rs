use std::{
	collections::BTreeMap,
	time::{
		SystemTime,
		UNIX_EPOCH,
	},
};

use rai_pal_proc_macros::serializable_struct;
use serde::Serialize;

use crate::{
	game_providers::game_provider::GameProviderId,
	local_database::{
		app_database::{
			AppDatabase,
			DbMutex,
		},
		game_database::GameDatabase,
		rusqlite_extensions::RowExt,
	},
	mod_providers::mod_provider::ModProviderId,
	mods::{
		game_mod::GameMod,
		installed_mod::InstalledMod,
	},
	path_extensions::PathExt,
	result::{
		Error,
		Result,
	},
};

#[serializable_struct]
pub struct GameModInfo {
	pub mod_id: String,
	pub installed_version: Option<String>,
	pub installed_hash: Option<String>,
	pub is_outdated: bool,
	pub has_installed_dependants: bool,
	pub compatible: bool,
}

pub trait ModDatabase {
	fn setup_mod_tables(&self) -> Result;
	fn insert_mod(&self, game_mod: &GameMod, provider_id: ModProviderId);
	fn get_mod(&self, mod_id: &str) -> Result<GameMod>;
	fn get_installed_mod(
		&self,
		provider_id: &GameProviderId,
		game_id: &str,
		mod_id: &str,
	) -> Result<Option<InstalledMod>>;
	fn try_get_installed_mod(
		&self,
		provider_id: &GameProviderId,
		game_id: &str,
		mod_id: &str,
	) -> Result<InstalledMod>;
	fn get_mod_map(&self) -> Result<BTreeMap<String, GameMod>>;
	fn refresh_installed_mods(&self) -> Result;
	fn get_game_mods(
		&self,
		provider_id: &GameProviderId,
		game_id: &str,
	) -> Result<Vec<GameModInfo>>;
	fn remove_stale_mods(&self, max_time: u64, provider_id: ModProviderId) -> Result;
}

impl ModDatabase for DbMutex {
	fn setup_mod_tables(&self) -> Result {
		self.lock_db()?.execute_batch(
			r"
			CREATE TABLE IF NOT EXISTS mods (
				id TEXT NOT NULL,
				provider_id TEXT NOT NULL,
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
				hide_from_game_mods_list INTEGER,
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

	fn insert_mod(&self, game_mod: &GameMod, provider_id: ModProviderId) {
		if let Err(err) = try_insert_mod(self, game_mod, provider_id) {
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
				hash,
				hide_from_game_mods_list
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
					hide_from_game_mods_list: row.get(18)?,
				})
			})?)
	}

	fn get_installed_mod(
		&self,
		provider_id: &GameProviderId,
		game_id: &str,
		mod_id: &str,
	) -> Result<Option<InstalledMod>> {
		let game = self.get_game(provider_id, game_id)?;

		let Ok(exe_path) = game.try_get_exe_path() else {
			return Ok(None);
		};
		let exe_path_hash = exe_path.hash_string();

		let is_installed = self
			.lock_db()?
			.prepare_cached(
				"SELECT EXISTS(
					SELECT 1
					FROM main.installed_mods
					WHERE exe_path_hash = $1 AND mod_id = $2
				)",
			)?
			.query_row(rusqlite::params![exe_path_hash, mod_id], |row| {
				row.get::<_, bool>(0)
			})?;

		if !is_installed {
			return Ok(None);
		}

		let game_mod = self.get_mod(mod_id)?;
		Ok(Some(InstalledMod::new(game_mod, game)))
	}

	fn try_get_installed_mod(
		&self,
		provider_id: &GameProviderId,
		game_id: &str,
		mod_id: &str,
	) -> Result<InstalledMod> {
		self.get_installed_mod(provider_id, game_id, mod_id)?
			.ok_or(Error::ModNotInstalled(mod_id.to_string()))
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
				hash,
				hide_from_game_mods_list
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
					hide_from_game_mods_list: row.get(18)?,
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
		let installed_game_ids = self
			.lock_db()?
			.prepare_cached(
				"SELECT DISTINCT provider_id, game_id FROM main.installed_games WHERE exe_path IS NOT NULL",
			)?
			.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
			.filter_map(|game_id| match game_id {
				Ok(id) => Some(id),
				Err(err) => {
					log::warn!("Failed to read installed game from local database: {err}");
					None
				}
			})
			.collect::<Vec<(crate::game_providers::game_provider::GameProviderId, String)>>();

		let game_mods = self.get_mod_map()?;
		let mut installed_mod_rows = Vec::new();

		for (provider_id, game_id) in installed_game_ids {
			let game = self.get_game(&provider_id, &game_id)?;
			let Ok(exe_path) = game.try_get_exe_path() else {
				continue;
			};
			let exe_path_hash = exe_path.hash_string();

			for game_mod in game_mods.values() {
				let Ok(manifest_path) = game_mod.get_manifest_target_path(&game) else {
					continue;
				};

				if !manifest_path.exists() {
					continue;
				}

				let Some(manifest) = GameMod::from_file(&manifest_path) else {
					continue;
				};

				installed_mod_rows.push((
					exe_path_hash.clone(),
					manifest.id,
					manifest
						.download
						.as_ref()
						.map(|download| download.id.clone()),
					manifest.hash,
				));
			}
		}

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

				for (exe_path_hash, mod_id, installed_version, installed_hash) in installed_mod_rows
				{
					statement.execute(rusqlite::params![
						exe_path_hash,
						mod_id,
						installed_version,
						installed_hash,
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
					m.dependencies AS dependencies,
					CASE
						WHEN im.mod_id IS NULL THEN 0
						ELSE 1
					END AS is_installed,
					CASE
						WHEN im.mod_id IS NULL THEN 0
						WHEN json_extract(m.download, '$.id') IS NOT im.installed_version THEN 1
						WHEN m.hash IS NOT im.installed_hash THEN 1
						ELSE 0
					END AS is_outdated,
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
				cm.mod_id,
				cm.installed_version,
				cm.installed_hash,
				cm.is_outdated,
				CASE
					WHEN EXISTS (
						SELECT 1
						FROM candidate_mods dependant
						INNER JOIN json_each(dependant.dependencies) dep
						WHERE dependant.mod_id <> cm.mod_id
							AND dependant.is_installed = 1
							AND json_extract(dep.value, '$.modId') = cm.mod_id
					) THEN 1
					ELSE 0
				END AS has_installed_dependants,
				CASE
					WHEN cm.game_major IS NULL THEN 1
					WHEN cm.min_major IS NOT NULL AND (
						cm.min_major > cm.game_major
						OR (
							cm.min_major = cm.game_major
							AND cm.min_minor IS NOT NULL
							AND cm.game_minor IS NOT NULL
							AND (
								cm.min_minor > cm.game_minor
								OR (
									cm.min_minor = cm.game_minor
									AND cm.min_patch IS NOT NULL
									AND cm.game_patch IS NOT NULL
									AND cm.min_patch > cm.game_patch
								)
							)
						)
					) THEN 0
					WHEN cm.max_major IS NOT NULL AND (
						cm.max_major < cm.game_major
						OR (
							cm.max_major = cm.game_major
							AND cm.max_minor IS NOT NULL
							AND cm.game_minor IS NOT NULL
							AND (
								cm.max_minor < cm.game_minor
								OR (
									cm.max_minor = cm.game_minor
									AND cm.max_patch IS NOT NULL
									AND cm.game_patch IS NOT NULL
									AND cm.max_patch < cm.game_patch
								)
							)
						)
					) THEN 0
					ELSE 1
				END AS compatible
			FROM candidate_mods cm
		",
			)?
			.query_map([provider_id.to_string(), game_id.to_string()], |row| {
				Ok(GameModInfo {
					mod_id: row.get(0)?,
					installed_version: row.get(1)?,
					installed_hash: row.get(2)?,
					is_outdated: row.get(3)?,
					has_installed_dependants: row.get(4)?,
					compatible: row.get(5)?,
				})
			})?
			.collect::<rusqlite::Result<Vec<GameModInfo>>>()?)
	}

	fn remove_stale_mods(&self, max_time: u64, provider_id: ModProviderId) -> Result {
		self.lock_db()?
			.prepare_cached("DELETE FROM main.mods WHERE provider_id = $1 AND created_at < $2;")?
			.execute(rusqlite::params![provider_id, max_time.cast_signed()])?;

		Ok(())
	}
}

fn serialize_json_option<T: Serialize>(value: Option<&T>) -> Result<String> {
	serde_json::to_string(&value).map_err(Into::into)
}

fn try_insert_mod(
	connection_mutex: &DbMutex,
	game_mod: &GameMod,
	provider_id: ModProviderId,
) -> Result {
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
				provider_id,
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
				hide_from_game_mods_list,
				hash,
				created_at
			) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21)",
		)?
		.execute(rusqlite::params![
			provider_id,
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
			game_mod.hide_from_game_mods_list,
			game_mod.hash,
			SystemTime::now()
				.duration_since(UNIX_EPOCH)?
				.as_secs()
				.cast_signed()
		])?;

	Ok(())
}
