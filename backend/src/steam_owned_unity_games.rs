use crate::{appinfo, game::OperatingSystem, serializable_struct, Result};
use anyhow::anyhow;
use std::collections::HashSet;
use steamlocate::SteamDir;

const UNITY_STEAM_APP_IDS_URL: &str =
    "https://raw.githubusercontent.com/Raicuparta/steam-unity-app-ids/main/unity-app-ids.txt";

serializable_struct!(OwnedUnityGame {
    id: String,
    name: String,
    installed: bool,
    os_list: HashSet<OperatingSystem>,
});

pub async fn get(_: tauri::AppHandle) -> Result<Vec<OwnedUnityGame>> {
    let response = reqwest::get(UNITY_STEAM_APP_IDS_URL).await?.text().await?;

    // TODO this is repeated in steam_game.
    // Should try to cache it.
    let steam_dir =
        SteamDir::locate().ok_or_else(|| anyhow!("Failed to locate Steam on this system."))?;
    let app_info = appinfo::read(&steam_dir.path.join("appcache/appinfo.vdf"));

    return Ok(response
        .split(',')
        .filter_map(|app_id| {
            let id = app_id.parse::<u32>().ok()?;

            let app_info = app_info.as_ref().ok()?.apps.get(&id)?;

            let os_list: HashSet<_> = app_info
                .launch_options
                .iter()
                .filter_map(|launch| match launch.clone().os_list?.as_str() {
                    "linux" => Some(OperatingSystem::Linux),
                    "windows" => Some(OperatingSystem::Windows),
                    _ => None,
                })
                .collect();

            Some(OwnedUnityGame {
                id: app_id.to_owned(),
                name: app_info.name.clone(),
                // TODO do this some other way
                // installed: steam_games.get(&id).is_some(),
                installed: false,
                os_list,
            })
        })
        .collect());
}
