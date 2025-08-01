use std::{
	ops::Deref,
	path::{Path, PathBuf},
	sync::{Mutex, MutexGuard},
	time::{Instant, SystemTime, UNIX_EPOCH},
};

use rai_pal_proc_macros::serializable_struct;
use rusqlite::OpenFlags;

use crate::{
	debug::LoggableInstant,
	game::DbGame,
	game_title::get_normalized_titles,
	games_query::{GamesQuery, GamesSortBy, InstallState},
	paths,
	providers::provider::ProviderId,
	remote_game,
	result::{Error, Result},
};

pub type DbMutex = Mutex<rusqlite::Connection>;

pub trait GameDatabase {
	fn lock_db(&self) -> Result<MutexGuard<'_, rusqlite::Connection>>;
	fn insert_game(&self, game: &DbGame);
	fn get_game(&self, provider_id: &ProviderId, game_id: &str) -> Result<DbGame>;
	fn get_game_ids(&self, query: Option<GamesQuery>) -> Result<GameIdsResponse>;
	fn remove_stale_games(&self, provider_id: &ProviderId, max_time: u64) -> Result;
}

#[serializable_struct]
pub struct GameIdsResponse {
	game_ids: Vec<(ProviderId, String)>,
	total_count: i64,
}

impl GameDatabase for DbMutex {
	fn lock_db(&self) -> Result<MutexGuard<'_, rusqlite::Connection>> {
		self.lock()
			.map_err(|err| Error::DatabaseLockFailed(err.to_string()))
	}

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

	fn get_game(&self, provider_id: &ProviderId, game_id: &str) -> Result<DbGame> {
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
						provider.as_ref().map(|p| format!("g.provider_id = '{p}'"))
					})
					.collect();
				if !provider_conditions.is_empty() {
					filters.push(format!("({})", provider_conditions.join(" OR ")));
				}
			}

			if !filter.tags.is_empty() {
				let tag_conditions: Vec<String> = filter
					.tags
					.iter()
					.map(|tag| {
						tag.as_ref().map_or_else(
							|| "g.tags = '[]'".to_string(),
							|t| format!("g.tags LIKE '%\"{t}\"%'"),
						)
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
					.filter_map(|engine| engine.as_ref().map(|e| format!("'{e}'")))
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
							.map(|b| format!("ig.unity_backend = '{b}'"))
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
				"(g.display_title LIKE '%{trimmed_search}%' OR nt.normalized_title LIKE '%{trimmed_search}%')"
			));
		}

		// Combine all filters into a single WHERE clause
		let where_clause = if filters.is_empty() {
			"1=1".to_string() // No filters, match all rows
		} else {
			filters.join(" AND ")
		};

		let sql_query = &format!(
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
			ORDER BY {}
			",
			sort_columns
				.iter()
				.map(|col| format!("{col} {sort_order}"))
				.collect::<Vec<_>>()
				.join(", ")
		);

		let game_ids = self
			.lock_db()?
			.prepare(sql_query)?
			.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
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
			total_count,
		})
	}

	fn remove_stale_games(&self, provider_id: &ProviderId, max_time: u64) -> Result {
		self.lock_db()?
			.prepare_cached("DELETE FROM main.games WHERE provider_id = $1 AND created_at < $2;")?
			.execute(rusqlite::params![provider_id, max_time])?;

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
				created_at
			) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
		)?
		.execute(rusqlite::params![
			game.provider_id,
			game.game_id.clone(),
			game.external_id.clone(),
			game.display_title.clone(),
			game.thumbnail_url.clone(),
			game.release_date,
			game.tags.clone(),
			game.title_discriminator.clone(),
			game.provider_commands.clone(),
			SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs()
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
					architecture
				)
				 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
			)?
			.execute(rusqlite::params![
				game.provider_id,
				game.game_id.clone(),
				exe_path,
				game.engine_brand,
				game.engine_version_major,
				game.engine_version_minor,
				game.engine_version_patch,
				game.engine_version_display.clone(),
				game.unity_backend.clone(),
				game.architecture.clone(),
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

pub fn create() -> Result<DbMutex> {
	let mut instant = Instant::now();
	instant.log_next("Creating local database...");

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
			PRIMARY KEY (provider_id, external_id)
		);
	",
	)?;

	attach_remote_database(&connection, &remote_game::get_database_file_path()?)?;

	instant.log_next("Created local database!");

	Ok(DbMutex::new(connection))
}

pub fn attach_remote_database<TConnection: Deref<Target = rusqlite::Connection>>(
	local_database_connection: TConnection,
	path: &Path,
) -> Result {
	let mut instant = Instant::now();
	instant.log_next("Attaching remote database...");

	if !path.is_file() {
		return Ok(());
	}

	let path_str = path.to_string_lossy();

	local_database_connection
		.execute(&format!("ATTACH DATABASE '{path_str}' AS remote_db;"), [])?;

	local_database_connection.execute(
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

	let mut update_statement = local_database_connection.prepare_cached(
		"UPDATE main.remote_games SET engine_version_major = ?, engine_version_minor = ?, engine_version_patch = ? 
		 WHERE provider_id = ? AND external_id = ? AND engine_version_display = ?"
	)?;

	let mut select_statement = local_database_connection.prepare_cached(
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

	local_database_connection.execute("DETACH DATABASE remote_db;", [])?;

	instant.log_next("Finished attaching up remote games database.");

	Ok(())
}

fn db_file_path() -> Result<PathBuf> {
	Ok(paths::app_data_path()?.join("db.sqlite"))
}
