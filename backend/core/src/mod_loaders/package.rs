use std::path::{
	Path,
	PathBuf,
};

use rai_pal_proc_macros::serializable_struct;

use super::mod_loader::{
	ModLoaderActions,
	ModLoaderData,
	ModLoaderId,
	ModLoaderStatic,
};
use crate::{
	game::DbGame,
	local_mod::{
		LocalMod,
		ModKind,
	},
	mod_loaders::mod_database::ModConfigs,
	paths::{
		self,
	},
	result::{
		Error,
		Result,
	},
};

#[serializable_struct]
pub struct PackageLoader {
	pub data: ModLoaderData,
}

impl ModLoaderStatic for PackageLoader {
	const ID: ModLoaderId = ModLoaderId::Package;

	fn new(resources_path: &Path) -> Result<Self>
	where
		Self: std::marker::Sized,
	{
		Ok(Self {
			data: ModLoaderData {
				id: Self::ID,
				path: resources_path.join(Self::ID.as_str()),
				kind: ModKind::Runnable,
				engine: None,
			},
		})
	}
}

impl ModLoaderActions for PackageLoader {
	fn get_data(&self) -> &ModLoaderData {
		&self.data
	}

	async fn install_mod_inner(&self, _game: &DbGame, _local_mod: &LocalMod) -> Result {
		Ok(())
	}

	async fn uninstall_mod_inner(&self, _game: &DbGame, _local_mod: &LocalMod) -> Result {
		Ok(())
	}

	async fn run_without_game(&self, _local_mod: &LocalMod) -> Result {
		Err(Error::UnsupportedOperation(
			"Package does not support running".to_string(),
		))
	}

	fn open_installed_mod_folder(&self, _game: &DbGame, local_mod: &LocalMod) -> Result {
		paths::open_folder_or_parent(&local_mod.data.path)
	}

	fn get_config_path(&self, _game: &DbGame, _mod_configs: &ModConfigs) -> Result<PathBuf> {
		Err(Error::UnsupportedOperation(
			"Package does not support configs".to_string(),
		))
	}
}
