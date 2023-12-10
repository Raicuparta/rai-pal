use std::{
	collections::HashMap,
	path::{
		Path,
		PathBuf,
	},
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
	windows,
	Result,
};

serializable_struct!(UnrealVr {
  pub data: ModLoaderData,
});

impl UnrealVr {
	const EXE_NAME: &'static str = "UnrealVR.exe";
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

	async fn install_mod(&self, game: &InstalledGame, local_mod: &LocalMod) -> Result {
		let parameters = format!(
			"--attach=\"{}\"",
			game.executable
				.path
				.file_name()
				.ok_or_else(|| Error::FailedToGetFileName(game.executable.path.clone()))?
				.to_string_lossy()
		);

		windows::run_as_admin(
			&self.get_mod_path(&local_mod.common)?.join(Self::EXE_NAME),
			&parameters,
		)
	}

	fn get_mod_path(&self, _game_mod: &CommonModData) -> Result<PathBuf> {
		Ok(self.get_data().path.clone())
	}

	fn get_local_mods(&self) -> Result<HashMap<String, LocalMod>> {
		let local_mod = LocalMod::new(
			Self::ID,
			&self.get_data().path.join(Self::EXE_NAME),
			Some(GameEngineBrand::Unreal),
			None,
		)?;

		Ok(HashMap::from([(local_mod.common.id.clone(), local_mod)]))
	}
}
