use std::collections::HashMap;

use super::mod_provider::ModProvider;
use crate::game_mods::game_mod::GameMod;
use crate::game_mods::mod_database::ModDatabase;
use crate::result::Result;

pub struct UrlModProvider {
	pub url: String,
}

impl UrlModProvider {
	pub async fn get_mods_async(&self) -> Result<HashMap<String, GameMod>> {
		let database = ModDatabase::get_from_url(&self.url).await?;

		let mut mods_map = HashMap::new();
		for game_mod in database.mods {
			mods_map.insert(game_mod.id.clone(), game_mod);
		}

		Ok(mods_map)
	}
}

impl ModProvider for UrlModProvider {
	fn get_mods(&self) -> Result<HashMap<String, GameMod>> {
		unreachable!("UrlModProvider requires async fetching; use get_mods_async instead")
	}
}
