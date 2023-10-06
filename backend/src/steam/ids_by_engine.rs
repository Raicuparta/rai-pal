use steamlocate::SteamDir;

use super::appinfo::{
	self,
	SteamAppInfoFile,
};
use crate::{
	game_engines::game_engine::GameEngineBrand,
	serializable_struct,
	Result,
};

const UNITY_STEAM_APP_IDS_URL: &str =
	"https://raw.githubusercontent.com/Raicuparta/steam-unity-app-ids/main";

serializable_struct!(GameDatabaseEntry {
	pub id: String,
	pub engine: GameEngineBrand,
});

pub async fn get_ids_for_engine(engine: GameEngineBrand) -> Result<Vec<GameDatabaseEntry>> {
	Ok(reqwest::get(format!("{UNITY_STEAM_APP_IDS_URL}/{engine}"))
		.await?
		.text()
		.await?
		.split('\n')
		.map(|id| GameDatabaseEntry {
			id: id.to_string(),
			engine,
		})
		.collect())
}

pub async fn get_unowned_games() -> Result<Vec<GameDatabaseEntry>> {
	let mut ids = get_ids_for_engine(GameEngineBrand::Unity).await?;
	ids.append(&mut get_ids_for_engine(GameEngineBrand::Unreal).await?);
	ids.append(&mut get_ids_for_engine(GameEngineBrand::Godot).await?);

	let steam_dir = SteamDir::locate()?;
	let app_info = appinfo::read(steam_dir.path())?;

	Ok(ids
		.into_iter()
		.filter(|entry| {
			if let Ok(id) = entry.id.parse::<u32>() {
				return app_info.apps.get(&id).is_none();
			}
			false
		})
		.collect())
}
