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
	async fn get_mods<TCallback>(&self, callback: &TCallback) -> Result
	where
		TCallback: Fn(HashMap<String, GameMod>) + Send + Sync;
}

pub async fn get_all_mods<TCallback>(callback: TCallback) -> Result
where
	TCallback: Fn(HashMap<String, GameMod>) + Send + Sync,
{
	FolderModProvider::default().get_mods(&callback).await?;
	UrlModProvider::default().get_mods(&callback).await?;
	Ok(())
}
