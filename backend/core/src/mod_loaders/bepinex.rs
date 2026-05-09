use std::{
	fs,
	path::{
		Path,
		PathBuf,
	},
};

use rai_pal_proc_macros::serializable_struct;

use super::mod_loader::ModLoaderStatic;
use crate::{
	files::copy_dir_all,
	game::DbGame,
	game_engines::game_engine::EngineBrand,
	local_mod::{
		LocalMod,
		ModKind,
	},
	mod_loaders::{
		mod_database::ModConfigs,
		mod_loader::{
			ModLoaderActions,
			ModLoaderData,
			ModLoaderId,
		},
	},
	paths,
	result::{
		Error,
		Result,
	},
};

#[serializable_struct]
pub struct BepInEx {
	pub data: ModLoaderData,
	pub id: ModLoaderId,
}

impl ModLoaderStatic for BepInEx {
	const ID: ModLoaderId = ModLoaderId::BepInEx;

	fn new(resources_path: &Path) -> Result<Self> {
		Ok(Self {
			id: Self::ID,
			data: ModLoaderData {
				id: Self::ID,
				path: resources_path.join(Self::ID.as_str()),
				kind: ModKind::Installable,
				engine: Some(EngineBrand::Unity),
			},
		})
	}
}

impl ModLoaderActions for BepInEx {
	fn get_data(&self) -> &ModLoaderData {
		&self.data
	}

	fn get_wine_dll_overrides(&self, _game: &DbGame) -> Vec<String> {
		vec!["winhttp".to_string()]
	}

	async fn install_loader(&self, game: &DbGame, local_mod: &LocalMod) -> Result {
		copy_dir_all(&local_mod.data.path, game.get_installed_mods_folder()?)?;

		let exe_path = game.try_get_exe_path()?;
		let unity_backend = game.unity_backend.ok_or_else(|| {
			Error::ModInstallInfoInsufficient(
				"unity_backend".to_string(),
				game.display_title.clone(),
			)
		})?;

		let unity_backend_path = self.data.path.join(unity_backend.to_string());
		let game_mods_folder = game.get_installed_mods_folder()?;
		let game_folder = paths::path_parent(exe_path)?;

		fs::copy(
			game_mods_folder.join("winhttp.dll"),
			game_folder.join("winhttp.dll"),
		)?;

		let config_origin_path = self.data.path.join("config").join(if is_legacy(game) {
			"BepInEx-legacy.cfg"
		} else {
			"BepInEx.cfg"
		});

		let config_target_folder = game_mods_folder.join("BepInEx").join("config");
		fs::create_dir_all(&config_target_folder)?;
		fs::copy(config_origin_path, config_target_folder.join("BepInEx.cfg"))?;

		let doorstop_config = fs::read_to_string(unity_backend_path.join("doorstop_config.ini"))?;

		fs::write(
			game_folder.join("doorstop_config.ini"),
			doorstop_config.replace(
				"{{MOD_FILES_PATH}}",
				game_mods_folder.to_string_lossy().as_ref(),
			),
		)?;

		Ok(())
	}

	async fn install_mod_inner(&self, game: &DbGame, local_mod: &LocalMod) -> Result {
		let bepinex_folder = game.get_installed_mods_folder()?.join("BepInEx");

		let mod_plugin_path = local_mod.data.path.join("plugins");
		if mod_plugin_path.is_dir() {
			copy_dir_all(
				mod_plugin_path,
				bepinex_folder.join("plugins").join(&local_mod.common.id),
			)?;
		}

		let mod_patch_path = local_mod.data.path.join("patchers");
		if mod_patch_path.is_dir() {
			copy_dir_all(
				mod_patch_path,
				bepinex_folder.join("patchers").join(&local_mod.common.id),
			)?;
		}

		Ok(())
	}

	async fn uninstall_mod_inner(&self, game: &DbGame, local_mod: &LocalMod) -> Result {
		let installed_mods_folder = game.get_installed_mods_folder()?;
		let bepinex_folder = installed_mods_folder.join("BepInEx");

		let plugins_folder = bepinex_folder.join("plugins").join(&local_mod.common.id);
		if plugins_folder.is_dir() {
			fs::remove_dir_all(plugins_folder)?;
		}

		let patchers_folder = bepinex_folder.join("patchers").join(&local_mod.common.id);
		if patchers_folder.is_dir() {
			fs::remove_dir_all(patchers_folder)?;
		}

		Ok(())
	}

	async fn run_without_game(&self, local_mod: &LocalMod) -> Result {
		Err(Error::CantRunNonRunnable(local_mod.common.id.clone()))
	}

	fn open_installed_mod_folder(&self, game: &DbGame, local_mod: &LocalMod) -> Result {
		let game_data_folder = game.get_installed_mods_folder()?;
		let plugin_folder = game_data_folder
			.join("BepInEx")
			.join("plugins")
			.join(&local_mod.common.id);

		paths::open_folder_or_parent(&plugin_folder)
	}

	fn get_config_path(&self, game: &DbGame, mod_configs: &ModConfigs) -> Result<PathBuf> {
		let destination_path = game
			.get_installed_mods_folder()?
			.join("BepInEx")
			.join(&mod_configs.destination_path);

		Ok(destination_path)
	}
}

fn is_legacy(game: &DbGame) -> bool {
	game.engine_version_major.is_some_and(|major| {
		major < 5 || (major == 5 && game.engine_version_minor.is_some_and(|minor| minor < 5))
	})
}

#[cfg(target_os = "linux")]
pub fn set_up_proton_environment() -> Result {
	let path = paths::base_dirs()?.config_dir().join("environment.d");

	fs::create_dir_all(&path)?;

	fs::write(
		path.join("90-rai-pal-wine-overrides.conf"),
		"WINEDLLOVERRIDES=\"winhttp.dll=n,b\"",
	)?;

	Ok(())
}
