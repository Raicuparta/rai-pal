use std::string;

use steamlocate::SteamDir;

use super::appinfo;
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
	pub nsfw: bool,
});

pub async fn get_nsfw_ids() -> Result<Vec<String>> {
	Ok(reqwest::get(format!("{UNITY_STEAM_APP_IDS_URL}/NSFW"))
		.await?
		.text()
		.await?
		.split('\n')
		.map(string::ToString::to_string)
		.collect())
}

pub async fn get_ids_for_engine(
	engine: GameEngineBrand,
	nsfw_ids: &Option<Vec<String>>,
) -> Result<Vec<GameDatabaseEntry>> {
	Ok(reqwest::get(format!("{UNITY_STEAM_APP_IDS_URL}/{engine}"))
		.await?
		.text()
		.await?
		.split('\n')
		.map(|id| {
			let id_string = id.to_string();
			let nsfw = nsfw_ids
				.as_ref()
				.map_or(false, |ids| ids.contains(&id_string));
			GameDatabaseEntry {
				id: id_string,
				engine,
				nsfw,
			}
		})
		.collect())
}

pub async fn get_unowned_games() -> Result<Vec<GameDatabaseEntry>> {
	let nsfw_ids = Some(get_nsfw_ids().await?);

	let mut ids = get_ids_for_engine(GameEngineBrand::Unity, &nsfw_ids).await?;
	ids.append(&mut get_ids_for_engine(GameEngineBrand::Unreal, &nsfw_ids).await?);
	ids.append(&mut get_ids_for_engine(GameEngineBrand::Godot, &nsfw_ids).await?);

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
