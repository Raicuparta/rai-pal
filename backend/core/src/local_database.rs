use std::{
	sync::Mutex,
	time::{SystemTime, UNIX_EPOCH},
};

use crate::{game::DbGame, game_title::get_normalized_titles, result::Result};

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
