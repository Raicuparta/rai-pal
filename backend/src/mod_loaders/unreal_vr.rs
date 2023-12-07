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
	game_mod::GameMod,
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
		let path = resources_path.join(Self::ID);
		let mods = if path.join(Self::EXE_NAME).exists() {
			let local_mod = LocalMod::new(
				&path.join(Self::EXE_NAME),
				Some(GameEngineBrand::Unreal),
				None,
				ModKind::Runnable,
			)?;

			HashMap::from([(
				local_mod.common.id.clone(),
				GameMod {
					local_mod: Some(local_mod.data),
					remote_mod: None,
					common: local_mod.common,
				},
			)])
		} else {
			HashMap::new()
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

	fn get_mod_path(&self, mod_id: &str) -> Result<PathBuf> {
		todo!()
	}
}
