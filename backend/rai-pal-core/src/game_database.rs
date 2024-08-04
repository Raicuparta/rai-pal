use rai_pal_proc_macros::serializable_struct;

use crate::game_engines::game_engine::EngineBrand;
use crate::result::Result;

const URL_BASE: &str = "https://raicuparta.github.io/rai-pal-db/game-db";

// The repository over at github.com/Raicuparta/rai-pal-db can have multiple versions of the database.
// This way we prevent old versions of Rai Pal from breaking unless we want them to.
// So when you need to change the database in a backwards-incompatible way,
// you would create a new folder in the database repository and change this number to match the folder.
const DATABASE_VERSION: i32 = 0;

pub type GameDatabase = Vec<DatabaseEntry>;

#[serializable_struct]
pub struct DatabaseEntry {
	pub title: String,
	pub engine: Option<EngineBrand>,
	pub engine_versions: Option<Vec<String>>,
	pub steam_ids: Option<Vec<String>>,
	pub gog_ids: Option<Vec<String>>,
	pub epic_ids: Option<Vec<String>>,
}

pub async fn get() -> Result<GameDatabase> {
	let url = format!("{URL_BASE}/{DATABASE_VERSION}/games.json");
	let response = reqwest::get(&url).await?;
	let mods: Vec<DatabaseEntry> = response.json().await?;
	Ok(mods)
}
