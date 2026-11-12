use super::game_provider::ProviderActions;
use crate::{game::Game, game_providers::game_provider::WineProviderActions, result::Result};

pub struct Dummy;

impl ProviderActions for Dummy {
	fn get_games(&self) -> Result<Vec<Game>> {
		Ok(Vec::new())
	}
}

impl WineProviderActions for Dummy {}
