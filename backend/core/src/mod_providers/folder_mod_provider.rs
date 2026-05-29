use std::collections::HashMap;

use super::mod_provider::ModProvider;
use crate::{
	mods::game_mod::GameMod,
	result::Result,
};

pub struct FolderModProvider;

impl ModProvider for FolderModProvider {
	fn default() -> Self {
		Self {}
	}

	async fn get_mods<TCallback>(&self, callback: &TCallback) -> Result
	where
		TCallback: Fn(HashMap<String, GameMod>) + Send + Sync,
	{
		// TODO
		Ok(())
	}
}
