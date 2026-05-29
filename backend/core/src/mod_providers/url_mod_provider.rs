use std::{
	collections::HashMap,
	fs,
	path::PathBuf,
};

use rai_pal_proc_macros::serializable_struct;

use super::mod_provider::ModProvider;
use crate::{
	app_paths,
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

	async fn get_mods<TCallback>(&self, callback: &TCallback) -> Result
	where
		TCallback: Fn(HashMap<String, GameMod>) + Send + Sync,
	{
		if let Some(cached_database) = ModDatabase::get_from_cache()? {
			callback(cached_database.get_mod_map());
		}

		callback(ModDatabase::get_from_url(&self.url).await?.get_mod_map());

		Ok(())
	}
}

#[serializable_struct]
pub struct ModDatabase {
	pub mods: Vec<GameMod>,
}

impl ModDatabase {
	pub fn get_cache_path() -> Result<PathBuf> {
		Ok(app_paths::temp_dir("mod_database")?.join("mod_database.json"))
	}

	pub fn get_from_cache() -> Result<Option<Self>> {
		let cache_path = Self::get_cache_path()?;
		if !cache_path.exists() {
			return Ok(None);
		}

		let contents = fs::read_to_string(cache_path)?;
		let result = serde_json::from_str::<Self>(&contents)?;
		Ok(Some(result))
	}

	pub async fn get_from_url(url: &str) -> Result<Self> {
		let result = http::CLIENT.get(url).send().await?.json::<Self>().await?;

		fs::write(
			Self::get_cache_path()?,
			serde_json::to_string_pretty(&result)?,
		)?;

		Ok(result)
	}

	pub fn get_mod_map(&self) -> HashMap<String, GameMod> {
		self.mods
			.iter()
			.map(|game_mod| (game_mod.id.clone(), game_mod.clone()))
			.collect()
	}
}
