use rai_pal_proc_macros::serializable_struct;

use super::game_provider::ProviderActions;
use crate::{
	game_providers::game_provider::WineProviderActions,
	local_database::game_database::DbMutex,
	result::Result,
};

#[serializable_struct]
pub struct Dummy;

impl ProviderActions for Dummy {
	fn insert_games(&self, _: &DbMutex) -> Result {
		Ok(())
	}
}

impl WineProviderActions for Dummy {}
