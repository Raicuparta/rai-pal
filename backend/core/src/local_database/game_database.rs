use std::{
	fs,
	path::{
		Path,
		PathBuf,
	},
	time::{
		Instant,
		SystemTime,
		UNIX_EPOCH,
	},
};

use rai_pal_proc_macros::serializable_struct;
use rusqlite::OpenFlags;

use crate::{
	app_paths,
	debug::LoggableInstant,
	game::DbGame,
	game_providers::game_provider::GameProviderId,
	game_title::get_normalized_titles,
	games_query::{
		FilterGroup,
		GamesQuery,
		GamesSortBy,
		InstallState,
	},
	local_database::{
		app_database::{
			AppDatabase,
			DbMutex,
		},
		rusqlite_extensions::{
			JsonData,
			RowExt,
		},
	},
	operating_system::OperatingSystem,
	path_extensions::{
		AsValidStr,
		PathExt,
	},
	remote_game,
	result::Result,
};

pub trait GameDatabase {
	fn insert_game(&self, game: &DbGame);
	fn get_game(&self, provider_id: &GameProviderId, game_id: &str) -> Result<DbGame>;
	fn get_game_ids(&self, query: Option<GamesQuery>) -> Result<GameIdsResponse>;
	fn remove_stale_games(&self, provider_id: &GameProviderId, max_time: u64) -> Result;
}

#[serializable_struct]
pub struct GameIdsResponse {
	pub game_ids: Vec<(GameProviderId, String)>,
	pub total_count: u32,
}

fn build_nullable_exclusion_filter<T: std::fmt::Display + std::hash::Hash + std::cmp::Eq>(
	column_expr: &str,
	group: &FilterGroup<T>,
	params: &mut Vec<Box<dyn rusqlite::types::ToSql>>,
) -> Option<String> {
	let mut conditions: Vec<String> = Vec::new();

	if group.unknown.as_ref().is_some_and(|item| !item.enabled) {
		conditions.push(format!("{column_expr} IS NOT NULL"));
	}

	let disabled: Vec<&T> = group
		.known
		.iter()
		.filter(|(_, item)| !item.enabled)
		.map(|(val, _)| val)
		.collect();

	if !disabled.is_empty() {
		let placeholders: Vec<_> = disabled
			.iter()
			.map(|val| {
				params.push(Box::new(val.to_string()));
				"?".to_string()
			})
			.collect();
		let cond = if group.unknown.as_ref().is_none_or(|item| item.enabled) {
			format!(
				"{column_expr} NOT IN ({}) OR {column_expr} IS NULL",
				placeholders.join(", ")
			)
		} else {
			format!("{column_expr} NOT IN ({})", placeholders.join(", "))
		};
		conditions.push(cond);
	}

	if conditions.is_empty() {
		None
	} else {
		Some(format!("({})", conditions.join(" AND ")))
	}
}

impl GameDatabase for DbMutex {
	fn insert_game(&self, game: &DbGame) {
		if let Err(err) = try_insert_game(self, game) {
			log::error!(
				"Failed to insert game ({}/{}) into local database: {}",
				game.provider_id,
				game.game_id,
				err
			);
		}
	}

	fn get_game(&self, provider_id: &GameProviderId, game_id: &str) -> Result<DbGame> {
		Ok(self
			.lock_db()?
			.prepare_cached(
				r"
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
			ig.os,
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
	",
			)?
			.query_row([provider_id.to_string(), game_id.to_string()], |row| {
				Ok(DbGame {
					provider_id: row.get(0)?,
					game_id: row.get(1)?,
					external_id: row.get(2)?,
					display_title: row.get(3)?,
					title_discriminator: row.get(4)?,
					thumbnail_url: row.get(5)?,
					release_date_rfc3339: row.get(6)?,
					tags: row.get_json(7)?,
					provider_commands: row.get_json(8)?,
					exe_path: row.get_path(9)?,
					unity_backend: row.get(10)?,
					architecture: row.get(11)?,
					os: row.get(12)?,
					engine_brand: row.get(13)?,
					engine_version_major: row.get(14)?,
					engine_version_minor: row.get(15)?,
					engine_version_patch: row.get(16)?,
					engine_version_display: row.get(17)?,
				})
			})?)
	}

