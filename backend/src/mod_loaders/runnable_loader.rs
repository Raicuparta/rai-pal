use std::{
	path::{
		Path,
		PathBuf,
	},
	process::Command,
};

use async_trait::async_trait;
use log::error;

use super::mod_loader::{
	ModLoaderActions,
	ModLoaderData,
	ModLoaderStatic,
};
use crate::{
	game_mod::CommonModData,
	installed_game::InstalledGame,
	local_mod::{
		self,
		LocalMod,
		ModKind,
	},
	mod_manifest,
	paths::glob_path,
	result::Error,
	serializable_struct,
	Result,
};

serializable_struct!(RunnableLoader {
  pub data: ModLoaderData,
});

impl RunnableLoader {}

#[async_trait]
impl ModLoaderStatic for RunnableLoader {
	const ID: &'static str = "runnable";

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
impl ModLoaderActions for RunnableLoader {
	fn get_data(&self) -> &ModLoaderData {
		&self.data
	}

	fn install(&self, _game: &InstalledGame) -> Result {
		todo!()
	}

	async fn install_mod_inner(&self, game: &InstalledGame, local_mod: &LocalMod) -> Result {
		let mod_folder = self.get_mod_path(&local_mod.common)?;

		let runnable = local_mod
			.data
			.manifest
			.as_ref()
			.and_then(|manifest| manifest.runnable.as_ref())
			.ok_or_else(|| Error::RunnableManifestNotFound(local_mod.common.id.clone()))?;

		Command::new(mod_folder.join(&runnable.path))
			.current_dir(mod_folder)
			// .arg(&format!(
			// 	"--attach={}",
			// 	game.executable
			// 		.path
			// 		.file_name()
			// 		.ok_or_else(|| Error::FailedToGetFileName(game.executable.path.clone()))?
			// 		.to_string_lossy()
			// ))
			.args(&runnable.args)
			.spawn()?;

		Ok(())
	}

	fn get_mod_path(&self, mod_data: &CommonModData) -> Result<PathBuf> {
		Ok(Self::get_installed_mods_path()?.join(&mod_data.id))
	}

	fn get_local_mods(&self) -> Result<local_mod::Map> {
		let mods_path = Self::get_installed_mods_path()?;

		let manifests = glob_path(
			&mods_path
				.join("*")
				// TODO manifest name const somewhere.
				.join("rai-pal-manifest.json"),
		)?;

		let mut mod_map = local_mod::Map::default();

		for manifest_path_result in manifests {
			// TODO don't crash whole thing on single mod failure.
			match manifest_path_result {
				Ok(manifest_path) => {
					if let Some(manifest) = mod_manifest::get(&manifest_path) {
						let local_mod = LocalMod::new(
							Self::ID,
							manifest_path.parent().unwrap_or(&manifest_path),
							manifest.engine, // TODO read from manifest.
							manifest.unity_backend,
						)?;

						mod_map.insert(local_mod.common.id.clone(), local_mod);
					}
				}
				Err(error) => {
					error!(
						"Failed to read mod manifest from {}. Error: {}",
						mods_path.display(),
						error
					);
				}
			}
		}

		Ok(mod_map)
	}
}
