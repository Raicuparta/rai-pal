use rai_pal_proc_macros::serializable_struct;

use super::provider::ProviderActions;
use crate::{
	local_database::DbMutex,
	result::Result,
};

#[serializable_struct]
pub struct Dummy;

impl ProviderActions for Dummy {
	fn insert_games(&self, _: &DbMutex) -> Result {
		Ok(())
	}
}
