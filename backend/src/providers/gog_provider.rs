use std::{
	collections::HashSet,
	path::PathBuf,
};

use async_trait::async_trait;
use rusqlite::{
	params_from_iter,
	Connection,
	OpenFlags,
	Row,
};

use super::provider::{
	self,
	ProviderId,
};
use crate::{
	game_engines::game_engine::GameEngine,
	installed_game::InstalledGame,
	owned_game::OwnedGame,
	pc_gaming_wiki,
	provider::{
		ProviderActions,
		ProviderStatic,
	},
	serializable_struct,
	Result,
};

struct GogDbEntry {
	id: String,
	title: String,
	image_url: Option<String>,
	release_date: Option<i32>,
}

pub struct Gog {
	engine_cache: provider::EngineCache,
}

impl ProviderStatic for Gog {
	const ID: &'static ProviderId = &ProviderId::Gog;

	fn new() -> Result<Self>
	where
		Self: Sized,
	{
		let engine_cache = Self::try_get_engine_cache();

		Ok(Self { engine_cache })
	}
}

#[async_trait]
impl ProviderActions for Gog {
	fn get_installed_games(&self) -> Result<Vec<InstalledGame>> {
		Ok(game_scanner::gog::games()
			.unwrap_or_default()
			.iter()
			.filter_map(|game| {
				InstalledGame::new(
					game.path.as_ref()?,
					&game.name,
					Self::ID.to_owned(),
					None,
					None,
					None,
				)
			})
			.collect())
	}

	async fn get_owned_games(&self) -> Result<Vec<OwnedGame>> {
		let owned_games = futures::future::join_all(get_database().unwrap_or_default().iter().map(
			|db_entry| async {
				OwnedGame {
					// TODO should add a constructor to OwnedGame to avoid ID collisions and stuff.
					id: db_entry.id.clone(),
					provider_id: *Self::ID,
					name: db_entry.title.clone(),
					installed: false, // TODO
					os_list: HashSet::default(),
					engine: get_engine(&db_entry.id, &self.engine_cache).await,
					release_date: db_entry.release_date.unwrap_or_default().into(),
					thumbnail_url: db_entry.image_url.clone().unwrap_or_default(),
					game_mode: None,
					uevr_score: None,
				}
			},
		))
		.await;

		Self::try_save_engine_cache(
			&owned_games
				.clone()
				.into_iter()
				.map(|owned_game| (owned_game.name.clone(), owned_game.engine))
				.collect(),
		);

		Ok(owned_games)
	}
}

async fn get_engine(gog_id: &str, cache: &provider::EngineCache) -> Option<GameEngine> {
	if let Some(cached_engine) = cache.get(gog_id) {
		return cached_engine.clone();
	}

	pc_gaming_wiki::get_engine(&format!("GOGcom_ID%20HOLDS%20%22{gog_id}%22")).await
}

serializable_struct!(GogDbEntryMeta { release_date: i32 });

fn get_database() -> Result<Vec<GogDbEntry>> {
	// TODO get from registry or something.
	let database_path = PathBuf::from("C:\\ProgramData\\GOG.com\\Galaxy\\storage\\galaxy-2.0.db");

	let connection = Connection::open_with_flags(database_path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;

	// TODO: Uh, this is showing me games I don't own, so I should get the manifest IDs from the other table first.
	let mut statement = connection.prepare(
		r"SELECT 
releaseKey,
MAX(CASE WHEN gamePieceTypeId = 277 THEN value END) AS title,
MAX(CASE WHEN gamePieceTypeId = 814 THEN value END) AS images,
MAX(CASE WHEN gamePieceTypeId = 815 THEN value END) AS meta
FROM 
GamePieces
WHERE
releaseKey GLOB 'gog_*'
GROUP BY 
releaseKey;",
	)?;

	let rows: Vec<GogDbEntry> = statement
		.query_map([], |row| {
			let id: String = row.get(0)?;
			let title_json: String = row.get(1)?;
			let images_json: Option<String> = row.get(2).ok();
			let meta_json: Option<String> = row.get(3).ok();

			let release_date = meta_json
				.and_then(|json| serde_json::from_str::<GogDbEntryMeta>(&json).ok())
				.map(|meta| meta.release_date);

			Ok(GogDbEntry {
				id: id.replace("gog_", ""),
				title: title_json,
				image_url: images_json,
				release_date,
			})
		})?
		.filter_map(|row| row.ok()) // TODO log errors
		.collect();

	Ok(rows)
}
