use std::{
	collections::{
		HashMap,
		HashSet,
	},
	path::PathBuf,
	string,
};

use steamlocate::SteamDir;

use super::{
	appinfo::SteamAppInfoFile,
	thumbnail::get_steam_thumbnail,
};
use crate::{
	installed_game,
	mod_loaders::mod_loader,
	Result,
};

pub async fn get(
	steam_dir: &SteamDir,
	app_info_file: &SteamAppInfoFile,
	mod_loaders: &mod_loader::DataMap,
) -> Result<installed_game::Map> {
	let mut game_map: installed_game::Map = HashMap::new();
	let mut used_paths: HashSet<PathBuf> = HashSet::new();
	let mut used_names: HashSet<String> = HashSet::new();

	for library in (steam_dir.libraries()?).flatten() {
		for app in library.apps().flatten() {
			if let Some(app_info) = app_info_file.apps.get(&app.app_id) {
				let sorted_launch_options = {
					let mut sorted_launch_options = app_info.launch_options.clone();
					sorted_launch_options.sort_by(|a, b| a.launch_id.cmp(&b.launch_id));
					sorted_launch_options
				};

				for launch_option in sorted_launch_options {
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

							if let Some(game) = installed_game::InstalledGame::new(
								&executable_id,
								name,
								discriminator,
								full_path,
								Some(&launch_option),
								Some(get_steam_thumbnail(&app.app_id.to_string())),
								mod_loaders,
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
