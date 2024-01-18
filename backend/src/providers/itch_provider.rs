use std::path::PathBuf;

use async_trait::async_trait;
use log::error;
use rusqlite::{
	Connection,
	OpenFlags,
};

use super::provider::ProviderId;
use crate::{
	installed_game::InstalledGame,
	owned_game::OwnedGame,
	pc_gaming_wiki,
	provider::{
		ProviderActions,
		ProviderStatic,
	},
	remote_game::{
		self,
		RemoteGame,
	},
	serializable_struct,
	Result,
};

#[derive(Clone)]
pub struct Itch {
	database: Vec<ItchDatabaseItem>,
	remote_game_cache: remote_game::Map,
}

impl ProviderStatic for Itch {
	const ID: &'static ProviderId = &ProviderId::Itch;

	fn new() -> Result<Self>
	where
		Self: Sized,
	{
		Ok(Self {
			database: get_database()?,
			remote_game_cache: Self::try_get_remote_game_cache(),
		})
	}
}

serializable_struct!(ItchDatabaseItem {
	id: i32,
	title: String,
	url: Option<String>,
	published_at: Option<String>,
  cover_url: Option<String>,
});

#[async_trait]
impl ProviderActions for Itch {
	fn get_installed_games(&self) -> Result<Vec<InstalledGame>> {
		Ok(Vec::default())
	}

	fn get_owned_games(&self) -> Result<Vec<OwnedGame>> {
		Ok(self
			.database
			.iter()
			.map(|row| {
				let mut game = OwnedGame::new(&row.id.to_string(), *Self::ID, &row.title);

				if let Some(thumbnail_url) = &row.cover_url {
					game.set_thumbnail_url(thumbnail_url);
				}

				game
			})
			.collect())
	}

	async fn get_remote_games(&self) -> Result<Vec<RemoteGame>> {
		let remote_games: Vec<RemoteGame> =
			futures::future::join_all(self.database.iter().map(|db_item| async {
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

fn get_database() -> Result<Vec<ItchDatabaseItem>> {
	let db_path = PathBuf::from(r"C:\Users\rai\AppData\Roaming\itch\db\butler.db");
	let connection = Connection::open_with_flags(db_path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
	let mut statement = connection.prepare("SELECT id, title, url, published_at, cover_url FROM 'games' WHERE type='default' AND classification='game'")?;

	let rows = statement.query_map([], |row| {
		Ok(ItchDatabaseItem {
			id: row.get(0)?,
			title: row.get(1)?,
			// TODO maybe not great that I'm swallowing errors here.
			url: row.get(2).ok(),
			published_at: row.get(3).ok(),
			cover_url: row.get(4).ok(),
		})
	})?;

	Ok(rows
		.filter_map(|row| match row {
			Ok(game) => Some(game),
			Err(err) => {
				error!("Failed create itch game from database: {err}");
				None
			}
		})
		.collect())
}
