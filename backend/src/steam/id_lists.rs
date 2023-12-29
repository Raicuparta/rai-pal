use std::{
	collections::HashSet,
	future,
};

use crate::{
	game_engines::game_engine::GameEngineBrand,
	serializable_struct,
	Result,
};

const STEAM_APP_IDS_URL_BASE: &str =
	"https://raw.githubusercontent.com/Raicuparta/steam-app-ids-by-engine/main";

serializable_struct!(SteamGame {
	pub id: String,
	pub engine: GameEngineBrand,
});

async fn get_list(list_name: &str) -> Result<HashSet<String>> {
	Ok(
		reqwest::get(format!("{STEAM_APP_IDS_URL_BASE}/{list_name}"))
			.await?
			.text()
			.await?
			.split('\n')
			.map(std::string::ToString::to_string)
			.collect(),
	)
}

fn get_ids_data_list(ids: &HashSet<String>, engine: GameEngineBrand) -> Vec<SteamGame> {
	ids.iter()
		.map(|id| SteamGame {
			id: id.to_string(),
			engine,
		})
		.collect()
}

pub async fn get() -> Result<Vec<SteamGame>> {
	let (unity, unreal, godot) =
		future::join!(get_list("Unity"), get_list("Unreal"), get_list("Godot"),).await;

	Ok([
		get_ids_data_list(&unity.unwrap_or_default(), GameEngineBrand::Unity),
		get_ids_data_list(&unreal.unwrap_or_default(), GameEngineBrand::Unreal),
		get_ids_data_list(&godot.unwrap_or_default(), GameEngineBrand::Godot),
	]
	.concat())
}
