use super::provider::{
	ProviderActions,
	ProviderStatic,
};
use crate::{
	installed_game,
	serializable_struct,
	Result,
};

serializable_struct!(SteamProvider {});

impl ProviderActions for SteamProvider {
	fn get_installed_games(&self) -> Result<installed_game::Map> {
		todo!()
	}
}

impl ProviderStatic for SteamProvider {
	const ID: &'static str = "steam";

	fn new() -> Result<Self>
	where
		Self: Sized,
	{
		todo!()
	}
}
