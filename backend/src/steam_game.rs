use std::collections::HashMap;
use steamlocate::SteamDir;

use crate::appinfo::read_appinfo;
use crate::game::Game;
use crate::game::GameMap;
use crate::game_executable::get_unity_scripting_backend;
use crate::game_executable::is_unity_exe;
use crate::game_executable::Architecture;
use crate::game_executable::GameExecutable;
use crate::game_executable::OperatingSystem;

pub fn get_steam_games() -> GameMap {
    let mut steam_dir = SteamDir::locate().unwrap();
    let app_info = read_appinfo(
        &steam_dir
            .path
            .join("appcache/appinfo.vdf")
            .to_string_lossy(),
    );

    let mut app_details_map: GameMap = HashMap::new();
    for (app_id, app_option) in steam_dir.apps() {
        if let Some(app) = app_option {
            if let Some(steam_launch_options) = app_info.apps.get(&app_id) {
                let id = app_id.to_owned();

                app_details_map.insert(
                    id,
                    Game {
                        id,
                        name: app.name.clone().unwrap_or_default(),
                        distinct_executables: steam_launch_options
                            .launch_options
                            .iter()
                            .filter_map(|launch_option| {
                                let executable = launch_option.executable.as_ref();
                                let full_path = app.path.join(executable?);

                                if !is_unity_exe(&full_path) {
                                    return None;
                                }

                                return Some(GameExecutable {
                                    architecture: Architecture::X64,
                                    full_path: full_path.clone(),
                                    id: launch_option.launch_id.clone(),
                                    is_legacy: false,
                                    operating_system: OperatingSystem::Linux,
                                    mod_files_path: String::from(""),
                                    name: executable?.to_str()?.to_owned(),
                                    scripting_backend: get_unity_scripting_backend(&full_path)
                                        .ok()?,
                                    steam_launch: Some(launch_option.clone()),
                                    unity_version: String::from("2020"),
                                });
                            })
                            .collect(),
                        executables: Vec::new(), // TODO distinguish them!
                    },
                );
            }
        }
    }

    app_details_map
}
