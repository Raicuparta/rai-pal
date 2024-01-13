use std::{
	collections::HashSet,
	path::PathBuf,
};

use async_trait::async_trait;
use log::error;
use rusqlite::{
	Connection,
	OpenFlags,
};
use serde::Deserialize;

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
	executable_path: Option<PathBuf>,
}

pub struct Gog {
	engine_cache: provider::EngineCache,
	database: Vec<GogDbEntry>,
}

impl ProviderStatic for Gog {
	const ID: &'static ProviderId = &ProviderId::Gog;

	fn new() -> Result<Self>
	where
		Self: Sized,
	{
		let engine_cache = Self::try_get_engine_cache();
		let database = get_database()?;

		Ok(Self {
			engine_cache,
			database,
		})
	}
}

#[async_trait]
impl ProviderActions for Gog {
	fn get_installed_games(&self) -> Result<Vec<InstalledGame>> {
		Ok(self
			.database
			.iter()
			.filter_map(|db_entry| {
				InstalledGame::new(
					db_entry.executable_path.as_ref()?,
					&db_entry.title,
					Self::ID.to_owned(),
					None,
					None,
					db_entry.image_url.clone(),
				)
			})
			.collect())
	}

	async fn get_owned_games(&self) -> Result<Vec<OwnedGame>> {
		let owned_games = futures::future::join_all(self.database.iter().map(|db_entry| async {
			OwnedGame {
				// TODO should add a constructor to OwnedGame to avoid ID collisions and stuff.
				id: db_entry.id.clone(),
				provider_id: *Self::ID,
				name: db_entry.title.clone(),
				installed: db_entry.executable_path.is_some(),
				os_list: HashSet::default(),
				engine: get_engine(&db_entry.id, &self.engine_cache).await,
				release_date: db_entry.release_date.unwrap_or_default().into(),
				thumbnail_url: db_entry.image_url.clone().unwrap_or_default(),
				game_mode: None,
				uevr_score: None,
			}
		}))
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

serializable_struct!(GogDbEntryTitle { title: Option<String> });
serializable_struct!(GogDbEntryImages { square_icon: Option<String> });
serializable_struct!(GogDbEntryMeta { release_date: Option<i32> });

fn get_database() -> Result<Vec<GogDbEntry>> {
	// TODO get from registry or something.
	let database_path = PathBuf::from("C:\\ProgramData\\GOG.com\\Galaxy\\storage\\galaxy-2.0.db");

	let connection = Connection::open_with_flags(database_path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;

	let mut statement = connection.prepare(
		r"SELECT 
    P.id,
    MAX(CASE WHEN GP.gamePieceTypeId = 277 THEN GP.value END) AS title,
    MAX(CASE WHEN GP.gamePieceTypeId = 814 THEN GP.value END) AS images,
    MAX(CASE WHEN GP.gamePieceTypeId = 815 THEN GP.value END) AS meta,
		MAX(PTLP.executablePath) AS executablePath
FROM 
    Products P
JOIN 
    GamePieces GP ON P.id = substr(GP.releaseKey, 5) AND GP.releaseKey GLOB 'gog_*'
LEFT JOIN 
    PlayTasks PT ON GP.releaseKey = PT.gameReleaseKey
LEFT JOIN 
    PlayTaskLaunchParameters PTLP ON PT.id = PTLP.playTaskId
GROUP BY 
    P.id;",
	)?;

	let rows: Vec<GogDbEntry> = statement
		.query_map([], |row| {
			let id: i32 = row.get(0)?;
			let title_json: Option<String> = row.get(1).ok();
			let images_json: Option<String> = row.get(2).ok();
			let meta_json: Option<String> = row.get(3).ok();
			let executable_path: Option<String> = row.get(4).ok();

			let title =
				try_parse_json::<GogDbEntryTitle>(&title_json).and_then(|title| title.title);

			let release_date =
				try_parse_json::<GogDbEntryMeta>(&meta_json).and_then(|meta| meta.release_date);

			let image_url = try_parse_json::<GogDbEntryImages>(&images_json)
				.and_then(|images| images.square_icon);

			Ok(GogDbEntry {
				id: id.to_string(),
				title: title.unwrap_or_else(|| id.to_string()),
				image_url,
				release_date,
				executable_path: executable_path.map(PathBuf::from),
			})
		})?
		.filter_map(|row_result| match row_result {
			Ok(row) => Some(row),
			Err(err) => {
				error!("Failed to read GOG database row: {err}");
				None
			}
		})
		.collect();

	Ok(rows)
}

fn try_parse_json<'a, TData>(json_option: &'a Option<String>) -> Option<TData>
where
	TData: Deserialize<'a>,
{
	let json = json_option.as_deref()?;
	match serde_json::from_str::<TData>(json) {
		Ok(data) => Some(data),
		Err(err) => {
			error!("Failed to parse GOG database json `{json}`. Error: {err}");
			None
		}
	}
}
