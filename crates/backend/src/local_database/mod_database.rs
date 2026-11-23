use std::{
	borrow::Cow,
	collections::BTreeMap,
	time::{SystemTime, UNIX_EPOCH},
};

use rusqlite::OptionalExtension;
use serde::Serialize;

use crate::{
	game::Game,
	game_providers::game_provider::GameProviderId,
	local_database::{
		app_database::{AppDatabase, Db},
		game_database::{GameDatabase, REMOTE_GAME_JOIN},
		rusqlite_extensions::RowExt,
	},
	mod_providers::mod_provider::ModProviderId,
	mods::{
		installed_mod::InstalledMod,
		manifest::{InstalledModManifest, Mod, ModDependency},
		mod_tag::ModTag,
	},
	operating_system::OperatingSystem,
	path_extensions::PathExt,
	result::{ContextResult, Error, LogErrExt, Result},
};

#[derive(serde::Serialize, Clone, Debug)]
pub struct ModInfo {
	pub mod_id: String,
	pub installed_version: Option<String>,
	pub installed_hash: Option<String>,
	pub is_outdated: bool,
	pub has_installed_dependants: bool,
	pub compatible: bool,
	pub tags: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct ModGame {
	pub provider_id: GameProviderId,
	pub game_id: String,
	pub title: String,
}

#[derive(Clone, Debug, Default)]
pub struct GameEnvironment {
	pub variables: BTreeMap<String, String>,
	pub mod_ids: Vec<String>,
}

pub trait ModDatabase {
	fn insert_mods(&self, mods: &[Mod], provider_id: ModProviderId, source_hash: &str);
	fn get_mod(&self, mod_id: &str) -> Result<Mod>;
	fn get_installed_mod(
		&self,
		mod_id: &str,
		provider_id_option: Option<GameProviderId>,
		game_id_option: Option<String>,
	) -> Result<Option<InstalledMod>>;
	fn get_installed_mods_for_game(&self, game: &Game) -> Result<Vec<InstalledMod>>;
	fn get_game_environment(&self, game: &Game) -> Result<GameEnvironment>;
	fn try_get_installed_mod(
		&self,
		provider_id: &GameProviderId,
		game_id: &str,
		mod_id: &str,
	) -> Result<InstalledMod>;
	fn get_mod_map(&self) -> Result<BTreeMap<String, Mod>>;
	fn refresh_installed_mods(&self) -> Result;
	fn get_game_mods(&self, provider_id: &GameProviderId, game_id: &str) -> Result<Vec<ModInfo>>;
	fn get_mod_games(&self) -> Result<BTreeMap<String, Vec<ModGame>>>;
	fn get_mod_ids_in_scope(&self, provider_id: ModProviderId, scope: &str) -> Result<Vec<String>>;
	fn remove_mods_except(&self, provider_id: ModProviderId, keep_ids: &[String]) -> Result;
	fn insert_mod_tags(
		&self,
		tags: &BTreeMap<String, ModTag>,
		provider_id: ModProviderId,
		source_hash: &str,
	);
	fn get_mod_tags(&self) -> Result<BTreeMap<String, ModTag>>;
	fn get_tag_ids_in_scope(&self, provider_id: ModProviderId, scope: &str) -> Result<Vec<String>>;
	fn remove_mod_tags_except(&self, provider_id: ModProviderId, keep_ids: &[String]) -> Result;
}

pub(crate) fn compute_scope(provider_id: ModProviderId, source_hash: &str) -> String {
	if source_hash.is_empty() {
		String::new()
	} else {
		format!("{provider_id}:{source_hash}")
	}
}

pub(crate) fn scope_id<'a>(scope: &'a str, original_id: &'a str) -> Cow<'a, str> {
	if scope.is_empty() {
		Cow::Borrowed(original_id)
	} else {
		Cow::Owned(format!("{scope}:{original_id}"))
	}
}

pub(crate) const GAME_IDS_COMPATIBILITY_FILTER: &str = r"(
	m.has_game_ids = 0
	OR EXISTS (
		SELECT 1
		FROM main.mod_game_ids mod_game_id
		INNER JOIN main.game_external_ids game_external_id
			ON mod_game_id.external_id_provider = CASE game_external_id.external_id_provider
				WHEN 'Manual' THEN 'NormalizedTitle'
				ELSE game_external_id.external_id_provider
			END
			AND game_external_id.external_id = mod_game_id.external_id
		WHERE mod_game_id.mod_id = m.id
			AND game_external_id.provider_id = g.provider_id
			AND game_external_id.game_id = g.game_id
	)
)";

