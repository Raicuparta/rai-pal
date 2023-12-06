use crate::{
	game_engines::{
		game_engine::GameEngineBrand,
		unity::UnityScriptingBackend,
	},
	serializable_struct,
	Result,
};

const URL_BASE: &str = "https://raw.githubusercontent.com/Raicuparta/rai-pal-db/main";

serializable_struct!(ModDatabase {
  pub mods: Vec<DatabaseMod>,
});

serializable_struct!(DatabaseMod {
  pub id: String,
  pub title: String,
  pub author: String,
  pub source_code: String,
  pub description: String,
  pub engine: GameEngineBrand,
  pub unity_backend: Option<UnityScriptingBackend>,
  pub downloads: Vec<ModDownload>,
});

serializable_struct!(ModDownload {
	version: String,
	url: String,
});

pub async fn get(mod_loader_id: &str) -> Result<ModDatabase> {
	Ok(reqwest::get(format!("{URL_BASE}/{mod_loader_id}.json"))
		.await?
		.json::<ModDatabase>()
		.await?)
}
