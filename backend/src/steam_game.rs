use std::collections::HashMap;
use steamlocate::SteamDir;

use crate::appinfo::read_appinfo;
use crate::game::Game;
use crate::game::GameMap;
use crate::game_executable::GameExecutable;
use crate::Result;

pub async fn get_steam_games() -> Result<GameMap> {
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
        let game_id = app_id.to_owned();

        app_details_map.insert(
            game_id,
            Game {
                id: game_id,
                name: app.name.clone().unwrap_or_default(),
                executables: app_info
                    .launch_options
                    .iter()
                    .filter_map(|steam_launch| {
                        let executable_id = format!("{game_id}_{}", steam_launch.launch_id.clone());

                        Some((
                            executable_id.clone(),
                            GameExecutable::new(
                                executable_id,
                                &app.path.join(steam_launch.executable.as_ref()?),
                                Some(steam_launch),
                            )?,
                        ))
                    })
                    .collect(),
            },
        );
    }

    Ok(app_details_map)
}
