use serde::Serialize;
use specta::Type;
use std::error::Error;

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

    return Ok(response
        .split(',')
        .map(|app_id| OwnedUnityGame {
            id: app_id.to_owned(),
            name: app_id.to_owned(),
            installed: false,
        })
        .collect());
}
