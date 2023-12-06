use async_trait::async_trait;

use super::{
	mod_database,
	mod_loader::{
		ModLoaderActions,
		ModLoaderData,
		ModLoaderStatic,
	},
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
				mods: vec![],
				path,
				database: mod_database::get(Self::ID).await.ok(), // TODO show error somewhere.
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