	fn get_game_ids(&self, query: Option<GamesQuery>) -> Result<GameIdsResponse> {
		let search = query.as_ref().map(|q| q.search.clone()).unwrap_or_default();

		// Build sorting logic
		let sort_columns = match query.as_ref().map(|q| q.sort_by) {
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

		// Build filtering logic dynamically with parameterized queries
		let mut filters = Vec::<String>::new();
		let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

		if let Some(filter) = query.as_ref().map(|q| &q.filter) {
			// Installed filter
			let installed_disabled = filter
				.installed
				.known
				.get(&InstallState::Installed)
				.is_some_and(|item| !item.enabled);
			let not_installed_disabled = filter
				.installed
				.known
				.get(&InstallState::NotInstalled)
				.is_some_and(|item| !item.enabled);
			if installed_disabled {
				filters.push("ig.exe_path IS NULL".to_string());
			}
			if not_installed_disabled {
				filters.push("ig.exe_path IS NOT NULL".to_string());
			}

			// Providers filter
			{
				let conditions: Vec<String> = filter
					.providers
					.known
					.iter()
					.filter(|(_, item)| !item.enabled)
					.map(|(provider, _)| {
						params.push(Box::new(provider.to_string()));
						"g.provider_id = ?".to_string()
					})
					.collect();
				if !conditions.is_empty() {
					filters.push(format!("NOT ({})", conditions.join(" OR ")));
				}
			}

			// Tags filter
			{
				let mut conditions: Vec<String> = Vec::new();
				if filter
					.tags
					.unknown
					.as_ref()
					.is_some_and(|item| !item.enabled)
				{
					conditions.push("g.tags = '[]'".to_string());
				}
				for (tag, item) in &filter.tags.known {
					if !item.enabled {
						params.push(Box::new(format!(r#"%"{tag}"%"#)));
						conditions.push("g.tags NOT LIKE ?".to_string());
					}
				}
				if !conditions.is_empty() {
					filters.push(format!("({})", conditions.join(" AND ")));
				}
			}

			// Engines filter
			if let Some(cond) = build_nullable_exclusion_filter(
				"COALESCE(ig.engine_brand, rg.engine_brand)",
				&filter.engines,
				&mut params,
			) {
				filters.push(cond);
			}

			// Unity backends filter
			if let Some(cond) = build_nullable_exclusion_filter(
				"ig.unity_backend",
				&filter.unity_backends,
				&mut params,
			) {
				filters.push(cond);
			}

			// Architectures filter
			if let Some(cond) = build_nullable_exclusion_filter(
				"ig.architecture",
				&filter.architectures,
				&mut params,
			) {
				filters.push(cond);
			}

			// Operating systems filter
			if let Some(cond) = build_nullable_exclusion_filter(
				"ig.os",
				&filter.os,
				&mut params,
			) {
				filters.push(cond);
			}

			// Mod families filter
			{
				let disabled: Vec<_> = filter
					.mod_families
					.known
					.iter()
					.filter(|(_, item)| !item.enabled)
					.collect();
				if !disabled.is_empty() {
					let family_placeholders: Vec<String> = disabled
						.iter()
						.map(|(family, _)| {
							params.push(Box::new((*family).clone()));
							"?".to_string()
						})
						.collect();

					let current_os = OperatingSystem::get_current().to_string();
					params.push(Box::new(current_os));

					filters.push(format!(
						r"NOT EXISTS (
						SELECT 1 FROM main.mods m
						WHERE m.family IN ({})
						AND (json_extract(m.engine, '$') IS NULL OR json_extract(m.engine, '$') = COALESCE(ig.engine_brand, rg.engine_brand))
						AND (json_extract(m.unity_backend, '$') IS NULL OR ig.unity_backend IS NULL OR json_extract(m.unity_backend, '$') = ig.unity_backend)
						AND (json_extract(m.architecture, '$') IS NULL OR ig.architecture IS NULL OR json_extract(m.architecture, '$') = ig.architecture)
						AND (json_extract(m.host_os, '$') IS NULL OR json_extract(m.host_os, '$') = ?)
						AND (
							COALESCE(ig.engine_version_major, rg.engine_version_major) IS NULL
							OR (
								(
									json_extract(m.engine_version_range, '$.minimum.major') IS NULL
									OR NOT (
										json_extract(m.engine_version_range, '$.minimum.major') > COALESCE(ig.engine_version_major, rg.engine_version_major)
										OR (
											json_extract(m.engine_version_range, '$.minimum.major') = COALESCE(ig.engine_version_major, rg.engine_version_major)
											AND json_extract(m.engine_version_range, '$.minimum.minor') IS NOT NULL
											AND COALESCE(ig.engine_version_minor, rg.engine_version_minor) IS NOT NULL
											AND (
												json_extract(m.engine_version_range, '$.minimum.minor') > COALESCE(ig.engine_version_minor, rg.engine_version_minor)
												OR (
													json_extract(m.engine_version_range, '$.minimum.minor') = COALESCE(ig.engine_version_minor, rg.engine_version_minor)
													AND json_extract(m.engine_version_range, '$.minimum.patch') IS NOT NULL
													AND COALESCE(ig.engine_version_patch, rg.engine_version_patch) IS NOT NULL
													AND json_extract(m.engine_version_range, '$.minimum.patch') > COALESCE(ig.engine_version_patch, rg.engine_version_patch)
												)
											)
										)
									)
								)
								AND (
									json_extract(m.engine_version_range, '$.maximum.major') IS NULL
									OR NOT (
										json_extract(m.engine_version_range, '$.maximum.major') < COALESCE(ig.engine_version_major, rg.engine_version_major)
										OR (
											json_extract(m.engine_version_range, '$.maximum.major') = COALESCE(ig.engine_version_major, rg.engine_version_major)
											AND json_extract(m.engine_version_range, '$.maximum.minor') IS NOT NULL
											AND COALESCE(ig.engine_version_minor, rg.engine_version_minor) IS NOT NULL
											AND (
												json_extract(m.engine_version_range, '$.maximum.minor') < COALESCE(ig.engine_version_minor, rg.engine_version_minor)
												OR (
													json_extract(m.engine_version_range, '$.maximum.minor') = COALESCE(ig.engine_version_minor, rg.engine_version_minor)
													AND json_extract(m.engine_version_range, '$.maximum.patch') IS NOT NULL
													AND COALESCE(ig.engine_version_patch, rg.engine_version_patch) IS NOT NULL
													AND json_extract(m.engine_version_range, '$.maximum.patch') < COALESCE(ig.engine_version_patch, rg.engine_version_patch)
												)
											)
										)
									)
								)
							)
						)
					)",
						family_placeholders.join(", ")
					));
				}
			}
		}

		let trimmed_search = search.trim();
		if !trimmed_search.is_empty() {
			params.push(Box::new(format!("%{trimmed_search}%")));
			params.push(Box::new(format!("%{trimmed_search}%")));
			filters.push("(g.display_title LIKE ? OR nt.normalized_title LIKE ?)".to_string());
		}

		let where_clause = if filters.is_empty() {
			"1=1".to_string()
		} else {
			filters.join(" AND ")
		};

		let sort_clause = sort_columns
			.iter()
			.map(|col| format!("{col} {sort_order}"))
			.collect::<Vec<_>>()
			.join(", ");

		let sql_query = format!(
			r"
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
			ORDER BY {sort_clause}
			",
		);

		let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| &**p).collect();
		let game_ids = self
			.lock_db()?
			.prepare(&sql_query)?
			.query_map(param_refs.as_slice(), |row| Ok((row.get(0)?, row.get(1)?)))?
			.filter_map(|game_id| match game_id {
				Ok(id) => Some(id),
				Err(err) => {
					log::warn!("Failed to read game from local database: {err}");
					None
				}
			})
			.collect();

		let total_count = self
			.lock_db()?
			.prepare_cached(
				r"
			SELECT COUNT(*)
			FROM main.games g
		",
			)?
			.query_row([], |row| row.get::<_, i64>(0))?;

		Ok(GameIdsResponse {
			game_ids,
			total_count: u32::try_from(total_count)?,
		})
	}

	fn remove_stale_games(&self, provider_id: &GameProviderId, max_time: u64) -> Result {
		let db = self.lock_db()?;
		db.prepare_cached(
			"DELETE FROM main.installed_games WHERE provider_id = $1 AND game_id IN (SELECT game_id FROM main.games WHERE provider_id = $1 AND created_at < $2)",
		)?
		.execute(rusqlite::params![provider_id, max_time.cast_signed()])?;
		db.prepare_cached(
			"DELETE FROM main.normalized_titles WHERE provider_id = $1 AND game_id IN (SELECT game_id FROM main.games WHERE provider_id = $1 AND created_at < $2)",
		)?
		.execute(rusqlite::params![provider_id, max_time.cast_signed()])?;
		db.prepare_cached("DELETE FROM main.games WHERE provider_id = $1 AND created_at < $2")?
			.execute(rusqlite::params![provider_id, max_time.cast_signed()])?;

		drop(db);
		Ok(())
	}
}

fn try_insert_game(connection_mutex: &DbMutex, game: &DbGame) -> Result {
	let mut connection = connection_mutex.lock_db()?;
	let transaction = connection.transaction()?;

	transaction
		.prepare_cached(
			"INSERT OR REPLACE INTO games (
				provider_id,
				game_id,
				external_id,
				display_title,
				thumbnail_url,
				release_date,
				tags,
				title_discriminator,
				provider_commands,
				exe_path_hash,
				created_at
			) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)",
		)?
		.execute(rusqlite::params![
			game.provider_id,
			game.game_id.clone(),
			game.external_id.clone(),
			game.display_title.clone(),
			game.thumbnail_url.clone(),
			game.release_date_rfc3339,
			JsonData(game.tags.clone()),
			game.title_discriminator.clone(),
			JsonData(game.provider_commands.clone()),
			game.exe_path
				.as_ref()
				.map(|exe_path| exe_path.as_path().hash_string()),
			SystemTime::now()
				.duration_since(UNIX_EPOCH)?
				.as_secs()
				.cast_signed()
		])?;