fn normalize_external_id_provider(provider: &str) -> &str {
	if provider == "Manual" {
		"NormalizedTitle"
	} else {
		provider
	}
}

pub(crate) fn create_tables(connection: &rusqlite::Connection) -> Result {
	connection.execute_batch(
		r"
			CREATE TABLE IF NOT EXISTS mods (
				id TEXT NOT NULL PRIMARY KEY,
				scope TEXT NOT NULL DEFAULT '',
				provider_id TEXT NOT NULL,
				title TEXT NOT NULL,
				author TEXT NOT NULL,
				website TEXT,
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
				optional_dependencies TEXT,
				required_dependencies TEXT,
				install TEXT,
				run_for_game TEXT,
				run_standalone TEXT,
				game_environment TEXT,
				hash TEXT,
				tags TEXT,
				game_ids TEXT,
				created_at INTEGER,
				has_game_ids INTEGER GENERATED ALWAYS AS (json_extract(game_ids, '$') IS NOT NULL) VIRTUAL,
				engine_key TEXT GENERATED ALWAYS AS (json_extract(engine, '$')) VIRTUAL,
				unity_backend_key TEXT GENERATED ALWAYS AS (json_extract(unity_backend, '$')) VIRTUAL,
				architecture_key TEXT GENERATED ALWAYS AS (json_extract(architecture, '$')) VIRTUAL,
				game_os_key TEXT GENERATED ALWAYS AS (json_extract(game_os, '$')) VIRTUAL,
				host_os_key TEXT GENERATED ALWAYS AS (json_extract(host_os, '$')) VIRTUAL,
				engine_min_major INTEGER GENERATED ALWAYS AS (json_extract(engine_version_range, '$.minimum.major')) VIRTUAL,
				engine_min_minor INTEGER GENERATED ALWAYS AS (json_extract(engine_version_range, '$.minimum.minor')) VIRTUAL,
				engine_min_patch INTEGER GENERATED ALWAYS AS (json_extract(engine_version_range, '$.minimum.patch')) VIRTUAL,
				engine_max_major INTEGER GENERATED ALWAYS AS (json_extract(engine_version_range, '$.maximum.major')) VIRTUAL,
				engine_max_minor INTEGER GENERATED ALWAYS AS (json_extract(engine_version_range, '$.maximum.minor')) VIRTUAL,
				engine_max_patch INTEGER GENERATED ALWAYS AS (json_extract(engine_version_range, '$.maximum.patch')) VIRTUAL
			);

			CREATE INDEX IF NOT EXISTS idx_mods_created_at ON mods(created_at);

			CREATE INDEX IF NOT EXISTS idx_mods_has_game_ids ON mods(has_game_ids);

			CREATE TABLE IF NOT EXISTS mod_game_ids (
				mod_id TEXT NOT NULL,
				external_id_provider TEXT NOT NULL,
				external_id TEXT NOT NULL,
				PRIMARY KEY (mod_id, external_id_provider, external_id)
			);

			CREATE INDEX IF NOT EXISTS idx_mod_game_ids_lookup ON mod_game_ids(external_id_provider, external_id);

			CREATE TABLE IF NOT EXISTS mod_tags (
				id TEXT NOT NULL PRIMARY KEY,
				scope TEXT NOT NULL DEFAULT '',
				provider_id TEXT NOT NULL,
				tag_id TEXT NOT NULL,
				data TEXT NOT NULL
			);

			CREATE TABLE IF NOT EXISTS mod_tag_links (
				mod_id TEXT NOT NULL,
				tag TEXT NOT NULL,
				PRIMARY KEY (mod_id, tag)
			) WITHOUT ROWID;

			CREATE TABLE IF NOT EXISTS installed_mods (
				exe_path_hash TEXT NOT NULL,
				mod_id TEXT NOT NULL,
				installed_version TEXT,
				installed_hash TEXT,
				created_at INTEGER,
				PRIMARY KEY (exe_path_hash, mod_id)
			);

			CREATE INDEX IF NOT EXISTS idx_installed_mods_hash ON installed_mods(exe_path_hash);
		",
	)?;

	Ok(())
}

