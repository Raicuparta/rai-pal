use std::{
	collections::HashMap,
	path::{
		Path,
		PathBuf,
	},
	process::Command,
};

use async_trait::async_trait;

use super::mod_loader::{
	ModLoaderActions,
	ModLoaderData,
	ModLoaderStatic,
};
use crate::{
	game_engines::game_engine::GameEngineBrand,
	game_mod::CommonModData,
	installed_game::InstalledGame,
	local_mod::{
		LocalMod,
		ModKind,
	},
	result::Error,
	serializable_struct,
	Result,
};

serializable_struct!(UnrealVr {
  pub data: ModLoaderData,
});

impl UnrealVr {
	const EXE_NAME: &'static str = "UEVRInjector.exe";
}

#[async_trait]
impl ModLoaderStatic for UnrealVr {
	const ID: &'static str = "uevr";

	async fn new(resources_path: &Path) -> Result<Self>
	where
		Self: std::marker::Sized,
	{
		Ok(Self {
			data: ModLoaderData {
				id: Self::ID.to_string(),
				path: resources_path.join(Self::ID),
				kind: ModKind::Runnable,
			},
		})
	}
}

#[async_trait]
impl ModLoaderActions for UnrealVr {
	fn get_data(&self) -> &ModLoaderData {
		&self.data
	}

	fn install(&self, _game: &InstalledGame) -> Result {
		todo!()
	}

	async fn install_mod_inner(&self, game: &InstalledGame, local_mod: &LocalMod) -> Result {
		let mod_folder = self.get_mod_path(&local_mod.common)?;

		Command::new(mod_folder.join(Self::EXE_NAME))
			.current_dir(mod_folder)
			.arg(&format!(
				"--attach={}",
				game.executable
					.path
					.file_name()
					.ok_or_else(|| Error::FailedToGetFileName(game.executable.path.clone()))?
					.to_string_lossy()
			))
			.spawn()?;

		Ok(())
	}

	fn get_mod_path(&self, mod_data: &CommonModData) -> Result<PathBuf> {
		Ok(Self::get_installed_mods_path()?.join(&mod_data.id))
	}

	fn get_local_mods(&self) -> Result<HashMap<String, LocalMod>> {
		let folder_path = &Self::get_installed_mods_path()?.join(Self::ID);
		if !folder_path.join(Self::EXE_NAME).is_file() {
			return Ok(HashMap::default());
		}

		let local_mod = LocalMod::new(Self::ID, folder_path, Some(GameEngineBrand::Unreal), None)?;

		Ok(HashMap::from([(local_mod.common.id.clone(), local_mod)]))
	}
}
