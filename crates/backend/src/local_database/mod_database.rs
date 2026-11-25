use fs_err as fs;
use std::{
	collections::{BTreeMap, HashSet},
	path::Path,
};

use rusqlite::OptionalExtension;
use serde::Serialize;

use crate::{
	app_paths,
	game::Game,
	game_providers::game_provider::GameProviderId,
	local_database::{
		app_database::{AppDatabase, Db},
		engine_compatibility::{self, VersionColumns},
		game_database::{GameDatabase, REMOTE_GAME_JOIN},
		rusqlite_extensions::RowExt,
	},
	mods::{
		installed_mod::InstalledMod,
		manifest::{InstallKind, InstalledManifest, Mod, ModDependency},
		mod_tag::ModTag,
	},
	operating_system::OperatingSystem,
	path_extensions::PathExt,
	result::{ContextResult, Error, Result},
};

#[derive(serde::Serialize, Clone, Debug)]
pub struct ModInfo {
	pub mod_id: String,
	pub installed_version: Option<String>,
	pub is_installed: bool,
	pub is_in_catalog: bool,
	pub broken: bool,
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
	fn replace_catalog(&self, mods: &[Mod], tags: &BTreeMap<String, ModTag>) -> Result;
	fn get_mod(&self, mod_id: &str) -> Result<Mod>;
	fn get_installed_mod(
		&self,
		provider_id: &GameProviderId,
		game_id: &str,
		mod_id: &str,
	) -> Result<Option<InstalledMod>>;
	fn get_installed_mods_for_game(&self, game: &Game) -> Result<Vec<InstalledMod>>;
	fn get_game_environment(&self, game: &Game) -> Result<GameEnvironment>;
	fn try_get_installed_mod(
		&self,
		provider_id: &GameProviderId,
		game_id: &str,
		mod_id: &str,
	) -> Result<InstalledMod>;
	fn get_mod_or_installed(
		&self,
		provider_id: &GameProviderId,
		game_id: &str,
		mod_id: &str,
	) -> Result<Mod>;
	fn get_mod_map(&self) -> Result<BTreeMap<String, Mod>>;
	fn get_catalog_mod_map(&self) -> Result<BTreeMap<String, Mod>>;
	fn refresh_installed_mods(&self) -> Result;
	fn get_game_mods(&self, provider_id: &GameProviderId, game_id: &str) -> Result<Vec<ModInfo>>;
	fn get_mod_games(&self) -> Result<BTreeMap<String, Vec<ModGame>>>;
	fn get_broken_installed_manifests(&self, game: &Game) -> Result<Vec<String>>;
	fn get_mod_tags(&self) -> Result<BTreeMap<String, ModTag>>;
}

const MOD_COLUMNS: &str = r"
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
	paths,
	optional_dependencies,
	required_dependencies,
	install,
	run_for_game,
	run_standalone,
	hash,
	tags,
	game_environment,
	game_ids
";

fn read_mod(row: &rusqlite::Row<'_>) -> rusqlite::Result<Mod> {
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
		tags: row.get_json(20)?,
		game_environment: row.get_json(21)?,
		game_ids: row.get_json(22)?,
	})
}

pub(crate) const GAME_IDS_COMPATIBILITY_FILTER: &str = r"(
	m.has_game_ids = 0
	OR EXISTS (
		SELECT 1
		FROM main.mod_game_ids mod_game_id
		INNER JOIN main.game_external_ids game_external_id
			ON mod_game_id.external_id_provider = game_external_id.external_id_provider
			AND game_external_id.external_id = mod_game_id.external_id
		WHERE mod_game_id.mod_id = m.id
			AND game_external_id.provider_id = g.provider_id
			AND game_external_id.game_id = g.game_id
	)
)";

