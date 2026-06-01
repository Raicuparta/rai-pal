use super::mod_provider::ModProvider;
use crate::{
	local_database::game_database::DbMutex,
	result::Result,
};

pub struct FolderModProvider;

impl ModProvider for FolderModProvider {
	fn default() -> Self {
		Self {}
	}

	async fn insert_mods(&self, _db: &DbMutex) -> Result {
		// TODO
		Ok(())
	}
}
