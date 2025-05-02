#![cfg(target_os = "windows")]

use std::path::{Path, PathBuf};

use log::error;
use rai_pal_proc_macros::serializable_struct;
use rusqlite::{Connection, OpenFlags};
use serde::Deserialize;
use winreg::{RegKey, enums::HKEY_LOCAL_MACHINE};

use super::provider_command::{ProviderCommand, ProviderCommandAction};
use crate::{
	local_database::{DbMutex, InsertGame},
	game::DbGame,
	paths,
	providers::provider::{ProviderActions, ProviderId, ProviderStatic},
	result::Result,
};

#[derive(Clone)]
struct GogDbEntry {
	id: String,
	title: String,
	image_url: Option<String>,
	release_date: Option<i32>,
	executable_path: Option<PathBuf>,
}

#[derive(Clone)]
pub struct Gog {}

impl Gog {
	fn get_game(db_entry: &GogDbEntry, launcher_path: &Path) -> DbGame {
		let mut game = DbGame::new(*Self::ID, db_entry.id.clone(), db_entry.title.clone());

		game.add_provider_command(
			ProviderCommandAction::ShowInLibrary,
			ProviderCommand::Path(
				launcher_path.to_owned(),
				[
					"/command=launch".to_string(),
					format!("/gameId={}", db_entry.id),
				]
				.to_vec(),
			),
		);

		game.thumbnail_url = db_entry.image_url.clone();
		game.release_date = db_entry.release_date.map(|date| date.into());

		game
	}
}

impl ProviderStatic for Gog {
	const ID: &'static ProviderId = &ProviderId::Gog;

	fn new() -> Result<Self>
	where
		Self: Sized,
	{
		Ok(Self {})
	}
}

impl ProviderActions for Gog {
	async fn insert_games(&self, db: &DbMutex) -> Result {
		if let Some(database) = get_database()? {
			let launcher_path = get_launcher_path()?;

			for db_entry in database {
				let mut game = Self::get_game(&db_entry, &launcher_path);
				if let Some(executable_path) = db_entry.executable_path.as_ref() {
					game.set_executable(&executable_path);
					game.add_provider_command(
						ProviderCommandAction::StartViaProvider,
						ProviderCommand::Path(
							launcher_path.clone(),
							vec![
								"/command=runGame".to_string(),
								format!("/gameId={}", db_entry.id),
							],
						),
					);
				}

				db.insert_game(&game);
			}
		} else {
			log::info!(
				"GOG database file not found. Probably means user hasn't installed GOG Galaxy."
			);
		}
		Ok(())
	}
}

#[serializable_struct]
pub struct GogDbEntryTitle {
	title: Option<String>,
}
#[serializable_struct]
pub struct GogDbEntryImages {
	square_icon: Option<String>,
}
#[serializable_struct]
pub struct GogDbEntryMeta {
	release_date: Option<i32>,
}

fn get_database() -> Result<Option<Vec<GogDbEntry>>> {
	let program_data = paths::try_get_program_data_path();
	let database_path = program_data.join("GOG.com/Galaxy/storage/galaxy-2.0.db");

	if !database_path.is_file() {
		return Ok(None);
	}

	let connection = Connection::open_with_flags(database_path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;

	let mut statement = connection.prepare(
		r"SELECT 
	Builds.productId AS id,
	MAX(CASE WHEN GamePieceTypes.type = 'originalTitle' THEN GamePieces.value END) AS title,
	MAX(CASE WHEN GamePieceTypes.type = 'originalImages' THEN GamePieces.value END) AS images,
	MAX(CASE WHEN GamePieceTypes.type = 'originalMeta' THEN GamePieces.value END) AS meta,
	MAX(PlayTaskLaunchParameters.executablePath) AS executablePath
FROM 
	Builds
JOIN 
	GamePieces ON GamePieces.releaseKey = 'gog_' || Builds.productId
LEFT JOIN 
	PlayTasks ON GamePieces.releaseKey = PlayTasks.gameReleaseKey
LEFT JOIN 
	PlayTaskLaunchParameters ON PlayTasks.id = PlayTaskLaunchParameters.playTaskId
JOIN
	GamePieceTypes ON GamePieces.gamePieceTypeId = GamePieceTypes.id
GROUP BY 
	Builds.productId;",
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

	Ok(Some(rows))
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
