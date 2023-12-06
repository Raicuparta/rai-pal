use std::collections::HashMap;

use async_trait::async_trait;

use super::mod_loader::{
	ModLoaderActions,
	ModLoaderData,
	ModLoaderStatic,
};
use crate::{
	installed_game::InstalledGame,
	serializable_struct,
};

serializable_struct!(MelonLoader {
  pub data: ModLoaderData,
});

#[async_trait]
impl ModLoaderStatic for MelonLoader {
	const ID: &'static str = "melonloader";

	async fn new(resources_path: &std::path::Path) -> crate::Result<Self>
	where
		Self: std::marker::Sized,
	{
		let path = resources_path.join(Self::ID);

		Ok(Self {
			data: ModLoaderData {
				id: Self::ID.to_string(),
				mods: HashMap::new(),
				path,
			},
		})
	}
}

impl ModLoaderActions for MelonLoader {
	fn get_data(&self) -> &ModLoaderData {
		&self.data
	}

	fn install(&self, _game: &InstalledGame) -> crate::Result {
		todo!()
	}

	fn install_mod(&self, _game: &InstalledGame, _mod_idd: &str) -> crate::Result {
		todo!()
	}

	fn open_mod_folder(&self, _mod_id: &str) -> crate::Result {
		todo!()
	}
}
