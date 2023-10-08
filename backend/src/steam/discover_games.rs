use std::collections::{HashMap, HashSet};

use steamlocate::SteamDir;

use super::{
	appinfo::{self, SteamAppInfo},
	id_lists::{self},
};
use crate::{game_engines::game_engine::GameEngineBrand, serializable_struct, Result};

serializable_struct!(DiscoverGame {
	pub id: String,
	pub engine: GameEngineBrand,
	pub nsfw: bool,
});

async fn get_discover_games(
	engine: GameEngineBrand,
	owned_apps: &HashMap<u32, SteamAppInfo>,
	nsfw_ids: &HashSet<String>,
) -> Result<Vec<DiscoverGame>> {
	Ok(id_lists::get(&GameEngineBrand::Unity.to_string())
		.await?
		.iter()
		.filter_map(|id| {
			if let Ok(id_number) = id.parse::<u32>() {
				if owned_apps.contains_key(&id_number) {
					return None;
				}
			} else {
				return None;
			}

			Some(DiscoverGame {
				id: id.to_string(),
				engine,
				nsfw: nsfw_ids.contains(id),
			})
		})
		.collect())
}

pub async fn get() -> Result<Vec<DiscoverGame>> {
	let steam_dir = SteamDir::locate()?;
	let owned_apps = appinfo::read(steam_dir.path())?.apps;
	let nsfw_ids = id_lists::get("NSFW").await?;

	Ok([
		get_discover_games(GameEngineBrand::Unity, &owned_apps, &nsfw_ids).await?,
		get_discover_games(GameEngineBrand::Unreal, &owned_apps, &nsfw_ids).await?,
		get_discover_games(GameEngineBrand::Godot, &owned_apps, &nsfw_ids).await?,
	]
	.concat())
}
