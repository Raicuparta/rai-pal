use std::collections::HashMap;

use crate::{
	game_mods::{
		game_mod::GameMod,
		mod_providers::{
			folder_mod_provider::FolderModProvider,
			url_mod_provider::UrlModProvider,
		},
	},
	result::Result,
};

pub trait ModProvider {
	fn default() -> Self;
	async fn get_mods(&self) -> Result<HashMap<String, GameMod>>;
}

pub async fn get_all_mods() -> Result<HashMap<String, GameMod>> {
	let mut mods = HashMap::new();
	mods.extend(FolderModProvider::default().get_mods().await?);
	mods.extend(UrlModProvider::default().get_mods().await?);
	Ok(mods)
}