pub(crate) fn create_tables(connection: &rusqlite::Connection) -> Result {
	connection.execute_batch(
		r"
			CREATE TABLE IF NOT EXISTS mods (
				id TEXT NOT NULL PRIMARY KEY,
				in_catalog INTEGER NOT NULL DEFAULT 1,
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
				paths TEXT,
				optional_dependencies TEXT,
				required_dependencies TEXT,
				install TEXT,
				run_for_game TEXT,
				run_standalone TEXT,
				game_environment TEXT,
				hash TEXT,
				tags TEXT,
				game_ids TEXT,
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
				manifest TEXT,
				PRIMARY KEY (exe_path_hash, mod_id)
			);

			CREATE INDEX IF NOT EXISTS idx_installed_mods_hash ON installed_mods(exe_path_hash);
		",
	)?;

	Ok(())
}

impl ModDatabase for Db {
	fn replace_catalog(&self, mods: &[Mod], tags: &BTreeMap<String, ModTag>) -> Result {
		try_replace_catalog(self, mods, tags)
	}

	fn get_mod(&self, mod_id: &str) -> Result<Mod> {
		self.read_db()?
			.prepare_cached(&format!(
				"SELECT {MOD_COLUMNS} FROM main.mods WHERE id = $1 LIMIT 1"
			))?
			.query_row([mod_id], read_mod)
			.optional()?
			.ok_or_else(|| Error::ModNotFound(mod_id.to_string()))
	}

