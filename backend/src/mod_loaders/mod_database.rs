use std::collections::HashMap;

use crate::{
	game_mod::CommonModData,
	serializable_struct,
	Result,
};

const URL_BASE: &str = "https://raw.githubusercontent.com/Raicuparta/rai-pal-db/main";

serializable_struct!(ModDatabase {
  pub mods: HashMap<String, RemoteMod>,
});

serializable_struct!(RemoteModData {
  pub title: String,
  pub author: String,
  pub source_code: String,
  pub description: String,
  pub downloads: Vec<ModDownload>,
});

serializable_struct!(RemoteMod {
	pub common: CommonModData,
	pub data: RemoteModData,
});

serializable_struct!(ModDownload {
	pub version: String,
	pub url: String,
});

pub async fn get(mod_loader_id: &str) -> Result<ModDatabase> {
	let random = rand::random::<u32>();

	Ok(reqwest::get(format!(
		"{URL_BASE}/{mod_loader_id}.json?cache_avoider={random}"
	))
	.await?
	.json::<ModDatabase>()
	.await?)
}
