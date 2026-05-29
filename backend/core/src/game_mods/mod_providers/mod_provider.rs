use std::collections::HashMap;

use crate::game_mods::game_mod::GameMod;
use crate::result::Result;

pub trait ModProvider {
	fn get_mods(&self) -> Result<HashMap<String, GameMod>>;
}
