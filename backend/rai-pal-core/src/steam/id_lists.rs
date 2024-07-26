use std::collections::{HashMap, HashSet};

use log::error;
use rai_pal_proc_macros::serializable_struct;

use crate::{game_engines::game_engine::EngineBrand, result::Result};

const STEAM_APP_IDS_URL_BASE: &str = "https://raicuparta.github.io/rai-pal-db/steam-ids";

#[serializable_struct]
pub struct SteamGame {
	pub id: String,
	pub engine: EngineBrand,
}

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

fn get_ids_data_list(ids: &HashSet<String>, engine: EngineBrand) -> Vec<SteamGame> {
	ids.iter()
		.map(|id| SteamGame {
			id: id.to_string(),
			engine,
		})
		.collect()
}

pub async fn get() -> Result<HashMap<String, SteamGame>> {
	let (unity, unreal, godot, game_maker) = tokio::join!(
		get_list("Unity"),
		get_list("Unreal"),
		get_list("Godot"),
		get_list("GameMaker")
	);

	let games = [
		get_ids_data_list(&unity, EngineBrand::Unity),
		get_ids_data_list(&unreal, EngineBrand::Unreal),
		get_ids_data_list(&godot, EngineBrand::Godot),
		get_ids_data_list(&game_maker, EngineBrand::GameMaker),
	]
	.concat();

	Ok(games
		.into_iter()
		.map(|game| (game.id.clone(), game))
		.collect())
}
