use rai_pal_proc_macros::serializable_struct;

use crate::{
	game_mods::game_mod::GameMod,
	http,
	result::Result,
};

const URL_BASE: &str = "https://raicuparta.github.io/rai-pal-db/mod-db";

// The repository over at github.com/Raicuparta/rai-pal-db can have multiple versions of the database.
// This way we prevent old versions of Rai Pal from breaking unless we want them to.
// So when you need to change the database in a backwards-incompatible way,
// you would create a new folder in the database repository and change this number to match the folder.
const DATABASE_VERSION: i32 = 1;

#[serializable_struct]
pub struct ModDatabase {
	pub mods: Vec<GameMod>,
}

impl ModDatabase {
	pub async fn get() -> Result<Self> {
		Self::get_from_url(&format!("{URL_BASE}/{DATABASE_VERSION}/mods.json")).await
	}

	pub async fn get_from_url(url: &str) -> Result<Self> {
		Ok(http::CLIENT.get(url).send().await?.json::<Self>().await?)
	}
}
