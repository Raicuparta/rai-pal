use serde::Serialize;
use specta::Type;
use std::error::Error;
use steamlocate::SteamDir;

use crate::{appinfo::read_appinfo, steam_game::get_steam_games};

const UNITY_STEAM_APP_IDS_URL: &str =
    "https://raw.githubusercontent.com/Raicuparta/steam-unity-app-ids/main/unity-app-ids.txt";

#[derive(Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct OwnedUnityGame {
    id: String,
    name: String,
    installed: bool,
}

pub async fn get_steam_owned_unity_games() -> Result<Vec<OwnedUnityGame>, Box<dyn Error>> {
    let response = reqwest::get(UNITY_STEAM_APP_IDS_URL).await?.text().await?;
    let steam_games = get_steam_games();

    // TODO this is repeated in steam_game.
    // Should try to cache it.
    let steam_dir = SteamDir::locate().unwrap();
    let app_info = read_appinfo(
        &steam_dir
            .path
            .join("appcache/appinfo.vdf")
            .to_string_lossy(),
    );

    return Ok(response
        .split(',')
        .filter_map(|app_id| {
            let id = app_id.parse::<u32>().ok()?;

            let app_info = app_info.apps.get(&id)?;

            return Some(OwnedUnityGame {
                id: app_id.to_owned(),
                name: app_info.name.to_owned(),
                installed: steam_games.get(&id).is_some(),
            });
        })
        .collect());
}
