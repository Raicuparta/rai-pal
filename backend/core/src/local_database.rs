use std::{
	ops::Deref,
	path::Path,
	sync::Mutex,
	time::{SystemTime, UNIX_EPOCH},
};

use rusqlite::OpenFlags;

use crate::{
	game::DbGame, game_engines::game_engine::EngineBrand, game_title::get_normalized_titles, paths,
	providers::provider::ProviderId, remote_game, result::Result,
};

pub type DbMutex = Mutex<rusqlite::Connection>;

pub trait InsertGame {
	fn insert_game(&self, game: &DbGame);
}

impl InsertGame for DbMutex {
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
}

fn try_insert_game(connection_mutex: &DbMutex, game: &DbGame) -> Result {
	let mut connection = connection_mutex.lock().unwrap();
	let transaction = connection.transaction()?;

	// TODO prepare this only once since it's always the same.
	transaction
		.prepare(
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
			SystemTime::now()
				.duration_since(UNIX_EPOCH)
				.unwrap()
				.as_secs()
		])?;

	if let Some(exe_path) = game.exe_path.as_ref() {
		transaction
			.prepare(
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
			.prepare(
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

	Ok(())
}

pub fn create() -> Result<DbMutex> {
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

	Ok(DbMutex::new(connection))
}

pub fn attach_remote_database<TConnection: Deref<Target = rusqlite::Connection>>(
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