	if let Some(exe_path) = game.exe_path.as_ref() {
		transaction
			.prepare_cached(
				"INSERT OR REPLACE INTO installed_games (
					provider_id,
					game_id,
					exe_path,
					engine_brand,
					engine_version_major,
					engine_version_minor,
					engine_version_patch,
					engine_version_display,
					unity_backend,
					architecture,
					os
				)
				 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)",
			)?
			.execute(rusqlite::params![
				game.provider_id,
				game.game_id.clone(),
				exe_path.try_to_str()?,
				game.engine_brand,
				game.engine_version_major,
				game.engine_version_minor,
				game.engine_version_patch,
				game.engine_version_display.clone(),
				game.unity_backend.clone(),
				game.architecture.clone(),
				game.os,
			])?;
	}

	for normalized_title in get_normalized_titles(&game.display_title) {
		transaction
			.prepare_cached(
				"INSERT OR REPLACE INTO normalized_titles (provider_id, game_id, normalized_title)
						VALUES ($1, $2, $3)",
			)?
			.execute(rusqlite::params![
				game.provider_id,
				game.game_id.clone(),
				normalized_title.clone(),
			])?;
	}

	transaction.commit()?;

	// tbh only here due to a clippy warning,
	// Clippy does't seem to realize connection is needed as long as transaction.
	drop(connection);

	Ok(())
}

