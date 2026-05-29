use std::collections::HashMap;

use rai_pal_proc_macros::serializable_struct;

use super::mod_provider::ModProvider;
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

pub struct UrlModProvider {
	pub url: String,
}

impl ModProvider for UrlModProvider {
	fn default() -> Self {
		Self {
			url: format!("{URL_BASE}/{DATABASE_VERSION}/mods.json"),
		}
	}

	async fn get_mods(&self) -> Result<HashMap<String, GameMod>> {
		let database = ModDatabase::get_from_url(&self.url).await?;

		let mut mods_map = HashMap::new();
		for game_mod in database.mods {
			mods_map.insert(game_mod.id.clone(), game_mod);
		}

		Ok(mods_map)
	}
}

#[serializable_struct]
pub struct ModDatabase {
	pub mods: Vec<GameMod>,
}

impl ModDatabase {
	pub async fn get_from_url(url: &str) -> Result<Self> {
		Ok(http::CLIENT.get(url).send().await?.json::<Self>().await?)
	}
}
