use std::{
	fs,
	path::{
		Path,
		PathBuf,
	},
};

use rai_pal_proc_macros::serializable_struct;

use super::mod_loader::{
	ModLoaderActions,
	ModLoaderData,
	ModLoaderId,
	ModLoaderStatic,
};
use crate::{
	files::copy_dir_all,
	game::DbGame,
	game_engines::game_engine::EngineBrand,
	local_mod::{
		LocalMod,
		ModKind,
	},
	mod_loaders::mod_database::ModConfigs,
	paths,
	result::{
		Error,
		Result,
	},
};

#[serializable_struct]
pub struct Ue4ss {
	pub data: ModLoaderData,
	pub id: ModLoaderId,
}

impl ModLoaderStatic for Ue4ss {
	const ID: ModLoaderId = ModLoaderId::Ue4ss;

	fn new(resources_path: &Path) -> Result<Self> {
		Ok(Self {
			id: Self::ID,
			data: ModLoaderData {
				id: Self::ID,
				path: resources_path.join(Self::ID.as_str()),
				kind: ModKind::Installable,
				engine: Some(EngineBrand::Unreal),
			},
		})
	}
}

impl ModLoaderActions for Ue4ss {
	fn get_data(&self) -> &ModLoaderData {
		&self.data
	}

	fn get_wine_dll_overrides(&self, _game: &DbGame) -> Vec<String> {
		vec!["dwmapi".to_string()]
	}

	async fn install_loader(&self, game: &DbGame, local_mod: &LocalMod) -> Result {
		let installed_mods_folder = game.get_installed_mods_folder()?;

		copy_dir_all(&local_mod.data.path, &installed_mods_folder)?;

		let exe_path = game.try_get_exe_path()?;

		let game_folder = paths::path_parent(exe_path)?;
		fs::create_dir_all(game_folder)?;

		fs::copy(
			local_mod.data.path.join("dwmapi.dll"),
			game_folder.join("dwmapi.dll"),
		)?;

		let ue4ss_path = installed_mods_folder.join("ue4ss").join("UE4SS.dll");
		fs::write(
			game_folder.join("override.txt"),
			ue4ss_path.to_string_lossy().as_ref(),
		)?;

		Ok(())
	}

	async fn install_mod_inner(&self, game: &DbGame, _local_mod: &LocalMod) -> Result {
		Err(Error::ModInstallInfoInsufficient(
			"ue4ss_mod_install_not_implemented".to_string(),
			game.display_title.clone(),
		))
	}

	async fn uninstall_mod(&self, _game: &DbGame, _local_mod: &LocalMod) -> Result {
		Ok(())
	}

	async fn run_without_game(&self, local_mod: &LocalMod) -> Result {
		Err(Error::CantRunNonRunnable(local_mod.common.id.clone()))
	}

	fn open_installed_mod_folder(&self, game: &DbGame, local_mod: &LocalMod) -> Result {
		let game_data_folder = game.get_installed_mods_folder()?;
		let mod_folder = game_data_folder
			.join("ue4ss")
			.join("UE4SS")
			.join("Mods")
			.join(&local_mod.common.id);

		crate::paths::open_folder_or_parent(&mod_folder)
	}

	fn open_loader_folder_for_game(&self, game: &DbGame) -> Result {
		let ue4ss_folder = game
			.get_installed_mods_folder()?
			.join("ue4ss")
			.join("UE4SS");
		crate::paths::open_folder_or_parent(&ue4ss_folder)
	}

	fn get_config_path(&self, game: &DbGame, mod_configs: &ModConfigs) -> Result<PathBuf> {
		Ok(game
			.get_installed_mods_folder()?
			.join("ue4ss")
			.join("UE4SS")
			.join(&mod_configs.destination_path))
	}
}
