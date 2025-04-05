#![cfg(target_os = "windows")]

use std::path::{Path, PathBuf};

use log::error;
use rai_pal_proc_macros::serializable_struct;
use serde::Deserialize;
use sqlx::{Row, sqlite::SqlitePoolOptions};
use winreg::{RegKey, enums::HKEY_LOCAL_MACHINE};

use super::provider_command::{ProviderCommand, ProviderCommandAction};
use crate::{
	game::{DbGame, GameId},
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
	fn get_installed_game(db_entry: &GogDbEntry, launcher_path: &Path) -> Option<DbGame> {
		// let mut game = InstalledGame::new(db_entry.executable_path.as_ref()?)?;

		// game.set_start_command_path(
		// 	launcher_path,
		// 	[
		// 		"/command=runGame".to_string(),
		// 		format!("/gameId={}", db_entry.id),
		// 	]
		// 	.to_vec(),
		// );

		// Some(game)
		None
	}

	// fn get_game(db_entry: &GogDbEntry, launcher_path: &Path) -> DbGame {
	// 	let mut game = Game::new(
	// 		GameId {
	// 			game_id: db_entry.id.clone(),
	// 			provider_id: *Self::ID,
	// 		},
	// 		&db_entry.title,
	// 	);

	// 	game.add_provider_command(
	// 		ProviderCommandAction::ShowInLibrary,
	// 		ProviderCommand::Path(
	// 			launcher_path.to_owned(),
	// 			[
	// 				"/command=launch".to_string(),
	// 				format!("/gameId={}", db_entry.id),
	// 			]
	// 			.to_vec(),
	// 		),
	// 	);

	// 	if let Some(thumbnail_url) = db_entry.image_url.clone() {
	// 		game.set_thumbnail_url(&thumbnail_url);
	// 	}

	// 	if let Some(release_date) = db_entry.release_date {
	// 		game.set_release_date(release_date.into());
	// 	}

	// 	game
	// }
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
	async fn insert_games(&self, pool: &sqlx::Pool<sqlx::Sqlite>) -> Result {
		Ok(())
	}

	async fn get_games<TCallback>(&self, mut callback: TCallback) -> Result
	where
		TCallback: FnMut(DbGame) + Send + Sync,
	{
		// if let Some(database) = get_database().await? {
		// 	let launcher_path = get_launcher_path()?;

		// 	for db_entry in database {
		// 		let mut game = Self::get_game(&db_entry, &launcher_path);
		// 		if let Some(installed_game) = Self::get_installed_game(&db_entry, &launcher_path) {
		// 			if let Some(start_command) = &installed_game.start_command {
		// 				game.add_provider_command(
		// 					ProviderCommandAction::StartViaProvider,
		// 					start_command.clone(),
		// 				);
		// 			}

		// 			game.add_provider_command(
		// 				ProviderCommandAction::StartViaExe,
		// 				ProviderCommand::Path(
		// 					installed_game.executable.path.clone(),
		// 					Vec::default(),
		// 				),
		// 			);

		// 			game.installed_game = Some(installed_game);
		// 		}

		// 		callback(game);
		// 	}
		// } else {
		// 	log::info!(
		// 		"GOG database file not found. Probably means user hasn't installed GOG Galaxy."
		// 	);
		// }

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

async fn get_database() -> Result<Option<Vec<GogDbEntry>>> {
	let program_data = paths::try_get_program_data_path();
	let database_path = program_data.join("GOG.com/Galaxy/storage/galaxy-2.0.db");

	if !database_path.is_file() {
		return Ok(None);
	}

	let pool = SqlitePoolOptions::new()
		.max_connections(5)
		.connect(&format!("sqlite://{}", database_path.display()))
		.await?;

	let rows = sqlx::query(
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
	)
	.fetch_all(&pool)
	.await?;

	let entries: Vec<GogDbEntry> = rows
		.into_iter()
		.filter_map(|row| {
			let id: i32 = row.get("id");
			let executable_path: Option<String> = row.try_get("executablePath").ok();
			let title = try_parse_json::<GogDbEntryTitle>(&row.try_get::<String, _>("title").ok()?)
				.and_then(|title| title.title);
			let release_date =
				try_parse_json::<GogDbEntryMeta>(&row.try_get::<String, _>("meta").ok()?)
					.and_then(|meta| meta.release_date);
			let image_url =
				try_parse_json::<GogDbEntryImages>(&row.try_get::<String, _>("images").ok()?)
					.and_then(|images| images.square_icon);

			Some(GogDbEntry {
				id: id.to_string(),
				title: title.unwrap_or_else(|| id.to_string()),
				image_url,
				release_date,
				executable_path: executable_path.map(PathBuf::from),
			})
		})
		.collect();

	Ok(Some(entries))
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

fn get_launcher_path() -> Result<PathBuf> {
	Ok(RegKey::predef(HKEY_LOCAL_MACHINE)
		.open_subkey(r"SOFTWARE\WOW6432Node\GOG.com\GalaxyClient\paths")
		.and_then(|reg_key| reg_key.get_value::<String, _>("client"))
		.map(PathBuf::from)?
		.join("GalaxyClient.exe"))
}
