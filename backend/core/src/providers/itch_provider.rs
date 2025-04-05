use std::{
	collections::HashMap,
	path::{Path, PathBuf},
};

use chrono::DateTime;
use log::error;
use rai_pal_proc_macros::serializable_struct;
use sqlx::{Row, sqlite::SqlitePool};

use super::provider_command::{ProviderCommand, ProviderCommandAction};
use crate::{
	game::{DbGame, GameId},
	installed_game::InstalledGame,
	providers::provider::{ProviderActions, ProviderId, ProviderStatic},
	result::{Error, Result},
};

#[derive(Clone)]
pub struct Itch {}

impl Itch {
	fn get_installed_game(cave: &ItchDatabaseCave) -> Option<InstalledGame> {
		let verdict = cave.verdict.as_ref()?;
		let exe_path = verdict.base_path.join(&verdict.candidates.first()?.path);
		InstalledGame::new(&exe_path)
	}

	// fn get_game(row: &ItchDatabaseGame) -> DbGame {
	// 	let mut game = Game::new(
	// 		GameId {
	// 			game_id: row.id.to_string(),
	// 			provider_id: *Self::ID,
	// 		},
	// 		&row.title,
	// 	);

	// 	if let Some(thumbnail_url) = &row.cover_url {
	// 		game.set_thumbnail_url(thumbnail_url);
	// 	}
	// 	if let Some(date_time) = row
	// 		.published_at
	// 		.as_ref()
	// 		.and_then(|published_at| DateTime::parse_from_rfc3339(published_at).ok())
	// 	{
	// 		game.set_release_date(date_time.timestamp());
	// 	}
	// 	game.add_provider_command(
	// 		ProviderCommandAction::ShowInLibrary,
	// 		ProviderCommand::String(format!("itch://games/{}", row.id)),
	// 	)
	// 	.add_provider_command(
	// 		ProviderCommandAction::Install,
	// 		ProviderCommand::String(format!("itch://install?game_id={}", row.id)),
	// 	);

	// 	game
	// }
}

impl ProviderStatic for Itch {
	const ID: &'static ProviderId = &ProviderId::Itch;

	fn new() -> Result<Self>
	where
		Self: Sized,
	{
		Ok(Self {})
	}
}

#[serializable_struct]
pub struct ItchDatabaseGame {
	id: i32,
	title: String,
	url: Option<String>,
	published_at: Option<String>,
	cover_url: Option<String>,
}

#[serializable_struct]
pub struct ItchDatabaseCave {
	id: i32,
	verdict: Option<ItchDatabaseVerdict>,
	title: String,
	cover_url: Option<String>,
}

#[serializable_struct]
pub struct ItchDatabaseVerdict {
	base_path: PathBuf,
	candidates: Vec<ItchDatabaseCandidate>,
}

#[serializable_struct]
pub struct ItchDatabaseCandidate {
	path: PathBuf,
}

#[serializable_struct]
pub struct ItchDatabase {
	games: Vec<ItchDatabaseGame>,
	caves: Vec<ItchDatabaseCave>,
}

impl ProviderActions for Itch {
	async fn get_games<TCallback>(&self, mut callback: TCallback) -> Result
	where
		TCallback: FnMut(DbGame) + Send + Sync,
	{
		// let app_data_path = directories::BaseDirs::new()
		// 	.ok_or_else(Error::AppDataNotFound)?
		// 	.config_dir()
		// 	.join("itch");

		// if let Some(database) = get_database(&app_data_path).await? {
		// 	let caves_map: HashMap<_, _> = database
		// 		.caves
		// 		.into_iter()
		// 		.map(|cave| (cave.id, cave))
		// 		.collect();

		// 	for db_entry in database.games {
		// 		let mut game = Self::get_game(&db_entry);
		// 		if let Some(cave) = caves_map.get(&db_entry.id) {
		// 			game.installed_game = Self::get_installed_game(cave);
		// 		}
		// 		callback(game);
		// 	}
		// } else {
		// 	log::info!(
		// 		"Itch database file not found. Probably means user hasn't installed the Itch app."
		// 	);
		// }

		Ok(())
	}
}

fn parse_verdict(json_option: Option<String>) -> Option<ItchDatabaseVerdict> {
	let json = json_option?;
	match serde_json::from_str(&json) {
		Ok(verdict) => Some(verdict),
		Err(err) => {
			error!("Failed to parse verdict from json `{json}`. Error: {err}");
			None
		}
	}
}

async fn get_database(app_data_path: &Path) -> Result<Option<ItchDatabase>> {
	let db_path = app_data_path.join("db").join("butler.db");

	if !db_path.is_file() {
		return Ok(None);
	}

	let pool = SqlitePool::connect(&db_path.to_string_lossy()).await?;

	let cave_rows = sqlx::query(
		r"
		SELECT
			caves.game_id, caves.verdict, games.title, games.cover_url
		FROM
			caves
		JOIN
			games ON caves.game_id = games.id;
	",
	)
	.fetch_all(&pool)
	.await?
	.into_iter()
	.map(|row| {
		Ok(ItchDatabaseCave {
			id: row.get("game_id"),
			title: row.get("title"),
			verdict: parse_verdict(row.try_get("verdict").ok()),
			cover_url: row.get("cover_url"),
		})
	})
	.collect::<Result<Vec<_>>>()?;

	let game_rows = sqlx::query(
		r"
		SELECT
			id, title, url, published_at, cover_url
		FROM
			'games'
		WHERE
			type='default' AND classification='game'
	",
	)
	.fetch_all(&pool)
	.await?
	.into_iter()
	.map(|row| {
		Ok(ItchDatabaseGame {
			id: row.get(0),
			title: row.get(1),
			url: row.get(2),
			published_at: row.get(3),
			cover_url: row.get(4),
		})
	})
	.collect::<Result<Vec<_>>>()?;

	Ok(Some(ItchDatabase {
		games: game_rows,
		caves: cave_rows,
	}))
}