	fn get_installed_mod(
		&self,
		provider_id: &GameProviderId,
		game_id: &str,
		mod_id: &str,
	) -> Result<Option<InstalledMod>> {
		let manifest = self
			.read_db()?
			.prepare_cached(
				"SELECT manifest
				FROM main.installed_mods im
				WHERE im.mod_id = $1
					AND im.exe_path_hash = (
						SELECT g.exe_path_hash
						FROM main.games g
						WHERE g.provider_id = $2 AND g.game_id = $3
						LIMIT 1
					)
				LIMIT 1",
			)?
			.query_row(
				rusqlite::params![mod_id, provider_id.to_string(), game_id],
				|row| row.get::<_, Option<String>>(0),
			)
			.optional()
			.with_context(|| format!("checking whether mod `{mod_id}` is installed"))?;

		let Some(manifest) = manifest else {
			return Ok(None);
		};

		let Some(game_mod) = read_installed_mod(self, mod_id, manifest)? else {
			return Ok(None);
		};

		Ok(Some(InstalledMod {
			game_mod,
			game: self.get_game(provider_id, game_id)?,
		}))
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

	fn get_mod_or_installed(
		&self,
		provider_id: &GameProviderId,
		game_id: &str,
		mod_id: &str,
	) -> Result<Mod> {
		match self.get_mod(mod_id) {
			Ok(game_mod) => Ok(game_mod),
			Err(Error::ModNotFound(_)) => Ok(self
				.try_get_installed_mod(provider_id, game_id, mod_id)?
				.game_mod),
			Err(error) => Err(error),
		}
	}

	fn get_installed_mods_for_game(&self, game: &Game) -> Result<Vec<InstalledMod>> {
		let exe_path_hash = game.try_get_exe_path()?.hash_string();

		let rows = self
			.read_db()?
			.prepare_cached(
				"SELECT mod_id, manifest
				FROM main.installed_mods
				WHERE exe_path_hash = $1",
			)?
			.query_map([exe_path_hash], |row| {
				Ok((row.get::<_, String>(0)?, row.get::<_, Option<String>>(1)?))
			})?
			.collect::<rusqlite::Result<Vec<_>>>()?;

		let mut installed_mods = Vec::new();

		for (mod_id, manifest) in rows {
			let Some(game_mod) = read_installed_mod(self, &mod_id, manifest)? else {
				continue;
			};

			installed_mods.push(InstalledMod {
				game_mod,
				game: game.clone(),
			});
		}

		Ok(installed_mods)
	}

	fn get_game_environment(&self, game: &Game) -> Result<GameEnvironment> {
		let mut environment = BTreeMap::new();
		let mut environment_mod_ids = Vec::new();

		for installed_mod in self.get_installed_mods_for_game(game)? {
			let mod_environment = installed_mod.game_mod.get_game_environment(Some(game));
			if !mod_environment.is_empty() {
				environment_mod_ids.push(installed_mod.game_mod.id.clone());
			}
			environment.extend(mod_environment);
		}

		Ok(GameEnvironment {
			variables: environment,
			mod_ids: environment_mod_ids,
		})
	}

	fn get_mod_map(&self) -> Result<BTreeMap<String, Mod>> {
		read_mod_map(self, false)
	}

	fn get_catalog_mod_map(&self) -> Result<BTreeMap<String, Mod>> {
		read_mod_map(self, true)
	}

	fn refresh_installed_mods(&self) -> Result {
		let exe_path_hashes = self
			.read_db()?
			.prepare_cached(
				"SELECT DISTINCT exe_path FROM main.installed_games WHERE exe_path IS NOT NULL",
			)?
			.query_map([], |row| row.get::<_, String>(0))?
			.filter_map(|exe_path| match exe_path {
				Ok(exe_path) => Some(Path::new(&exe_path).hash_string()),
				Err(err) => {
					log::warn!("Failed to read installed game from local database: {err}");
					None
				}
			})
			.collect::<Vec<_>>();

		let mut installed_mod_rows = Vec::new();

		for exe_path_hash in &exe_path_hashes {
			let manifests_folder = app_paths::installed_mods_path()?
				.join(exe_path_hash)
				.join("manifests");

			for manifest in read_installed_manifests(&manifests_folder, InstallKind::PerGame) {
				installed_mod_rows.push((exe_path_hash.clone(), manifest));
			}
		}

		let shared_mods_folder = app_paths::shared_mods_path()?;

		for manifest in read_installed_manifests(&shared_mods_folder, InstallKind::Shared) {
			for exe_path_hash in &exe_path_hashes {
				installed_mod_rows.push((exe_path_hash.clone(), manifest.clone()));
			}
		}

		let mut connection = self.write_db()?;
		let transaction = connection.transaction()?;

		transaction.execute(
			"DELETE FROM main.mod_game_ids WHERE mod_id IN (SELECT id FROM main.mods WHERE in_catalog = 0);",
			[],
		)?;
		transaction.execute(
			"DELETE FROM main.mod_tag_links WHERE mod_id IN (SELECT id FROM main.mods WHERE in_catalog = 0);",
			[],
		)?;
		transaction.execute("DELETE FROM main.mods WHERE in_catalog = 0;", [])?;

		{
			let mut known_ids = transaction
				.prepare_cached("SELECT id FROM main.mods;")?
				.query_map([], |row| row.get::<_, String>(0))?
				.collect::<rusqlite::Result<HashSet<String>>>()?;

			for (_, installed_manifest) in &installed_mod_rows {
				let Some(manifest) = installed_manifest.manifest.as_ref() else {
					continue;
				};

				if known_ids.insert(manifest.id.clone()) {
					insert_mod_row(&transaction, manifest, false)?;
				}
			}
		}

		transaction.execute("DELETE FROM main.installed_mods;", [])?;

		{
			let mut statement = transaction.prepare_cached(
				"INSERT OR REPLACE INTO main.installed_mods (
					exe_path_hash,
					mod_id,
					manifest
				) VALUES ($1, $2, $3)",
			)?;

			for (exe_path_hash, installed_manifest) in installed_mod_rows {
				statement.execute(rusqlite::params![
					exe_path_hash,
					installed_manifest.id,
					installed_manifest
						.manifest
						.as_ref()
						.map(serde_json::to_string)
						.transpose()?,
				])?;
			}
		}

		transaction.commit()?;
		drop(connection);

		Ok(())
	}

