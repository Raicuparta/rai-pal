use std::collections::HashMap;

use crate::{
	game_engines::game_engine::GameEngineBrand,
	serializable_struct,
	Result,
};

const UNITY_STEAM_APP_IDS_URL: &str =
	"https://raw.githubusercontent.com/Raicuparta/steam-unity-app-ids/main";

serializable_struct!(GameDatabaseEntry {
	id: String,
	engine: GameEngineBrand,
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

pub async fn get_ids() -> Result<Vec<GameDatabaseEntry>> {
	let mut ids = get_ids_for_engine(GameEngineBrand::Unity).await?;
	ids.append(&mut get_ids_for_engine(GameEngineBrand::Unreal).await?);
	ids.append(&mut get_ids_for_engine(GameEngineBrand::Godot).await?);

	Ok(ids)
}
