use std::collections::HashMap;

use super::mod_provider::ModProvider;
use crate::game_mods::game_mod::GameMod;
use crate::result::Result;

pub struct FolderModProvider;

impl ModProvider for FolderModProvider {
	fn get_mods(&self) -> Result<HashMap<String, GameMod>> {
		todo!("FolderModProvider is not yet implemented")
	}
}