	fn get_game_mods(&self, provider_id: &GameProviderId, game_id: &str) -> Result<Vec<ModInfo>> {
		let version_compatible = engine_compatibility::engine_version_in_range(
			VersionColumns {
				major: "cm.game_major",
				minor: "cm.game_minor",
				patch: "cm.game_patch",
			},
			VersionColumns {
				major: "cm.min_major",
				minor: "cm.min_minor",
				patch: "cm.min_patch",
			},
			VersionColumns {
				major: "cm.max_major",
				minor: "cm.max_minor",
				patch: "cm.max_patch",
			},
		);

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
					json_extract(im.manifest, '$.download.id') AS installed_version,
					m.optional_dependencies AS optional_dependencies,
					m.required_dependencies AS required_dependencies,
					m.tags AS tags,
					m.in_catalog AS in_catalog,
					CASE
						WHEN im.mod_id IS NULL THEN 0
						ELSE 1
					END AS is_installed,
					CASE
						WHEN im.mod_id IS NULL THEN 0
						WHEN json_extract(m.download, '$.id') IS NOT json_extract(im.manifest, '$.download.id') THEN 1
						WHEN m.hash IS NOT json_extract(im.manifest, '$.hash') THEN 1
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
			),
			installed_dependencies AS (
				SELECT dependant.mod_id AS mod_id, dependency.value AS dependency
				FROM candidate_mods dependant
				INNER JOIN json_each(dependant.optional_dependencies) dependency ON 1=1
				WHERE dependant.is_installed = 1
				UNION ALL
				SELECT dependant.mod_id AS mod_id, dependency.value AS dependency
				FROM candidate_mods dependant
				INNER JOIN json_each(dependant.required_dependencies) dependency ON 1=1
				WHERE dependant.is_installed = 1
			)
			SELECT
				cm.mod_id,
				cm.installed_version,
				cm.is_installed,
				cm.in_catalog AS is_in_catalog,
				cm.is_outdated,
				CASE
					WHEN EXISTS (
						SELECT 1
						FROM installed_dependencies dependant
						WHERE dependant.mod_id <> cm.mod_id
							AND (
								json_extract(dependant.dependency, '$.modId') = cm.mod_id
								OR EXISTS (SELECT 1 FROM main.mod_tag_links mod_tag_link WHERE mod_tag_link.mod_id = cm.mod_id AND mod_tag_link.tag = json_extract(dependant.dependency, '$.tag'))
							)
					) THEN 1
					ELSE 0
				END AS has_installed_dependants,
				CASE WHEN {version_compatible} THEN 1 ELSE 0 END AS compatible,
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
						is_installed: row.get(2)?,
						is_in_catalog: row.get(3)?,
						broken: false,
						is_outdated: row.get(4)?,
						has_installed_dependants: row.get(5)?,
						compatible: row.get(6)?,
						tags: row.get_json(7)?,
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
				ON mod_game_id.external_id_provider = game_external_id.external_id_provider
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

	fn get_broken_installed_manifests(&self, game: &Game) -> Result<Vec<String>> {
		let exe_path_hash = game.try_get_exe_path()?.hash_string();

		let file_names = self
			.read_db()?
			.prepare_cached(
				"SELECT mod_id FROM main.installed_mods
				WHERE exe_path_hash = $1
					AND manifest IS NULL
					AND mod_id NOT IN (SELECT id FROM main.mods)",
			)?
			.query_map([exe_path_hash], |row| row.get::<_, String>(0))?
			.collect::<rusqlite::Result<Vec<_>>>()?;

		Ok(file_names)
	}

