use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use lazy_regex::regex;
use rai_pal_proc_macros::serializable_struct;

use crate::game_engines::game_engine::{
	EngineBrand, EngineVersion, EngineVersionNumbers, GameEngine,
};
use crate::game_subscription::GameSubscription;
use crate::paths;
use crate::providers::provider::ProviderId;
use crate::result::Result;

const URL_BASE: &str = "https://raicuparta.github.io/rai-pal-db/game-db";

// The repository over at github.com/Raicuparta/rai-pal-db can have multiple versions of the database.
// This way we prevent old versions of Rai Pal from breaking unless we want them to.
// So when you need to change the database in a backwards-incompatible way,
// you would create a new folder in the database repository and change this number to match the folder.
const DATABASE_VERSION: i32 = 2;

#[serializable_struct]
struct GameDatabaseEngineVersion {
	brand: String,
	version: Option<String>,
}

#[serializable_struct]
struct GameDatabaseEntry {
	pub title: Option<String>,
	pub engines: Option<Vec<GameDatabaseEngineVersion>>,
	pub ids: Option<HashMap<ProviderId, Vec<String>>>,
	pub subscriptions: Option<HashSet<GameSubscription>>,
}

#[serializable_struct]
pub struct RemoteGame {
	pub title: Option<String>,
	pub engine: Option<GameEngine>,
	pub ids: HashMap<ProviderId, Vec<String>>,
	pub subscriptions: Option<HashSet<GameSubscription>>,
}

impl RemoteGame {
	//
	pub fn get_url(&self, provider_id: &ProviderId) -> Option<String> {
		self.ids
			.get(provider_id)
			.and_then(|ids| ids.first())
			.and_then(|id| match provider_id {
				ProviderId::Xbox => Some(format!("https://www.xbox.com/games/store/a/{id}")),
				ProviderId::Ubisoft => Some(format!("https://store.ubisoft.com/a/{id}.html")),
				_ => None,
			})
	}
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
// We just find some numbers in the string separated by something (usually periods).
pub fn parse_version(version: &str) -> Option<EngineVersion> {
	let version_numbers = regex!(r"\d+")
		.find_iter(version)
		.filter_map(|capture| capture.as_str().parse::<u32>().ok())
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

pub fn get_database_file_path() -> Result<PathBuf> {
	Ok(paths::app_data_path()?.join("remote.sqlite"))
}

pub async fn download_database() -> Result<PathBuf> {
	let url = format!("{URL_BASE}/{DATABASE_VERSION}/games.db");

	let response = reqwest::get(&url).await?;

	let file_path = get_database_file_path()?;

	fs::create_dir_all(paths::path_parent(&file_path)?)?;

	fs::write(&file_path, response.bytes().await?)?;

	Ok(file_path)
}
