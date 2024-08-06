use rai_pal_proc_macros::serializable_struct;

use crate::game_engines::game_engine::{
	EngineBrand, EngineVersion, EngineVersionNumbers, GameEngine,
};
use crate::result::Result;

const URL_BASE: &str = "https://raicuparta.github.io/rai-pal-db/game-db";

// The repository over at github.com/Raicuparta/rai-pal-db can have multiple versions of the database.
// This way we prevent old versions of Rai Pal from breaking unless we want them to.
// So when you need to change the database in a backwards-incompatible way,
// you would create a new folder in the database repository and change this number to match the folder.
const DATABASE_VERSION: i32 = 0;

#[serializable_struct]
struct GameDatabaseEngineVersion {
	brand: String,
	version: Option<String>,
}

#[serializable_struct]
struct GameDatabaseEntry {
	pub title: Option<String>,
	pub engines: Option<Vec<GameDatabaseEngineVersion>>,
	pub steam_ids: Option<Vec<String>>,
	pub gog_ids: Option<Vec<String>>,
	pub epic_ids: Option<Vec<String>>,
}

#[serializable_struct]
pub struct RemoteGame {
	pub title: Option<String>,
	pub engines: Option<Vec<GameEngine>>,
	pub steam_ids: Option<Vec<String>>,
	pub gog_ids: Option<Vec<String>>,
	pub epic_ids: Option<Vec<String>>,
}

fn engine_brand_from_string(brand: &str) -> Option<EngineBrand> {
	let brand_lower = brand.to_lowercase();
	if brand_lower.contains("unity") {
		Some(EngineBrand::Unity)
	} else if brand_lower.contains("unreal") {
		Some(EngineBrand::Unreal)
	} else if brand_lower.contains("godot") {
		Some(EngineBrand::Godot)
	} else if brand_lower.contains("gamemaker") {
		Some(EngineBrand::GameMaker)
	} else {
		None
	}
}

// Version strings in PCGamingWiki can be all weird, so parsing is pretty lax here.
// We just split by dots and take the first numbers we find.
fn parse_version(version: &str) -> Option<EngineVersion> {
	let version_numbers = version
		.split('.')
		.filter_map(|part| part.parse::<u32>().ok())
		.take(2)
		.collect::<Vec<_>>();

	version_numbers.first().map(|major| EngineVersion {
		display: version_numbers
			.iter()
			.map(u32::to_string)
			.collect::<Vec<_>>()
			.join("."),
		numbers: EngineVersionNumbers {
			major: *major,
			minor: version_numbers.get(1).copied(),
			patch: version_numbers.get(2).copied(),
		},
		suffix: None,
	})
}

pub async fn get() -> Result<Vec<RemoteGame>> {
	let url = format!("{URL_BASE}/{DATABASE_VERSION}/games.json");
	let response = reqwest::get(&url).await?;

	let game_database: Vec<GameDatabaseEntry> = response.json().await?;

	let games = game_database
		.into_iter()
		.map(|entry| RemoteGame {
			title: entry.title,
			engines: entry.engines.map(|engines| {
				engines
					.into_iter()
					.filter_map(|engine| {
						Some(GameEngine {
							brand: engine_brand_from_string(&engine.brand)?,
							version: engine
								.version
								.and_then(|version| parse_version(&version))
								// If we can't parse the version or it wasn't provided, we can check if there's a number in the actual engine name.
								// This is common for Unreal Engine, since it usually shows up as "Unreal Engine 4" or similar.
								.or_else(|| parse_version(&engine.brand)),
						})
					})
					.collect()
			}),
			steam_ids: entry.steam_ids,
			gog_ids: entry.gog_ids,
			epic_ids: entry.epic_ids,
		})
		.collect();

	Ok(games)
}
