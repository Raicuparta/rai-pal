use std::path::PathBuf;

use crate::{
	game_engines::{
		game_engine::GameEngineBrand,
		unity::UnityScriptingBackend,
	},
	serializable_struct,
	Result,
};

const URL_BASE: &str = "https://raw.githubusercontent.com/Raicuparta/rai-pal-db/main";

serializable_struct!(DatabaseEntry {
	pub id: String,
	pub title: String,
	pub author: String,
	pub source_code: String,
	pub description: String,
	pub latest_version: ModDownload,
	pub engine: Option<GameEngineBrand>,
	pub unity_backend: Option<UnityScriptingBackend>,
});

serializable_struct!(ModDatabase {
  pub mods: Vec<DatabaseEntry>,
});

serializable_struct!(ModDownload {
	pub id: String,
	pub url: String,
	pub root: Option<PathBuf>,
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
