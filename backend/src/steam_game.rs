use std::collections::HashMap;
use steamlocate::SteamDir;

use crate::appinfo::read_appinfo;
use crate::game::Game;
use crate::game::GameMap;
use crate::game_executable::is_unity_exe;
use crate::game_executable::GameExecutable;

pub fn get_steam_apps() -> GameMap {
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
                            .iter()
                            .filter_map(|launch_option| {
                                if let Some(executable) = &launch_option.executable {
                                    let full_path = app.path.join(executable);

                                    if !is_unity_exe(&full_path) {
                                        return None;
                                    }

                                    return Some(GameExecutable {
                                        architecture: String::from("x64"),
                                        full_path,
                                        id: launch_option.launch_id.clone(),
                                        is_legacy: false,
                                        is_linux: false,
                                        mod_files_path: String::from(""),
                                        name: executable.clone(),
                                        scripting_backend: String::from("il2cpp"),
                                        steam_launch: Some(launch_option.clone()),
                                        unity_version: String::from("2020"),
                                    });
                                }
                                return None;
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
