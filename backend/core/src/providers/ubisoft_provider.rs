use rai_pal_proc_macros::serializable_struct;

use super::provider::{ProviderActions, ProviderId, ProviderStatic};
use crate::{game::DbGame, result::Result};

#[serializable_struct]
pub struct Ubisoft {}

impl ProviderStatic for Ubisoft {
	const ID: &'static ProviderId = &ProviderId::Ubisoft;

	fn new() -> Result<Self>
	where
		Self: Sized,
	{
		Ok(Self {})
	}
}

impl ProviderActions for Ubisoft {
	async fn get_games<TCallback>(&self, mut _callback: TCallback) -> Result
	where
		TCallback: FnMut(DbGame) + Send + Sync,
	{
		// Nothing for now
		// we're currently only getting subscription games from the remote game database.
		Ok(())
	}
}
