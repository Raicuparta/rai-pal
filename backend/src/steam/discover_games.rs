use super::{
	appinfo::SteamAppInfoFile,
	id_lists::{
		self,
		SteamGame,
	},
};
use crate::Result;

pub async fn get(app_info: &SteamAppInfoFile) -> Result<Vec<SteamGame>> {
	Ok(id_lists::get()
		.await?
		.into_iter()
		.filter(|steam_game| {
			if let Ok(id_number) = steam_game.id.parse::<u32>() {
				if app_info.apps.contains_key(&id_number) {
					return false;
				}
			} else {
				return false;
			}

			true
		})
		.collect())
}