	fn get_mod_tags(&self) -> Result<BTreeMap<String, ModTag>> {
		let rows = self
			.read_db()?
			.prepare_cached("SELECT id, data FROM main.mod_tags ORDER BY id;")?
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
}

fn read_installed_mod(db: &Db, mod_id: &str, manifest: Option<String>) -> Result<Option<Mod>> {
	if let Some(manifest) = manifest {
		match serde_json::from_str::<Mod>(&manifest) {
			Ok(manifest) => return Ok(Some(manifest)),
			Err(error) => {
				log::warn!("Failed to parse stored installed mod manifest for `{mod_id}`: {error}");
			}
		}
	}

	match db.get_mod(mod_id) {
		Ok(game_mod) => Ok(Some(game_mod)),
		Err(Error::ModNotFound(_)) => Ok(None),
		Err(error) => Err(error),
	}
}

fn read_installed_manifests(folder: &Path, install_kind: InstallKind) -> Vec<InstalledManifest> {
	let Ok(entries) = fs::read_dir(folder) else {
		return Vec::new();
	};

	entries
		.filter_map(|entry| {
			let path = entry.ok()?.path();

			let manifest_path = if install_kind == InstallKind::Shared {
				path.join(Mod::FILE_NAME)
			} else {
				path
			};

			if !manifest_path.is_file() {
				return None;
			}

			InstalledManifest::from_file(&manifest_path, install_kind)
		})
		.collect()
}

fn read_mod_map(db: &Db, catalog_only: bool) -> Result<BTreeMap<String, Mod>> {
	let filter = if catalog_only {
		" WHERE in_catalog = 1"
	} else {
		""
	};

	Ok(db
		.read_db()?
		.prepare_cached(&format!("SELECT {MOD_COLUMNS} FROM main.mods{filter}"))?
		.query_map([], read_mod)?
		.filter_map(|game_mod| match game_mod {
			Ok(parsed) => Some((parsed.id.clone(), parsed)),
			Err(err) => {
				log::warn!("Failed to read mod from local database: {err}");
				None
			}
		})
		.collect())
}

fn serialize_json_option<T: Serialize>(value: Option<&T>) -> Result<String> {
	serde_json::to_string(&value).map_err(Into::into)
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

fn insert_mod_row(
	transaction: &rusqlite::Transaction<'_>,
	game_mod: &Mod,
	in_catalog: bool,
) -> Result {
	transaction
		.prepare_cached(
			"INSERT OR REPLACE INTO mods (
				id,
				in_catalog,
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
				paths,
				optional_dependencies,
				required_dependencies,
				install,
				run_for_game,
				run_standalone,
				game_environment,
				hash,
				tags,
				game_ids
			) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24)",
		)?
		.execute(rusqlite::params![
			game_mod.id,
			in_catalog,
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
			serialize_json_option(game_mod.optional_dependencies.as_ref())?,
			serialize_json_option(game_mod.required_dependencies.as_ref())?,
			serialize_json_option(game_mod.install.as_ref())?,
			serialize_json_option(game_mod.run_for_game.as_ref())?,
			serialize_json_option(game_mod.run_standalone.as_ref())?,
			serialize_json_option(game_mod.game_environment.as_ref())?,
			game_mod.hash,
			serde_json::to_string(&game_mod.tags.clone().unwrap_or_default())?,
			serialize_json_option(game_mod.game_ids.as_ref())?,
		])?;

	if let Some(game_ids) = &game_mod.game_ids {
		let mut insert_game_id = transaction.prepare_cached(
			"INSERT OR IGNORE INTO main.mod_game_ids (mod_id, external_id_provider, external_id) VALUES ($1, $2, $3);",
		)?;

		for (provider, ids) in &game_ids.0 {
			for id in ids {
				insert_game_id.execute(rusqlite::params![game_mod.id, provider, id])?;
			}
		}
	}

	if let Some(tags) = &game_mod.tags {
		let mut insert_tag_link = transaction.prepare_cached(
			"INSERT OR IGNORE INTO main.mod_tag_links (mod_id, tag) VALUES ($1, $2);",
		)?;

		for tag in tags {
			insert_tag_link.execute(rusqlite::params![game_mod.id, tag])?;
		}
	}

	Ok(())
}

fn insert_mod_tag(transaction: &rusqlite::Transaction<'_>, tag_id: &str, tag: &ModTag) -> Result {
	transaction
		.prepare_cached("INSERT OR REPLACE INTO main.mod_tags (id, data) VALUES ($1, $2)")?
		.execute(rusqlite::params![tag_id, serde_json::to_string(tag)?])?;

	Ok(())
}

fn try_replace_catalog(db: &Db, mods: &[Mod], tags: &BTreeMap<String, ModTag>) -> Result {
	let mut connection = db.write_db()?;
	let transaction = connection.transaction()?;

	transaction.execute_batch(
		r"
		DELETE FROM main.mod_tag_links;
		DELETE FROM main.mod_game_ids;
		DELETE FROM main.mods;
		DELETE FROM main.mod_tags;
		",
	)?;

	for game_mod in mods {
		insert_mod_row(&transaction, game_mod, true)?;
	}

	for (tag_id, tag) in tags {
		insert_mod_tag(&transaction, tag_id, tag)?;
	}

	transaction.commit()?;
	drop(connection);

	Ok(())
}