impl ModDatabase for Db {
	fn insert_mods(&self, mods: &[Mod], provider_id: ModProviderId, source_hash: &str) {
		if let Err(err) = try_insert_mods(self, mods, provider_id, source_hash) {
			log::error!(
				"Failed to insert {} mods into local database: {}",
				mods.len(),
				err
			);
		}
	}

	fn get_mod(&self, mod_id: &str) -> Result<Mod> {
		self.read_db()?
			.prepare_cached(
				r"
			SELECT
				id,
				title,
				author,
				website,
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
				optional_dependencies,
				required_dependencies,
				install,
				run_for_game,
				run_standalone,
				hash,
				hide_from_game_mods_list,
				tags,
				scope,
				game_environment,
				game_ids
			FROM main.mods
			WHERE id = $1
			LIMIT 1
		",
			)?
			.query_row([mod_id], |row| {
				Ok(Mod {
					id: row.get(0)?,
					title: row.get(1)?,
					author: row.get(2)?,
					website: row.get(3)?,
					description: row.get(4)?,
					download: row.get_json(5)?,
					engine: row.get_json(6)?,
					engine_version_range: row.get_json(7)?,
					unity_backend: row.get_json(8)?,
					architecture: row.get_json(9)?,
					game_os: row.get_json(10)?,
					host_os: row.get_json(11)?,
					deprecated: row.get(12)?,
					paths: row.get_json(13)?,
					optional_dependencies: row.get_json(14)?,
					required_dependencies: row.get_json(15)?,
					install: row.get_json(16)?,
					run_for_game: row.get_json(17)?,
					run_standalone: row.get_json(18)?,
					hash: row.get(19)?,
					hide_from_game_mods_list: row.get(20)?,
					tags: row.get_json(21)?,
					scope: row.get(22)?,
					game_environment: row.get_json(23)?,
					game_ids: row.get_json(24)?,
				})
			})
			.optional()?
			.ok_or_else(|| Error::ModNotFound(mod_id.to_string()))
	}

	fn get_installed_mod(
		&self,
		mod_id: &str,
		provider_id_option: Option<GameProviderId>,
		game_id_option: Option<String>,
	) -> Result<Option<InstalledMod>> {
		let is_installed = self
			.read_db()?
			.prepare_cached(
				"SELECT EXISTS(
					SELECT 1
					FROM main.installed_mods im
					WHERE im.mod_id = $1
						AND (
							$2 IS NULL OR $3 IS NULL OR im.exe_path_hash = (
								SELECT g.exe_path_hash
								FROM main.games g
								WHERE g.provider_id = $2 AND g.game_id = $3
								LIMIT 1
							)
						)
				)",
			)?
			.query_row(
				rusqlite::params![
					mod_id,
					provider_id_option.map(|provider_id| { provider_id.to_string() }),
					game_id_option.as_deref(),
				],
				|row| row.get::<_, bool>(0),
			)
			.with_context(|| format!("checking whether mod `{mod_id}` is installed"))?;

		if !is_installed {
			return Ok(None);
		}

		let game_mod = match self.get_mod(mod_id) {
			Ok(game_mod) => game_mod,
			Err(Error::ModNotFound(_)) => return Ok(None),
			Err(error) => return Err(error),
		};
		let game_option = if let Some(game_id) = game_id_option
			&& let Some(provider_id) = provider_id_option
		{
			Some(self.get_game(&provider_id, &game_id)?)
		} else {
			None
		};

