use super::provider::{
	ProviderActions,
	ProviderStatic,
};
use crate::{
	installed_game,
	mod_loaders::mod_loader,
	serializable_struct,
	Result,
};

serializable_struct!(ManualProvider {});

impl ProviderActions for ManualProvider {
	fn get_installed_games(
		&self,
		mod_loaders: &mod_loader::DataMap,
	) -> Result<installed_game::Map> {
		Ok(installed_game::Map::new())
	}
}

impl ProviderStatic for ManualProvider {
	const ID: &'static str = "manual";

	fn new() -> Result<Self>
	where
		Self: Sized,
	{
		Ok(Self {})
	}
}
