use std::collections::HashSet;

use crate::Result;

const STEAM_APP_IDS_URL_BASE: &str =
	"https://raw.githubusercontent.com/Raicuparta/steam-app-ids-by-engine/main";

pub async fn get(list_name: &str) -> Result<HashSet<String>> {
	Ok(
		reqwest::get(format!("{STEAM_APP_IDS_URL_BASE}/{list_name}"))
			.await?
			.text()
			.await?
			.split('\n')
			.map(|id| id.to_string())
			.collect(),
	)
}
