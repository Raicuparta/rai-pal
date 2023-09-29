use super::mod_loader::{
	ModLoaderActions,
	ModLoaderData,
	ModLoaderStatic,
};
use crate::serializable_struct;

serializable_struct!(MelonLoader {
  pub data: ModLoaderData,
});

impl ModLoaderStatic for MelonLoader {
	const ID: &'static str = "melonloader";

	fn new(resources_path: &std::path::Path) -> crate::Result<Self>
	where
		Self: std::marker::Sized,
	{
		let path = resources_path.join(Self::ID);

		Ok(Self {
			data: ModLoaderData {
				id: Self::ID.to_string(),
				mods: vec![],
				path,
			},
		})
	}
}

impl ModLoaderActions for MelonLoader {
	fn get_data(&self) -> &ModLoaderData {
		&self.data
	}

	fn install(&self, _game: &crate::game::Game) -> crate::Result {
		todo!()
	}

	fn install_mod(&self, _game: &crate::game::Game, _mod_idd: &str) -> crate::Result {
		todo!()
	}

	fn open_mod_folder(&self, _mod_id: &str) -> crate::Result {
		todo!()
	}
}
