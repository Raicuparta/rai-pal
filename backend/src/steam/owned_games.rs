use std::collections::HashSet;

use steamlocate::SteamDir;

use super::{
	appinfo::{self, SteamAppInfoFile},
	id_lists,
	thumbnail::get_steam_thumbnail,
};
use crate::{
	game::OperatingSystem, game_engines::game_engine::GameEngineBrand, serializable_struct, Result,
};

serializable_struct!(OwnedGame {
	id: String,
	name: String,
	installed: bool,
	os_list: HashSet<OperatingSystem>,
	engine: GameEngineBrand,
	release_date: i32,
	thumbnail_url: String,
});

async fn get_engine_games(
	engine: GameEngineBrand,
	steam_dir: &SteamDir,
	app_info: &SteamAppInfoFile,
) -> Result<Vec<OwnedGame>> {
	Ok(id_lists::get(&engine.to_string())
		.await?
		.iter()
		.filter_map(|id| {
			let id_number = id.parse::<u32>().ok()?;

			let app_info = app_info.apps.get(&id_number)?;

			let os_list: HashSet<_> = app_info
				.launch_options
				.iter()
				.filter_map(|launch| {
					launch
						.os_list
						.as_ref()
						.and_then(|os_list| match os_list.as_str() {
							"linux" => Some(OperatingSystem::Linux),
							"windows" => Some(OperatingSystem::Windows),
							_ => None,
						})
				})
				.collect();

			let installed = steam_dir
				.app(id_number)
				.map_or(false, |steam_app| steam_app.is_some());

			let release_date = app_info
				.original_release_date
				.or(app_info.steam_release_date)
				.unwrap_or_default();

			Some(OwnedGame {
				id: id.clone(),
				name: app_info.name.clone(),
				installed,
				os_list,
				engine,
				release_date,
				thumbnail_url: get_steam_thumbnail(id),
			})
		})
		.collect())
}

pub async fn get() -> Result<Vec<OwnedGame>> {
	let steam_dir = SteamDir::locate()?;
	let app_info = appinfo::read(steam_dir.path())?;

	Ok([
		get_engine_games(GameEngineBrand::Unity, &steam_dir, &app_info).await?,
		get_engine_games(GameEngineBrand::Unreal, &steam_dir, &app_info).await?,
		get_engine_games(GameEngineBrand::Godot, &steam_dir, &app_info).await?,
	]
	.concat())
}
