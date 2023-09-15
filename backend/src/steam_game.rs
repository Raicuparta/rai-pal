use std::collections::HashMap;
use steamlocate::SteamDir;

use crate::appinfo::read_appinfo;
use crate::game_executable::{GameExecutable, GameMap};
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

    for (_, app, app_info) in steam_dir
        .apps()
        .iter()
        .filter_map(|(app_id, app)| Some((app_id, app.as_ref()?, app_info.apps.get(app_id)?)))
    {
        for launch_option in &app_info.launch_options {
            let executable_id = format!("{}_{}", launch_option.app_id, launch_option.launch_id);

            if let Some(executable_path) = launch_option.executable.as_ref() {
                if let Some(game_executable) = GameExecutable::new(
                    executable_id.clone(),
                    &app.path.join(executable_path),
                    Some(launch_option),
                ) {
                    app_details_map.insert(executable_id.clone(), game_executable);
                }
            }
        }
    }

    Ok(app_details_map)
}
