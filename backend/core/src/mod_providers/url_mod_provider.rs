use rai_pal_proc_macros::serializable_struct;

use super::mod_provider::ModProvider;
use crate::{
	http,
	local_database::{
		game_database::DbMutex,
		mod_database::ModDatabase,
	},
	mods::game_mod::GameMod,
	result::Result,
};

const URL_BASE: &str = "https://raicuparta.github.io/rai-pal-db/mod-db";

// The repository over at github.com/Raicuparta/rai-pal-db can have multiple versions of the database.
// This way we prevent old versions of Rai Pal from breaking unless we want them to.
// So when you need to change the database in a backwards-incompatible way,
// you would create a new folder in the database repository and change this number to match the folder.
const DATABASE_VERSION: i32 = 1;

pub struct UrlModProvider {
	pub url: String,
}

impl ModProvider for UrlModProvider {
	fn default() -> Self {
		Self {
			url: format!("{URL_BASE}/{DATABASE_VERSION}/mods.json"),
		}
	}

	async fn insert_mods(&self, db: &DbMutex) -> Result {
		UrlModDatabase::get_from_url(&self.url)
			.await?
			.insert_mods(db);

		Ok(())
	}
}

#[serializable_struct]
pub struct UrlModDatabase {
	pub mods: Vec<GameMod>,
}

impl UrlModDatabase {
	pub async fn get_from_url(url: &str) -> Result<Self> {
		Ok(http::CLIENT.get(url).send().await?.json::<Self>().await?)
	}

	pub fn insert_mods(&self, db: &DbMutex) {
		for game_mod in &self.mods {
			db.insert_mod(game_mod);
		}
	}
}
