use std::{
	collections::{
		HashMap,
		HashSet,
	},
	future,
};

use crate::{
	game_engines::game_engine::GameEngineBrand,
	serializable_enum,
	serializable_struct,
	Result,
};

const STEAM_APP_IDS_URL_BASE: &str =
	"https://raw.githubusercontent.com/Raicuparta/steam-app-ids-by-engine/main";

serializable_enum!(UevrScore { A, B, C, D, E });

serializable_struct!(SteamGame {
	pub id: String,
	pub engine: GameEngineBrand,
	pub uevr_score: Option<UevrScore>,
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

// TODO this should be a more generic thing where you can load arbitrary json databases,
// and show them as columns in Rai Pal.
// For now it's just a hardcoded uevr db.
async fn get_uevr_scores() -> Result<HashMap<String, UevrScore>> {
	Ok(
		reqwest::get(format!("{STEAM_APP_IDS_URL_BASE}/uevr-scores.json"))
			.await?
			.json::<HashMap<String, UevrScore>>()
			.await?,
	)
}

fn get_ids_data_list(ids: &HashSet<String>, engine: GameEngineBrand) -> Vec<SteamGame> {
	ids.iter()
		.map(|id| SteamGame {
			id: id.to_string(),
			engine,
			uevr_score: None,
		})
		.collect()
}

pub async fn get() -> Result<HashMap<String, SteamGame>> {
	let (unity, unreal, godot) =
		future::join!(get_list("Unity"), get_list("Unreal"), get_list("Godot"),).await;

	let uevr_scores = get_uevr_scores().await.unwrap_or_default();

	let mut games = [
		get_ids_data_list(&unity.unwrap_or_default(), GameEngineBrand::Unity),
		get_ids_data_list(&unreal.unwrap_or_default(), GameEngineBrand::Unreal),
		get_ids_data_list(&godot.unwrap_or_default(), GameEngineBrand::Godot),
	]
	.concat();

	for game in &mut games {
		game.uevr_score = uevr_scores.get(&game.id).copied();
	}

	Ok(games
		.into_iter()
		.map(|game| (game.id.clone(), game))
		.collect())
}
