use std::path::Path;

use super::mod_loader::{
	ModLoaderActions,
	ModLoaderData,
	ModLoaderStatic,
};
use crate::{
	game_engines::game_engine::GameEngineBrand,
	game_mod::{
		Mod,
		ModKind,
	},
	installed_game::InstalledGame,
	result::Error,
	serializable_struct,
	windows,
	Result,
};

serializable_struct!(UnrealVr {
  pub data: ModLoaderData,
});

impl UnrealVr {
	const EXE_NAME: &'static str = "UnrealVR.exe";
}

impl ModLoaderStatic for UnrealVr {
	const ID: &'static str = "uevr";

	fn new(resources_path: &Path) -> Result<Self>
	where
		Self: std::marker::Sized,
	{
		let path = resources_path.join(Self::ID);
		let mods = if path.join(Self::EXE_NAME).exists() {
			vec![Mod::new(
				&path.join(Self::EXE_NAME),
				Some(GameEngineBrand::Unreal),
				None,
				ModKind::Runnable,
			)?]
		} else {
			vec![]
		};

		Ok(Self {
			data: ModLoaderData {
				id: Self::ID.to_string(),
				mods,
				path,
			},
		})
	}
}

impl ModLoaderActions for UnrealVr {
	fn get_data(&self) -> &ModLoaderData {
		&self.data
	}

	fn install(&self, game: &InstalledGame) -> Result {
		let parameters = format!(
			"--attach=\"{}\"",
			game.executable
				.path
				.file_name()
				.ok_or_else(|| Error::FailedToGetFileName(game.executable.path.clone()))?
				.to_string_lossy()
		);

		windows::run_as_admin(&self.data.path.join(Self::EXE_NAME), &parameters)
	}

	fn install_mod(&self, game: &InstalledGame, _mod_idd: &str) -> Result {
		self.install(game)
	}

	fn open_mod_folder(&self, _mod_id: &str) -> Result {
		Ok(open::that_detached(&self.data.path)?)
	}
}
