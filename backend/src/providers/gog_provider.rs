use std::path::PathBuf;

use async_trait::async_trait;
use log::error;
use rusqlite::{
	Connection,
	OpenFlags,
};
use serde::Deserialize;
use winreg::{
	enums::HKEY_LOCAL_MACHINE,
	RegKey,
};

use super::{
	provider::{
		self,
		ProviderId,
	},
	provider_command::ProviderCommand,
};
use crate::{
	game_engines::game_engine::GameEngine,
	installed_game::InstalledGame,
	owned_game::OwnedGame,
	paths,
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
	launcher_path: PathBuf,
}

impl ProviderStatic for Gog {
	const ID: &'static ProviderId = &ProviderId::Gog;

	fn new() -> Result<Self>
	where
		Self: Sized,
	{
		Ok(Self {
			engine_cache: Self::try_get_engine_cache(),
			database: get_database()?,
			launcher_path: get_launcher_path()?,
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
				let mut game = InstalledGame::new(
					db_entry.executable_path.as_ref()?,
					&db_entry.title,
					Self::ID.to_owned(),
				)?;

				game.set_start_command_path(
					&self.launcher_path,
					[
						"/command=runGame".to_string(),
						format!("/gameId={}", db_entry.id),
					]
					.to_vec(),
				);
				game.set_provider_game_id(&db_entry.id);

				if let Some(image_url) = &db_entry.image_url {
					game.set_thumbnail_url(image_url);
				}

				Some(game)
			})
			.collect())
	}

	async fn get_owned_games(&self) -> Result<Vec<OwnedGame>> {
		let owned_games = futures::future::join_all(self.database.iter().map(|db_entry| async {
			let mut game = OwnedGame::new(&db_entry.id, *Self::ID, &db_entry.title);

			game.set_show_library_command(ProviderCommand::Path(
				self.launcher_path.clone(),
				[
					"/command=launch".to_string(),
					format!("/gameId={}", db_entry.id),
				]
				.to_vec(),
			));

			if let Some(thumbnail_url) = db_entry.image_url.clone() {
				game.set_thumbnail_url(&thumbnail_url);
			}

			if let Some(release_date) = db_entry.release_date {
				game.set_release_date(release_date.into());
			}

			if let Some(engine) = get_engine(&db_entry.id, &self.engine_cache).await {
				game.set_engine(engine);
			}

			game
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
	let program_data = paths::try_get_program_data_path();
	let database_path = program_data.join("GOG.com/Galaxy/storage/galaxy-2.0.db");

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
			let id: i32 = row.get("id")?;
			let executable_path: Option<String> = try_get_string(row, "executablePath");
			let title = try_get_json::<GogDbEntryTitle>(row, "title").and_then(|title| title.title);
			let release_date =
				try_get_json::<GogDbEntryMeta>(row, "meta").and_then(|meta| meta.release_date);
			let image_url = try_get_json::<GogDbEntryImages>(row, "images")
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

fn try_parse_json<TData>(json: &str) -> Option<TData>
where
	TData: for<'a> Deserialize<'a>,
{
	match serde_json::from_str::<TData>(json) {
		Ok(data) => Some(data),
		Err(err) => {
			error!("Failed to parse GOG database json `{json}`. Error: {err}");
			None
		}
	}
}

fn try_get_string(row: &rusqlite::Row, id: &str) -> Option<String> {
	match row.get::<&str, Option<String>>(id) {
		Ok(value) => value,
		Err(err) => {
			error!("Failed to read GOG database value `{id}`. Error: {err}");
			None
		}
	}
}

fn try_get_json<TData>(row: &rusqlite::Row, id: &str) -> Option<TData>
where
	TData: for<'a> Deserialize<'a>,
{
	let json = try_get_string(row, id)?;
	try_parse_json(&json)
}

fn get_launcher_path() -> Result<PathBuf> {
	Ok(RegKey::predef(HKEY_LOCAL_MACHINE)
		.open_subkey(r"SOFTWARE\WOW6432Node\GOG.com\GalaxyClient\paths")
		.and_then(|reg_key| reg_key.get_value::<String, _>("client"))
		.map(PathBuf::from)?
		.join("GalaxyClient.exe"))
}
