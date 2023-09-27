use std::{
	collections::{
		HashMap,
		HashSet,
	},
	path::PathBuf,
	string,
};

use steamlocate::SteamDir;

use super::appinfo;
use crate::{
	game,
	Result,
};

pub async fn get() -> Result<game::Map> {
	let steam_dir = SteamDir::locate()?;

	let app_info_file = appinfo::read(&steam_dir.path().join("appcache/appinfo.vdf"))?;

	let mut game_map: game::Map = HashMap::new();

	for library in (steam_dir.libraries()?).flatten() {
		for app in library.apps().flatten() {
			if let Some(app_info) = app_info_file.apps.get(&app.app_id) {
				let mut launch_options = app_info.launch_options.clone();
				launch_options.sort_by(|a, b| a.launch_id.cmp(&b.launch_id));

				let mut used_paths: HashSet<PathBuf> = HashSet::new();
				let mut used_names: HashSet<String> = HashSet::new();

				for launch_option in launch_options {
					let executable_id =
						format!("{}_{}", launch_option.app_id, launch_option.launch_id);

					if let Some(executable_path) = launch_option.executable.as_ref() {
						let full_path = &app.path.join(executable_path);

						if used_paths.contains(full_path) {
							continue;
						}

						if let Some(name) = &app.name {
							let discriminator = if used_names.contains(name) {
								launch_option.description.as_ref().map_or_else(
									|| executable_path.to_str().map(string::ToString::to_string),
									|description| Some(description.clone()),
								)
							} else {
								None
							};

							if let Some(game) = game::Game::new(
								executable_id.clone(),
								name.clone(),
								discriminator,
								full_path,
								Some(&launch_option),
							) {
								game_map.insert(executable_id.clone(), game);
								used_names.insert(name.clone());
								used_paths.insert(full_path.clone());
							}
						}
					}
				}
			}
		}
	}

	Ok(game_map)
}