pub fn try_create() -> Result<DbMutex> {
	match create() {
		Ok(db) => Ok(db),
		Err(initial_error) => {
			log::error!(
				"Failed to set up local databases. Deleting them and retrying once. Error: {initial_error}"
			);
			cleanup_database_files();

			match create() {
				Ok(db) => Ok(db),
				Err(retry_error) => {
					log::error!(
						"Failed to set up local databases after cleanup retry. Giving up. Error: {retry_error}"
					);
					Err(retry_error)
				}
			}
		}
	}
}

pub fn attach_remote(path: &Path) -> Result {
	let mut instant = Instant::now();
	instant.log_next("Attaching remote database...");

	if !path.is_file() {
		return Ok(());
	}

	let mut connection = open_attach_database_connection()?;
	if is_database_attached(&connection, "remote_db")? {
		connection.execute("DETACH DATABASE remote_db;", [])?;
	}

	let attach_work_result = (|| -> Result {
		connection.execute("ATTACH DATABASE ?1 AS remote_db;", [path.try_to_str()?])?;

		let transaction = connection.transaction()?;

		transaction.execute(
			r"
			INSERT OR IGNORE INTO main.remote_games (
				provider_id, external_id, engine_brand, engine_version_major,
				engine_version_minor, engine_version_patch, engine_version_display
			)
			SELECT
				provider_id,
				external_id,
				engine_brand,
				NULL,
				NULL,
				NULL,
				engine_version
			FROM remote_db.games;
			",
			[],
		)?;

		{
			let mut update_statement = transaction.prepare_cached(
				"UPDATE main.remote_games SET engine_version_major = ?, engine_version_minor = ?, engine_version_patch = ?
				 WHERE provider_id = ? AND external_id = ? AND engine_version_display = ?"
			)?;

			let mut select_statement = transaction.prepare_cached(
				"SELECT
					provider_id,
					external_id,
					engine_version
				FROM
					remote_db.games
				WHERE
					engine_version IS NOT NULL AND engine_version != ''",
			)?;

			let rows = select_statement.query_map([], |row| {
				let provider_id: String = row.get(0)?;
				let external_id: String = row.get(1)?;
				let engine_version: String = row.get(2)?;
				Ok((provider_id, external_id, engine_version))
			})?;

			for row_result in rows {
				match row_result {
					Ok((provider_id, external_id, engine_version)) => {
						if let Some(parsed) = remote_game::parse_version(&engine_version) {
							update_statement.execute(rusqlite::params![
								parsed.numbers.major,
								parsed.numbers.minor,
								parsed.numbers.patch,
								provider_id,
								external_id,
								engine_version
							])?;
						}
					}
					Err(err) => {
						log::warn!("Failed to read remote game row: {err}");
					}
				}
			}
		}

		transaction.commit()?;
		Ok(())
	})();

	let detach_result = match is_database_attached(&connection, "remote_db") {
		Ok(true) => connection
			.execute("DETACH DATABASE remote_db;", [])
			.map(|_| ())
			.map_err(Into::into),
		Ok(false) => Ok(()),
		Err(err) => Err(err),
	};

	match (attach_work_result, detach_result) {
		(Ok(()), Ok(())) => {}
		(Err(work_err), Ok(())) => return Err(work_err),
		(Ok(()), Err(detach_err)) => return Err(detach_err),
		(Err(work_err), Err(detach_err)) => {
			log::warn!("Failed to detach remote_db after attach_remote error: {detach_err}");
			return Err(work_err);
		}
	}

	instant.log_next("Finished attaching up remote games database.");

	Ok(())
}

