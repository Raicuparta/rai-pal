use std::collections::HashMap;

use super::mod_provider::ModProvider;
use crate::{
	game_mods::game_mod::GameMod,
	result::Result,
};

pub struct FolderModProvider;

impl ModProvider for FolderModProvider {
	fn default() -> Self {
		Self {}
	}

	async fn get_mods(&self) -> Result<HashMap<String, GameMod>> {
		// TODO
		Ok(HashMap::new())
	}
}
