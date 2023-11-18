use super::provider::{
	ProviderActions,
	ProviderStatic,
};
use crate::{
	installed_game,
	serializable_struct,
	Result,
};

serializable_struct!(ManualProvider {});

impl ProviderActions for ManualProvider {
	fn get_installed_games(&self) -> Result<installed_game::Map> {
		todo!()
	}
}

impl ProviderStatic for ManualProvider {
	const ID: &'static str = "manual";

	fn new() -> Result<Self>
	where
		Self: Sized,
	{
		todo!()
	}
}
