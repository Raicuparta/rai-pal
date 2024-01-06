use std::{
	collections::{
		HashMap,
		HashSet,
	},
	future,
};

use log::error;

use crate::{
	game_engines::game_engine::GameEngineBrand,
	serializable_enum,
	serializable_struct,
	Result,
};

const STEAM_APP_IDS_URL_BASE: &str = "https://raicuparta.github.io/rai-pal-db/steam-ids";

serializable_enum!(UevrScore { A, B, C, D, E });

serializable_struct!(SteamGame {
	pub id: String,
	pub engine: GameEngineBrand,
	pub uevr_score: Option<UevrScore>,
});

async fn get_list(list_name: &str) -> HashSet<String> {
	match reqwest::get(format!("{STEAM_APP_IDS_URL_BASE}/{list_name}")).await {
		Ok(response) => match response.text().await {
			Ok(text) => text
				.split('\n')
				.map(|line| line.trim().to_string())
				.collect(),
			Err(err) => {
				error!("Failed to parse ids list {list_name}: {err}");
				HashSet::default()
			}
		},
		Err(err) => {
			error!("Failed to download ids list {list_name}: {err}");
			HashSet::default()
		}
	}
}

// TODO this should be a more generic thing where you can load arbitrary json databases,
// and show them as columns in Rai Pal.
// For now it's just a hardcoded uevr db.
async fn get_uevr_scores() -> HashMap<String, UevrScore> {
	match reqwest::get(format!("{STEAM_APP_IDS_URL_BASE}/uevr-scores.json")).await {
		Ok(response) => response
			.json::<HashMap<String, UevrScore>>()
			.await
			.unwrap_or_else(|err| {
				error!("Failed to parse uevr ids list: {err}");
				HashMap::default()
			}),
		Err(err) => {
			error!("Failed to download uevr ids list: {err}");
			HashMap::default()
		}
	}
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

	let uevr_scores = get_uevr_scores().await;

	let mut games = [
		get_ids_data_list(&unity, GameEngineBrand::Unity),
		get_ids_data_list(&unreal, GameEngineBrand::Unreal),
		get_ids_data_list(&godot, GameEngineBrand::Godot),
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
