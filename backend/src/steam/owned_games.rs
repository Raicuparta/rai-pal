use std::{
	collections::HashSet,
	fs,
};

use lazy_regex::{
	BytesRegex,
	Regex,
};
use steamlocate::SteamDir;

use super::{
	appinfo::SteamAppInfoFile,
	id_lists,
	thumbnail::get_steam_thumbnail,
};
use crate::{
	game_engines::game_engine::GameEngineBrand,
	game_executable::OperatingSystem,
	serializable_struct,
	Result,
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

pub async fn get(steam_dir: &SteamDir, app_info_file: &SteamAppInfoFile) -> Result<Vec<OwnedGame>> {
	Ok(id_lists::get()
		.await?
		.iter()
		.filter_map(|steam_id_data| {
			let id_number = steam_id_data.id.parse::<u32>().ok()?;

			let app_info = app_info_file.apps.get(&id_number)?;

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

			// Games in appinfo.vdf aren't necessarily owned.
			// Most of them are, but there are also a bunch of other games that Steam needs to reference for one reason or another.
			// assets.vdf is another cache file, and from my (not very extensive) tests, it to really only include owned files.
			// Free games are some times not there though, so I'm presuming that any free game found in appinfo.vdf is owned.
			// appinfo.vdf is also still needed since most of the game data we want is there, so we can't just read everything from assets.vdf.
			let owned = app_info.is_free
				|| fs::read(steam_dir.path().join("appcache/librarycache/assets.vdf")).map_or(
					false,
					|assets_cache_bytes| {
						// Would be smarter to actually parse assets.vdf and extract all the ids,
						// but I didn't feel like figuring out how to parse another binary vdf.
						// Maybe later. But most likely never.
						BytesRegex::new(&format!("{}", steam_id_data.id))
							.map_or(false, |regex| regex.is_match(&assets_cache_bytes))
					},
				);

			if !owned {
				return None;
			}

			let installed = steam_dir
				.app(id_number)
				.map_or(false, |steam_app| steam_app.is_some());

			let release_date = app_info
				.original_release_date
				.or(app_info.steam_release_date)
				.unwrap_or_default();

			Some(OwnedGame {
				id: steam_id_data.id.clone(),
				name: app_info.name.clone(),
				installed,
				os_list,
				engine: steam_id_data.engine,
				release_date,
				thumbnail_url: get_steam_thumbnail(&steam_id_data.id),
			})
		})
		.collect())
}
