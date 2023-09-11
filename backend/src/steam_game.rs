use std::collections::HashMap;
use steamlocate::SteamDir;

use crate::appinfo::read_appinfo;
use crate::game::Game;
use crate::game::GameMap;
use crate::game_executable::get_os_and_architecture;
use crate::game_executable::get_unity_scripting_backend;
use crate::game_executable::get_unity_version;
use crate::game_executable::is_unity_exe;
use crate::game_executable::GameExecutable;

pub fn get_steam_games() -> GameMap {
    let mut steam_dir = SteamDir::locate().unwrap();
    let app_info = read_appinfo(
        &steam_dir
            .path
            .join("appcache/appinfo.vdf")
            .to_string_lossy(),
    );

    let mut app_details_map: GameMap = HashMap::new();

    for (app_id, app, app_info) in steam_dir
        .apps()
        .iter()
        .filter_map(|(app_id, app)| Some((app_id, app.as_ref()?, app_info.apps.get(app_id)?)))
    {
        let id = app_id.to_owned();

        app_details_map.insert(
            id,
            Game {
                id,
                name: app.name.clone().unwrap_or_default(),
                distinct_executables: app_info
                    .launch_options
                    .iter()
                    .filter_map(|launch_option| {
                        let executable = launch_option.executable.as_ref();
                        let full_path = app.path.join(executable?);
                        let (operating_system, architecture) =
                            get_os_and_architecture(&full_path).ok()?;

                        if !is_unity_exe(&full_path) {
                            return None;
                        }

                        return Some(GameExecutable {
                            architecture,
                            full_path: full_path.clone(),
                            id: launch_option.launch_id.clone(),
                            is_legacy: false,
                            operating_system,
                            mod_files_path: String::from(""),
                            name: executable?.to_str()?.to_owned(),
                            scripting_backend: get_unity_scripting_backend(&full_path).ok()?,
                            steam_launch: Some(launch_option.clone()),
                            unity_version: get_unity_version(&full_path),
                        });
                    })
                    .collect(),
                executables: Vec::new(), // TODO distinguish them!
            },
        );
    }

    app_details_map
}