		Ok(Some(InstalledMod::new(game_mod, game_option)))
	}

	fn try_get_installed_mod(
		&self,
		provider_id: &GameProviderId,
		game_id: &str,
		mod_id: &str,
	) -> Result<InstalledMod> {
		self.get_installed_mod(mod_id, Some(*provider_id), Some(game_id.to_string()))?
			.ok_or(Error::ModNotInstalled(mod_id.to_string()))
	}

	fn get_installed_mods_for_game(&self, game: &Game) -> Result<Vec<InstalledMod>> {
		let exe_path_hash = game.try_get_exe_path()?.hash_string();

		let mod_ids = self
			.read_db()?
			.prepare_cached("SELECT mod_id FROM main.installed_mods WHERE exe_path_hash = $1")?
			.query_map([exe_path_hash], |row| row.get::<_, String>(0))?
			.collect::<rusqlite::Result<Vec<String>>>()?;

		let mut installed_mods = Vec::new();

		for mod_id in mod_ids {
			if let Some(game_mod) = self
				.get_mod(&mod_id)
				.ok_or_log("Failed to load installed mod")
			{
				installed_mods.push(InstalledMod::new(game_mod, Some(game.clone())));
			}
		}

		Ok(installed_mods)
	}

	fn get_game_environment(&self, game: &Game) -> Result<GameEnvironment> {
		let exe_path_hash = game.try_get_exe_path()?.hash_string();

		let mod_ids = self
			.read_db()?
			.prepare_cached("SELECT mod_id FROM main.installed_mods WHERE exe_path_hash = $1")?
			.query_map([exe_path_hash], |row| row.get::<_, String>(0))?
			.collect::<rusqlite::Result<Vec<String>>>()?;

		let mut environment = BTreeMap::new();
		let mut environment_mod_ids = Vec::new();

		for mod_id in mod_ids {
			let Some(game_mod) = self
				.get_mod(&mod_id)
				.ok_or_log("Failed to load mod environment")
			else {
				continue;
			};
			let mod_environment = game_mod.get_game_environment(Some(game));
			if !mod_environment.is_empty() {
				environment_mod_ids.push(mod_id);
			}
			environment.extend(mod_environment);
		}

		Ok(GameEnvironment {
			variables: environment,
			mod_ids: environment_mod_ids,
		})
	}

	fn get_mod_map(&self) -> Result<BTreeMap<String, Mod>> {
		Ok(self
			.read_db()?
			.prepare_cached(
				r"
			SELECT
				id,
				title,
				author,
				website,
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
				optional_dependencies,
				required_dependencies,
				install,
				run_for_game,
				run_standalone,
				hash,
				hide_from_game_mods_list,
				tags,
				scope,
				game_environment,
				game_ids
			FROM main.mods
		",
			)?
			.query_map([], |row| {
				Ok(Mod {
					id: row.get(0)?,
					title: row.get(1)?,
					author: row.get(2)?,
					website: row.get(3)?,
					description: row.get(4)?,
					download: row.get_json(5)?,
					engine: row.get_json(6)?,
					engine_version_range: row.get_json(7)?,
					unity_backend: row.get_json(8)?,
					architecture: row.get_json(9)?,
					game_os: row.get_json(10)?,
					host_os: row.get_json(11)?,
					deprecated: row.get(12)?,
					paths: row.get_json(13)?,
					optional_dependencies: row.get_json(14)?,
					required_dependencies: row.get_json(15)?,
					install: row.get_json(16)?,
					run_for_game: row.get_json(17)?,
					run_standalone: row.get_json(18)?,
					hash: row.get(19)?,
					hide_from_game_mods_list: row.get(20)?,
					tags: row.get_json(21)?,
					scope: row.get(22)?,
					game_environment: row.get_json(23)?,
					game_ids: row.get_json(24)?,
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
			.read_db()?
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
			let Some(game) = self
				.get_game(&provider_id, &game_id)
				.ok_or_log("Failed to load installed game while refreshing mods")
			else {
				continue;
			};
			let Some(exe_path) = game
				.try_get_exe_path()
				.ok_or_log("Failed to get game executable path while refreshing installed mods")
			else {
				continue;
			};
			let exe_path_hash = exe_path.hash_string();

			for game_mod in game_mods.values() {
				if game_mod.install.is_none() {
					continue;
				}

				let Some(manifest_path) = game_mod
					.get_manifest_target_path(Some(&game))
					.ok_or_log("Failed to get installed mod manifest path")
				else {
					continue;
				};

				if !manifest_path.exists() {
					continue;
				}

				let Some(manifest) = InstalledModManifest::from_file(&manifest_path) else {
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
			let mut connection = self.write_db()?;
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

	fn get_game_mods(&self, provider_id: &GameProviderId, game_id: &str) -> Result<Vec<ModInfo>> {
		let mut mod_infos = self
			.read_db()?
			.prepare_cached(&format!(
				r"
			WITH
			game_info AS MATERIALIZED (
				SELECT
					g.provider_id AS provider_id,
					g.game_id AS game_id,
					g.exe_path_hash AS exe_path_hash,
					COALESCE(ig.engine_brand, rg.engine_brand) AS engine_brand,
					ig.unity_backend AS unity_backend,
					ig.architecture AS architecture,
					ig.executable_os AS executable_os,
					COALESCE(ig.engine_version_major, rg.engine_version_major) AS game_major,
					COALESCE(ig.engine_version_minor, rg.engine_version_minor) AS game_minor,
					COALESCE(ig.engine_version_patch, rg.engine_version_patch) AS game_patch
				FROM main.games g
				LEFT JOIN main.installed_games ig ON g.provider_id = ig.provider_id AND g.game_id = ig.game_id
				{REMOTE_GAME_JOIN}
				WHERE g.provider_id = $1 AND g.game_id = $2
			),
			candidate_mods AS (
				SELECT DISTINCT
					m.id AS mod_id,
					im.installed_version AS installed_version,
					im.installed_hash AS installed_hash,
					m.optional_dependencies AS optional_dependencies,
					m.required_dependencies AS required_dependencies,
					m.tags AS tags,
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
					g.game_major AS game_major,
					g.game_minor AS game_minor,
					g.game_patch AS game_patch,
					m.engine_min_major AS min_major,
					m.engine_min_minor AS min_minor,
					m.engine_min_patch AS min_patch,
					m.engine_max_major AS max_major,
					m.engine_max_minor AS max_minor,
					m.engine_max_patch AS max_patch
				FROM game_info g
				INNER JOIN main.mods m ON 1=1
				LEFT JOIN main.installed_mods im ON
					g.exe_path_hash = im.exe_path_hash
					AND m.id = im.mod_id
				WHERE {GAME_IDS_COMPATIBILITY_FILTER}
					AND (
						m.deprecated IS NULL
						OR m.deprecated = 0
						OR im.mod_id IS NOT NULL
					)
					AND (
						m.engine_key IS NULL
						OR m.engine_key = g.engine_brand
					)
					AND (
						m.unity_backend_key IS NULL
						OR g.unity_backend IS NULL
						OR m.unity_backend_key = g.unity_backend
					)
					AND (
						m.architecture_key IS NULL
						OR g.architecture IS NULL
						OR m.architecture_key = g.architecture
					)
					AND (
						m.game_os_key IS NULL
						OR g.executable_os IS NULL
						OR m.game_os_key = g.executable_os
					)
					AND (
						m.host_os_key IS NULL
						OR m.host_os_key = $3
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
						INNER JOIN json_each(dependant.optional_dependencies) dep ON 1=1
						WHERE dependant.mod_id <> cm.mod_id
							AND dependant.is_installed = 1
							AND (
								json_extract(dep.value, '$.modId') = cm.mod_id
								OR EXISTS (SELECT 1 FROM main.mod_tag_links mod_tag_link WHERE mod_tag_link.mod_id = cm.mod_id AND mod_tag_link.tag = json_extract(dep.value, '$.tag'))
							)
					) THEN 1
					WHEN EXISTS (
						SELECT 1
						FROM candidate_mods dependant
						INNER JOIN json_each(dependant.required_dependencies) dep ON 1=1
						WHERE dependant.mod_id <> cm.mod_id
							AND dependant.is_installed = 1
							AND (
								json_extract(dep.value, '$.modId') = cm.mod_id
								OR EXISTS (SELECT 1 FROM main.mod_tag_links mod_tag_link WHERE mod_tag_link.mod_id = cm.mod_id AND mod_tag_link.tag = json_extract(dep.value, '$.tag'))
							)
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
				END AS compatible,
				cm.tags
			FROM candidate_mods cm
		",
			))?
			.query_map(
				[
					provider_id.to_string(),
					game_id.to_string(),
					OperatingSystem::get_current().to_string(),
				],
				|row| {
					Ok(ModInfo {
						mod_id: row.get(0)?,
						installed_version: row.get(1)?,
						installed_hash: row.get(2)?,
						is_outdated: row.get(3)?,
						has_installed_dependants: row.get(4)?,
						compatible: row.get(5)?,
						tags: row.get_json(6)?,
					})
				},
			)?
			.collect::<rusqlite::Result<Vec<ModInfo>>>()?;

		// A mod whose required dependencies don't exist (or aren't compatible)
		// is itself considered not compatible.
		let mod_map = self.get_mod_map()?;
		apply_required_dependency_compatibility(&mut mod_infos, &mod_map);

		Ok(mod_infos)
	}

	fn get_mod_games(&self) -> Result<BTreeMap<String, Vec<ModGame>>> {
		let rows = self
			.read_db()?
			.prepare_cached(
				r"
			SELECT
				mod_game_id.mod_id,
				g.provider_id,
				g.game_id,
				g.display_title,
				g.title_discriminator
			FROM main.mod_game_ids mod_game_id
			INNER JOIN main.game_external_ids game_external_id
				ON mod_game_id.external_id_provider = CASE game_external_id.external_id_provider
					WHEN 'Manual' THEN 'NormalizedTitle'
					ELSE game_external_id.external_id_provider
				END
				AND game_external_id.external_id = mod_game_id.external_id
			INNER JOIN main.games g
				ON g.provider_id = game_external_id.provider_id
				AND g.game_id = game_external_id.game_id
			ORDER BY mod_game_id.mod_id, g.display_title;
		",
			)?
			.query_map([], |row| {
				let display_title = row.get::<_, String>(3)?;
				let discriminator = row.get::<_, Option<String>>(4)?;
				let title = if let Some(discriminator) = discriminator {
					format!("{display_title} ({discriminator})")
				} else {
					display_title
				};

				Ok((
					row.get::<_, String>(0)?,
					ModGame {
						provider_id: row.get(1)?,
						game_id: row.get(2)?,
						title,
					},
				))
			})?
			.collect::<rusqlite::Result<Vec<_>>>()?;

		let mut games: BTreeMap<String, Vec<ModGame>> = BTreeMap::new();

		for (mod_id, game) in rows {
			let entries = games.entry(mod_id).or_default();

			if !entries
				.iter()
				.any(|entry| entry.provider_id == game.provider_id && entry.game_id == game.game_id)
			{
				entries.push(game);
			}
		}

		Ok(games)
	}

	fn get_mod_ids_in_scope(&self, provider_id: ModProviderId, scope: &str) -> Result<Vec<String>> {
		let ids = self
			.read_db()?
			.prepare_cached("SELECT id FROM main.mods WHERE provider_id = $1 AND scope = $2;")?
			.query_map(rusqlite::params![provider_id, scope], |row| {
				row.get::<_, String>(0)
			})?
			.collect::<rusqlite::Result<Vec<String>>>()?;

		Ok(ids)
	}

	fn remove_mods_except(&self, provider_id: ModProviderId, keep_ids: &[String]) -> Result {
		let keep_json = serde_json::to_string(keep_ids)?;

		let mut connection = self.write_db()?;
		let transaction = connection.transaction()?;

		transaction.prepare_cached(
			"DELETE FROM main.mod_game_ids WHERE mod_id IN (SELECT id FROM main.mods WHERE provider_id = ? AND id NOT IN (SELECT value FROM json_each(?)) AND id NOT IN (SELECT mod_id FROM main.installed_mods));",
		)?
		.execute(rusqlite::params![provider_id, keep_json])?;
		transaction.prepare_cached(
			"DELETE FROM main.mod_tag_links WHERE mod_id IN (SELECT id FROM main.mods WHERE provider_id = ? AND id NOT IN (SELECT value FROM json_each(?)) AND id NOT IN (SELECT mod_id FROM main.installed_mods));",
		)?
		.execute(rusqlite::params![provider_id, keep_json])?;
		transaction
			.prepare_cached(
				"DELETE FROM main.mods WHERE provider_id = ? AND id NOT IN (SELECT value FROM json_each(?)) AND id NOT IN (SELECT mod_id FROM main.installed_mods);",
			)?
			.execute(rusqlite::params![provider_id, keep_json])?;

		transaction.commit()?;

		Ok(())
	}

	fn insert_mod_tags(
		&self,
		tags: &BTreeMap<String, ModTag>,
		provider_id: ModProviderId,
		source_hash: &str,
	) {
		if let Err(err) = try_insert_mod_tags(self, tags, provider_id, source_hash) {
			log::error!("Failed to insert mod tags into local database: {err}");
		}
	}

	fn get_mod_tags(&self) -> Result<BTreeMap<String, ModTag>> {
		let rows = self
			.read_db()?
			.prepare_cached("SELECT tag_id, data FROM main.mod_tags ORDER BY id;")?
			.query_map([], |row| {
				Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
			})?
			.collect::<rusqlite::Result<Vec<(String, String)>>>()?;

		let mut tags = BTreeMap::new();

		for (tag_id, data) in rows {
			match serde_json::from_str::<ModTag>(&data) {
				Ok(tag) => {
					tags.insert(tag_id, tag);
				}
				Err(err) => {
					log::warn!("Failed to parse mod tag `{tag_id}` from local database: {err}");
				}
			}
		}

		Ok(tags)
	}

	fn get_tag_ids_in_scope(&self, provider_id: ModProviderId, scope: &str) -> Result<Vec<String>> {
		let ids = self
			.read_db()?
			.prepare_cached("SELECT id FROM main.mod_tags WHERE provider_id = $1 AND scope = $2;")?
			.query_map(rusqlite::params![provider_id, scope], |row| {
				row.get::<_, String>(0)
			})?
			.collect::<rusqlite::Result<Vec<String>>>()?;

		Ok(ids)
	}

	fn remove_mod_tags_except(&self, provider_id: ModProviderId, keep_ids: &[String]) -> Result {
		let keep_json = serde_json::to_string(keep_ids)?;

		self.write_db()?
			.prepare_cached(
				"DELETE FROM main.mod_tags WHERE provider_id = ? AND id NOT IN (SELECT value FROM json_each(?));",
			)?
			.execute(rusqlite::params![provider_id, keep_json])?;

		Ok(())
	}
}

fn serialize_json_option<T: Serialize>(value: Option<&T>) -> Result<String> {
	serde_json::to_string(&value).map_err(Into::into)
}

fn scope_dependencies(deps: &[ModDependency], scope: &str) -> Vec<ModDependency> {
	deps.iter()
		.map(|dep| match dep {
			ModDependency::ModId { mod_id } => ModDependency::ModId {
				mod_id: scope_id(scope, mod_id).into_owned(),
			},
			ModDependency::Tag { .. } => dep.clone(),
		})
		.collect()
}

// A mod is considered not compatible if any of its required dependencies
// don't exist in this game's mod list, or aren't themselves compatible.
// Kept in a loop so incompatibility propagates through chains of required deps.
fn apply_required_dependency_compatibility(
	mod_infos: &mut [ModInfo],
	mod_map: &BTreeMap<String, Mod>,
) {
	loop {
		let mut changed = false;

		for i in 0..mod_infos.len() {
			if !mod_infos[i].compatible {
				continue;
			}

			let required_deps_ok = mod_map
				.get(&mod_infos[i].mod_id)
				.and_then(|game_mod| game_mod.required_dependencies.as_ref())
				.is_none_or(|required_deps| {
					required_deps.iter().all(|dep| {
						mod_infos
							.iter()
							.any(|info| info.compatible && dependency_matches(dep, info))
					})
				});

			if !required_deps_ok {
				mod_infos[i].compatible = false;
				changed = true;
			}
		}

		if !changed {
			break;
		}
	}
}

fn dependency_matches(dep: &ModDependency, info: &ModInfo) -> bool {
	match dep {
		ModDependency::ModId { mod_id } => info.mod_id == *mod_id,
		ModDependency::Tag { tag } => info.tags.contains(tag),
	}
}

fn try_insert_mods(
	connection_mutex: &Db,
	mods: &[Mod],
	provider_id: ModProviderId,
	source_hash: &str,
) -> Result {
	let scope = compute_scope(provider_id, source_hash);
	let created_at = SystemTime::now()
		.duration_since(UNIX_EPOCH)?
		.as_secs()
		.cast_signed();

	let mut connection = connection_mutex.write_db()?;
	let transaction = connection.transaction()?;

	{
		let mut insert_mod = transaction.prepare_cached(
			"INSERT OR REPLACE INTO mods (
				id,
				provider_id,
				title,
				author,
				website,
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
				optional_dependencies,
				required_dependencies,
				install,
				run_for_game,
				run_standalone,
				game_environment,
				hide_from_game_mods_list,
				hash,
				tags,
				created_at,
				scope,
				game_ids
			) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24, $25, $26, $27)",
		)?;
		let mut delete_game_ids =
			transaction.prepare_cached("DELETE FROM main.mod_game_ids WHERE mod_id = $1;")?;
		let mut insert_game_id = transaction.prepare_cached(
			"INSERT OR IGNORE INTO main.mod_game_ids (mod_id, external_id_provider, external_id) VALUES ($1, $2, $3);",
		)?;
		let mut delete_tag_links =
			transaction.prepare_cached("DELETE FROM main.mod_tag_links WHERE mod_id = $1;")?;
		let mut insert_tag_link = transaction.prepare_cached(
			"INSERT OR IGNORE INTO main.mod_tag_links (mod_id, tag) VALUES ($1, $2);",
		)?;

		for game_mod in mods {
			let scoped_id = scope_id(&scope, &game_mod.id);

			let scoped_optional_deps = game_mod
				.optional_dependencies
				.as_ref()
				.map(|deps| scope_dependencies(deps, &scope));
			let scoped_required_deps = game_mod
				.required_dependencies
				.as_ref()
				.map(|deps| scope_dependencies(deps, &scope));

			insert_mod.execute(rusqlite::params![
				scoped_id.as_ref(),
				provider_id,
				game_mod.title,
				game_mod.author,
				game_mod.website,
				game_mod.description,
				serialize_json_option(game_mod.download.as_ref())?,
				serialize_json_option(game_mod.engine.as_ref())?,
				serialize_json_option(game_mod.engine_version_range.as_ref())?,
				serialize_json_option(game_mod.unity_backend.as_ref())?,
				serialize_json_option(game_mod.architecture.as_ref())?,
				serialize_json_option(game_mod.game_os.as_ref())?,
				serialize_json_option(game_mod.host_os.as_ref())?,
				game_mod.deprecated,
				serialize_json_option(game_mod.paths.as_ref())?,
				serialize_json_option(scoped_optional_deps.as_ref())?,
				serialize_json_option(scoped_required_deps.as_ref())?,
				serialize_json_option(game_mod.install.as_ref())?,
				serialize_json_option(game_mod.run_for_game.as_ref())?,
				serialize_json_option(game_mod.run_standalone.as_ref())?,
				serialize_json_option(game_mod.game_environment.as_ref())?,
				game_mod.hide_from_game_mods_list,
				game_mod.hash,
				serde_json::to_string(&game_mod.tags.clone().unwrap_or_default())?,
				created_at,
				scope,
				serialize_json_option(game_mod.game_ids.as_ref())?,
			])?;

			delete_game_ids.execute([scoped_id.as_ref()])?;
			delete_tag_links.execute([scoped_id.as_ref()])?;

			if let Some(game_ids) = &game_mod.game_ids {
				for (provider, ids) in &game_ids.0 {
					let provider = normalize_external_id_provider(provider);

					for id in ids {
						insert_game_id.execute(rusqlite::params![
							scoped_id.as_ref(),
							provider,
							id
						])?;
					}
				}
			}

			if let Some(tags) = &game_mod.tags {
				for tag in tags {
					insert_tag_link.execute(rusqlite::params![scoped_id.as_ref(), tag])?;
				}
			}
		}
	}

	transaction.commit()?;
	drop(connection);

	Ok(())
}

fn try_insert_mod_tags(
	connection_mutex: &Db,
	tags: &BTreeMap<String, ModTag>,
	provider_id: ModProviderId,
	source_hash: &str,
) -> Result {
	let scope = compute_scope(provider_id, source_hash);

	let mut connection = connection_mutex.write_db()?;
	let transaction = connection.transaction()?;

	{
		let mut statement = transaction.prepare_cached(
			"INSERT OR REPLACE INTO mod_tags (
				id,
				scope,
				provider_id,
				tag_id,
				data
			) VALUES ($1, $2, $3, $4, $5)",
		)?;

		for (tag_id, tag) in tags {
			let scoped_id = scope_id(&scope, tag_id);

			statement.execute(rusqlite::params![
				scoped_id.as_ref(),
				scope,
				provider_id,
				tag_id,
				serde_json::to_string(tag)?,
			])?;
		}
	}

	transaction.commit()?;
	drop(connection);

	Ok(())
}
