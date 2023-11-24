use steamlocate::SteamDir;

use super::{
	appinfo::{self,},
	id_lists::{
		self,
		SteamGame,
	},
};
use crate::Result;

pub async fn get() -> Result<Vec<SteamGame>> {
	let steam_dir = SteamDir::locate()?;
	// TODO This is already done in the Steam provider, avoid repetition.
	let app_info = appinfo::read(steam_dir.path())?;

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
