use crate::appinfo;
use crate::game;
use crate::Result;
use anyhow::anyhow;
use std::collections::HashMap;
use std::string;
use steamlocate::SteamDir;

pub async fn get() -> Result<game::Map> {
    let mut steam_dir =
        SteamDir::locate().ok_or_else(|| anyhow!("Failed to locate Steam on this system."))?;

    let app_info = appinfo::read(&steam_dir.path.join("appcache/appinfo.vdf"));

    let mut game_map: game::Map = HashMap::new();

    for (_, app, app_info) in steam_dir.apps().iter().filter_map(|(app_id, app)| {
        Some((
            app_id,
            app.as_ref()?,
            app_info.as_ref().ok()?.apps.get(app_id)?,
        ))
    }) {
        for launch_option in &app_info.launch_options {
            let executable_id = format!("{}_{}", launch_option.app_id, launch_option.launch_id);

            if let Some(executable_path) = launch_option.executable.as_ref() {
                let full_path = &app.path.join(executable_path);

                if let Some(name) = &app.name {
                    let discriminator = if game_map.values().any(|game| &game.name == name) {
                        executable_path.to_str().map(string::ToString::to_string)
                    } else {
                        None
                    };

                    if let Some(game) = game::Game::new(
                        executable_id.clone(),
                        name.clone(),
                        discriminator,
                        full_path,
                        Some(launch_option),
                    ) {
                        game_map.insert(executable_id.clone(), game);
                    }
                }
            }
        }
    }

    Ok(game_map)
}
