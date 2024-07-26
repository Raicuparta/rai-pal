use std::path::{Path, PathBuf};

use async_trait::async_trait;
use chrono::DateTime;
use futures::future;
use log::error;
use rai_pal_proc_macros::serializable_struct;
use rusqlite::{Connection, OpenFlags};

use super::provider_command::{ProviderCommand, ProviderCommandAction};
use crate::{
	installed_game::InstalledGame,
	owned_game::OwnedGame,
	pc_gaming_wiki,
	providers::provider::{ProviderActions, ProviderId, ProviderStatic},
	remote_game::{self, RemoteGame},
	result::{Error, Result},
};

#[derive(Clone)]
pub struct Itch {
	remote_game_cache: remote_game::Map,
}

impl Itch {
	fn get_installed_game(cave: &ItchDatabaseCave) -> Option<InstalledGame> {
		let verdict = cave.verdict.as_ref()?;
		let exe_path = verdict.base_path.join(&verdict.candidates.first()?.path);
		let mut game = InstalledGame::new(&exe_path, &cave.title, *Self::ID)?;
		if let Some(cover_url) = &cave.cover_url {
			game.set_thumbnail_url(cover_url);
		}
		game.set_provider_game_id(&cave.id.to_string());

		Some(game)
	}

	fn get_owned_game(row: &ItchDatabaseGame) -> OwnedGame {
		let mut game = OwnedGame::new(&row.id.to_string(), *Self::ID, &row.title);

		if let Some(thumbnail_url) = &row.cover_url {
			game.set_thumbnail_url(thumbnail_url);
		}
		if let Some(date_time) = row
			.published_at
			.as_ref()
			.and_then(|published_at| DateTime::parse_from_rfc3339(published_at).ok())
		{
			game.set_release_date(date_time.timestamp());
		}
		game.add_provider_command(
			ProviderCommandAction::ShowInLibrary,
			ProviderCommand::String(format!("itch://games/{}", row.id)),
		)
		.add_provider_command(
			ProviderCommandAction::Install,
			ProviderCommand::String(format!("itch://install?game_id={}", row.id)),
		)
		.guess_app_type();

		game
	}

	async fn get_remote_game(&self, db_item: ItchDatabaseGame) -> RemoteGame {
		let mut remote_game = RemoteGame::new(*Self::ID, &db_item.id.to_string());

		if let Some(cached_remote_game) = self.remote_game_cache.get(&remote_game.id) {
			return cached_remote_game.clone();
		}

		match pc_gaming_wiki::get_engine_from_game_title(&db_item.title).await {
			Ok(Some(engine)) => {
				remote_game.set_engine(engine);
			}
			Ok(None) => {}
			Err(_) => {
				remote_game.set_skip_cache(true);
			}
		}

		remote_game
	}
}

impl ProviderStatic for Itch {
	const ID: &'static ProviderId = &ProviderId::Itch;

	fn new() -> Result<Self>
	where
		Self: Sized,
	{
		Ok(Self {
			remote_game_cache: Self::try_get_remote_game_cache(),
		})
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

#[async_trait]
impl ProviderActions for Itch {
	async fn get_games<TInstalledCallback, TOwnedCallback, TRemoteCallback>(
		&self,
		installed_callback: TInstalledCallback,
		owned_callback: TOwnedCallback,
		remote_callback: TRemoteCallback,
	) -> Result
	where
		TInstalledCallback: Fn(InstalledGame) + Send + Sync,
		TOwnedCallback: Fn(OwnedGame) + Send + Sync,
		TRemoteCallback: Fn(RemoteGame) + Send + Sync,
	{
		let app_data_path = directories::BaseDirs::new()
			.ok_or_else(Error::AppDataNotFound)?
			.config_dir()
			.join("itch");

		let database = get_database(&app_data_path)?;
		let mut remote_game_futures = Vec::new();

		for db_entry in database.games {
			owned_callback(Self::get_owned_game(&db_entry));
			remote_game_futures.push(self.get_remote_game(db_entry.clone()));
		}

		for db_entry in database.caves {
			if let Some(installed_game) = Self::get_installed_game(&db_entry) {
				installed_callback(installed_game);
			}
		}

		// TODO: cache
		future::join_all(remote_game_futures)
			.await
			.iter()
			.for_each(|remote_game| {
				remote_callback(remote_game.clone());
				// self.remote_game_cache
				// 	.insert(remote_game.id.clone(), remote_game.clone());
			});

		Ok(())
	}
}

fn parse_verdict(json_option: &Option<String>) -> Option<ItchDatabaseVerdict> {
	let json = json_option.as_ref()?;
	match serde_json::from_str(json) {
		Ok(verdict) => Some(verdict),
		Err(err) => {
			error!("Failed to parse verdict from json `{json}`. Error: {err}");
			None
		}
	}
}

fn get_database(app_data_path: &Path) -> Result<ItchDatabase> {
	let db_path = app_data_path.join("db").join("butler.db");
	let connection = Connection::open_with_flags(db_path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;

	let mut caves_statement = connection.prepare(
		r"SELECT
			caves.game_id, caves.verdict, games.title, games.cover_url
    FROM
			caves
    JOIN
			games ON caves.game_id = games.id;
  ",
	)?;
	let cave_rows = caves_statement.query_map([], |row| {
		Ok(ItchDatabaseCave {
			id: row.get("game_id")?,
			title: row.get("title")?,
			verdict: parse_verdict(&row.get("verdict").ok()),
			cover_url: row.get("cover_url").ok(),
		})
	})?;

	let mut games_statement = connection.prepare(
		r"SELECT
			id, title, url, published_at, cover_url
		FROM
			'games'
		WHERE
			type='default' AND classification='game'
		",
	)?;
	let game_rows = games_statement.query_map([], |row| {
		Ok(ItchDatabaseGame {
			id: row.get(0)?,
			title: row.get(1)?,
			url: row.get(2).ok(),
			published_at: row.get(3).ok(),
			cover_url: row.get(4).ok(),
		})
	})?;

	Ok(ItchDatabase {
		games: game_rows
			.filter_map(|row| match row {
				Ok(game) => Some(game),
				Err(err) => {
					error!("Failed create itch game from database: {err}");
					None
				}
			})
			.collect(),
		caves: cave_rows
			.filter_map(|row| match row {
				Ok(cave) => Some(cave),
				Err(err) => {
					error!("Failed create itch game from database: {err}");
					None
				}
			})
			.collect(),
	})
}