fn cleanup_database_files() {
	cleanup_database_file(db_file_path(), "local");
	cleanup_database_file(remote_game::get_database_file_path(), "remote");
}

fn cleanup_database_file(path_result: Result<PathBuf>, database_name: &str) {
	let path = match path_result {
		Ok(path) => path,
		Err(path_error) => {
			log::warn!("Failed to resolve {database_name} database path for cleanup: {path_error}");
			return;
		}
	};

	match fs::remove_file(&path) {
		Ok(()) => {}
		Err(remove_error) if remove_error.kind() == std::io::ErrorKind::NotFound => {}
		Err(remove_error) => {
			log::warn!(
				"Failed to delete {database_name} database after create failure ({}): {remove_error}",
				path.display()
			);
		}
	}
}

fn create() -> Result<DbMutex> {
	let mut instant = Instant::now();
	instant.log_next("Creating local database...");

	let connection = open_main_database_connection()?;

	connection.execute_batch(
		r"
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
			exe_path_hash TEXT,
			created_at INTEGER,
			PRIMARY KEY (provider_id, game_id)
		);

		CREATE INDEX IF NOT EXISTS idx_games_external_id ON games(provider_id, external_id);
		CREATE INDEX IF NOT EXISTS idx_games_exe_path_hash ON games(exe_path_hash);

		CREATE TABLE IF NOT EXISTS normalized_titles (
			provider_id TEXT NOT NULL,
			game_id TEXT NOT NULL,
			normalized_title TEXT NOT NULL,
			FOREIGN KEY (provider_id, game_id) REFERENCES games(provider_id, game_id) ON DELETE CASCADE,
			PRIMARY KEY (provider_id, game_id, normalized_title)
		);

		CREATE INDEX IF NOT EXISTS idx_normalized_titles ON normalized_titles(provider_id, game_id);
		CREATE INDEX IF NOT EXISTS idx_normalized_titles_title ON normalized_titles(normalized_title);

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
			os TEXT,
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
			PRIMARY KEY (provider_id, external_id)
		);
	",
	)?;

	let remote_database_path = remote_game::get_database_file_path()?;

	let mutex = DbMutex::new(connection);

	attach_remote(&remote_database_path)?;

	instant.log_next("Created local database!");

	Ok(mutex)
}

fn open_main_database_connection() -> Result<rusqlite::Connection> {
	open_local_database_connection()
}

fn open_attach_database_connection() -> Result<rusqlite::Connection> {
	open_local_database_connection()
}

fn open_local_database_connection() -> Result<rusqlite::Connection> {
	let path = db_file_path()?;
	if let Some(parent) = path.parent() {
		std::fs::create_dir_all(parent)?;
	}

	let connection = rusqlite::Connection::open_with_flags(
		path,
		OpenFlags::SQLITE_OPEN_CREATE
			| OpenFlags::SQLITE_OPEN_READ_WRITE
			| OpenFlags::SQLITE_OPEN_PRIVATE_CACHE,
	)?;

	connection.execute_batch(
		r"
		PRAGMA journal_mode = WAL;
		PRAGMA synchronous = OFF;
	",
	)?;

	Ok(connection)
}

fn is_database_attached(connection: &rusqlite::Connection, database_name: &str) -> Result<bool> {
	let databases = connection
		.prepare_cached("PRAGMA database_list;")?
		.query_map([], |row| row.get::<_, String>(1))?
		.collect::<rusqlite::Result<Vec<_>>>()?;

	Ok(databases.iter().any(|name| name == database_name))
}

fn db_file_path() -> Result<PathBuf> {
	app_paths::database_path("local")
}
