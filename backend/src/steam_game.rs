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
                executables: app_info
                    .launch_options
                    .iter()
                    .filter_map(|steam_launch| {
                        let id = steam_launch.launch_id.clone();

                        Some((
                            id.clone(),
                            GameExecutable::new(
                                id,
                                &app.path.join(steam_launch.executable.as_ref()?),
                                steam_launch,
                            )?,
                        ))
                    })
                    .collect(),
            },
        );
    }

    app_details_map
}
