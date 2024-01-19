use std::path::{
	Path,
	PathBuf,
};

use async_trait::async_trait;
use log::error;
use rusqlite::{
	Connection,
	OpenFlags,
};

use super::provider_command::{
	ProviderCommand,
	ProviderCommandAction,
};
use crate::{
	installed_game::{
		self,
		InstalledGame,
	},
	owned_game::OwnedGame,
	paths,
	pc_gaming_wiki,
	provider::{
		ProviderActions,
		ProviderId,
		ProviderStatic,
	},
	remote_game::{
		self,
		RemoteGame,
	},
	serializable_struct,
	Error,
	Result,
};

#[derive(Clone)]
pub struct Itch {
	database: ItchDatabase,
	app_data_path: PathBuf,
	remote_game_cache: remote_game::Map,
}

impl ProviderStatic for Itch {
	const ID: &'static ProviderId = &ProviderId::Itch;

	fn new() -> Result<Self>
	where
		Self: Sized,
	{
		let app_data_path = directories::BaseDirs::new()
			.ok_or_else(Error::AppDataNotFound)?
			.config_dir()
			.join("itch");

		Ok(Self {
			database: get_database(&app_data_path)?,
			remote_game_cache: Self::try_get_remote_game_cache(),
			app_data_path,
		})
	}
}

serializable_struct!(ItchDatabaseGame {
	id: i32,
	title: String,
	url: Option<String>,
	published_at: Option<String>,
  cover_url: Option<String>,
});

serializable_struct!(ItchDatabaseCave {
	id: i32,
	verdict: ItchDatabaseVerdict,
	title: String,
  cover_url: Option<String>,
});

serializable_struct!(ItchDatabaseVerdict {
	base_path: PathBuf,
  candidates: Vec<ItchDatabaseCandidate>
});

serializable_struct!(ItchDatabaseCandidate { path: PathBuf });

serializable_struct!(ItchDatabase {
	games: Vec<ItchDatabaseGame>,
  caves: Vec<ItchDatabaseCave>,
});

#[async_trait]
impl ProviderActions for Itch {
	fn get_installed_games(&self) -> Result<Vec<InstalledGame>> {
		Ok(self
			.database
			.caves
			.iter()
			.filter_map(|cave| {
				let exe_path = cave
					.verdict
					.base_path
					.join(&cave.verdict.candidates.first()?.path);
				let mut game = InstalledGame::new(&exe_path, &cave.title, *Self::ID)?;
				if let Some(cover_url) = &cave.cover_url {
					game.set_thumbnail_url(cover_url);
				}

				Some(game)
			})
			.collect())
	}

	fn get_owned_games(&self) -> Result<Vec<OwnedGame>> {
		Ok(self
			.database
			.games
			.iter()
			.map(|row| {
				let mut game = OwnedGame::new(&row.id.to_string(), *Self::ID, &row.title);

				if let Some(thumbnail_url) = &row.cover_url {
					game.set_thumbnail_url(thumbnail_url);
				}
				game.add_provider_command(
					ProviderCommandAction::ShowInLibrary,
					ProviderCommand::String(format!("itch://games/{}", row.id)),
				)
				.add_provider_command(
					ProviderCommandAction::Install,
					ProviderCommand::String(format!("itch://install?game_id={}", row.id)),
				);

				game
			})
			.collect())
	}

	async fn get_remote_games(&self) -> Result<Vec<RemoteGame>> {
		let remote_games: Vec<RemoteGame> =
			futures::future::join_all(self.database.games.iter().map(|db_item| async {
				let mut remote_game = RemoteGame::new(*Self::ID, &db_item.id.to_string());

				if let Some(cached_remote_game) = self.remote_game_cache.get(&remote_game.id) {
					return cached_remote_game.clone();
				}

				if let Some(engine) =
					pc_gaming_wiki::get_engine_from_game_title(&db_item.title).await
				{
					remote_game.set_engine(engine);
				}

				remote_game
			}))
			.await;

		Self::try_save_remote_game_cache(
			&remote_games
				.clone()
				.into_iter()
				.map(|remote_game| (remote_game.id.clone(), remote_game))
				.collect(),
		);

		Ok(remote_games)
	}
}

fn get_database(app_data_path: &Path) -> Result<ItchDatabase> {
	let db_path = app_data_path.join("db").join("butler.db");
	let connection = Connection::open_with_flags(db_path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;

	let mut caves_statement = connection.prepare(
		r"
    SELECT caves.game_id, caves.verdict, games.title, games.cover_url
    FROM caves
    JOIN games ON caves.game_id = games.id;
  ",
	)?;
	let cave_rows = caves_statement.query_map([], |row| {
		// TODO maybe not great that I'm swallowing errors here.
		let verdict_json: String = row.get("verdict")?;
		Ok(ItchDatabaseCave {
			id: row.get("game_id")?,
			title: row.get("title")?,
			cover_url: row.get("cover_url").ok(),
			verdict: serde_json::from_str(&verdict_json).unwrap(), // TODO no unwrap
		})
	})?; // TODO prevent everything crashing if one row fails.

	let mut games_statement = connection.prepare("SELECT id, title, url, published_at, cover_url FROM 'games' WHERE type='default' AND classification='game'")?;
	let game_rows = games_statement.query_map([], |row| {
		Ok(ItchDatabaseGame {
			id: row.get(0)?,
			title: row.get(1)?,
			// TODO maybe not great that I'm swallowing errors here.
			url: row.get(2).ok(),
			published_at: row.get(3).ok(),
			cover_url: row.get(4).ok(),
		})
	})?; // TODO prevent everything crashing if one row fails.

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
