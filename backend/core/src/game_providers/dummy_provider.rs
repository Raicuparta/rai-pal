use rai_pal_proc_macros::serializable_struct;

use super::game_provider::ProviderActions;
use crate::{
	local_database::DbMutex,
	game_providers::game_provider::WineProviderActions,
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
