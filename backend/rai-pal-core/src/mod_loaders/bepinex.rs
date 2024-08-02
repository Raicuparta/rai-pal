use std::{
	collections::HashMap,
	fs::{self, File},
	path::{Path, PathBuf},
};

use async_trait::async_trait;
use rai_pal_proc_macros::serializable_struct;
use zip::ZipArchive;

use super::mod_loader::ModLoaderStatic;
use crate::{
	files::copy_dir_all,
	game_engines::{
		game_engine::{EngineBrand, GameEngine},
		unity::UnityScriptingBackend,
	},
	game_mod::CommonModData,
	installed_game::InstalledGame,
	local_mod::{LocalMod, ModKind},
	mod_loaders::mod_loader::{ModLoaderActions, ModLoaderData},
	paths,
	result::{Error, Result},
};

#[serializable_struct]
pub struct BepInEx {
	pub data: ModLoaderData,
	pub id: &'static str,
}

impl ModLoaderStatic for BepInEx {
	const ID: &'static str = "bepinex";

	fn new(resources_path: &Path) -> Result<Self> {
		Ok(Self {
			id: Self::ID,
			data: ModLoaderData {
				id: Self::ID.to_string(),
				path: resources_path.join(Self::ID),
				kind: ModKind::Installable,
			},
		})
	}
}

#[async_trait]
impl ModLoaderActions for BepInEx {
	fn get_data(&self) -> &ModLoaderData {
		&self.data
	}

	fn install(&self, game: &InstalledGame) -> Result {
		let scripting_backend_path = &self.data.path.join(
			game.executable
				.scripting_backend
				.ok_or_else(|| {
					Error::ModInstallInfoInsufficient(
						"scripting_backend".to_string(),
						game.executable.path.clone(),
					)
				})?
				.to_string(),
		);
		let architecture_path = scripting_backend_path
			.join(
				// Hardcoded windows platform since that's all we support for now.
				"Windows",
			)
			.join(
				game.executable
					.architecture
					.ok_or_else(|| {
						Error::ModInstallInfoInsufficient(
							"architecture".to_string(),
							game.executable.path.clone(),
						)
					})?
					.to_string(),
			);

		let mod_loader_archive = architecture_path.join("mod-loader.zip");
		let folder_to_copy_to_game = architecture_path.join("copy-to-game");
		let game_data_folder = &game.get_installed_mods_folder()?;

		ZipArchive::new(File::open(mod_loader_archive)?)?.extract(game_data_folder)?;

		let game_folder = paths::path_parent(&game.executable.path)?;

		copy_dir_all(folder_to_copy_to_game, game_folder)?;

		let is_legacy = game.executable.engine.as_ref().map_or(false, is_legacy);

		let config_origin_path = &self.data.path.join("config").join(if is_legacy {
			"BepInEx-legacy.cfg"
		} else {
			"BepInEx.cfg"
		});

		let config_target_folder = game_data_folder.join("BepInEx").join("config");

		fs::create_dir_all(&config_target_folder)?;

		fs::copy(config_origin_path, config_target_folder.join("BepInEx.cfg"))?;

		let doorstop_config =
			fs::read_to_string(scripting_backend_path.join("doorstop_config.ini"))?;

		fs::write(
			game_folder.join("doorstop_config.ini"),
			doorstop_config.replace(
				"{{MOD_FILES_PATH}}",
				game_data_folder.to_string_lossy().as_ref(),
			),
		)?;

		Ok(())
	}

	async fn install_mod_inner(&self, game: &InstalledGame, local_mod: &LocalMod) -> Result {
		self.install(game)?;

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

	async fn uninstall_mod(&self, game: &InstalledGame, local_mod: &LocalMod) -> Result {
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

		let manifest_path = game.get_installed_mod_manifest_path(&local_mod.common.id)?;
		if manifest_path.is_file() {
			fs::remove_file(manifest_path)?;
		}

		Ok(())
	}

	fn configure_mod(&self, game: &InstalledGame, _local_mod: &LocalMod) -> Result {
		let game_data_folder = game.get_installed_mods_folder()?;
		let mod_config_path = game_data_folder.join("BepInEx").join("config");

		// TODO: actually open the specific config file somehow. Probably needs to be in the remote mod manifest.

		Ok(open::that_detached(mod_config_path)?)
	}

	async fn run_without_game(&self, local_mod: &LocalMod) -> Result {
		Err(Error::CantRunNonRunnable(local_mod.common.id.clone()))
	}

	fn open_installed_mod_folder(&self, game: &InstalledGame, local_mod: &LocalMod) -> Result {
		let game_data_folder = game.get_installed_mods_folder()?;
		let plugin_folder = game_data_folder
			.join("BepInEx")
			.join("plugins")
			.join(&local_mod.common.id);

		Ok(open::that_detached(plugin_folder)?)
	}

	fn get_mod_path(&self, mod_data: &CommonModData) -> Result<PathBuf> {
		mod_data.unity_backend.map_or_else(
			|| Err(Error::UnityBackendUnknown(mod_data.id.clone())),
			|unity_backend| {
				Ok(Self::get_installed_mods_path()?
					.join(unity_backend.to_string())
					.join(&mod_data.id))
			},
		)
	}

	fn get_local_mods(&self) -> Result<HashMap<String, LocalMod>> {
		let installed_mods_path = Self::get_installed_mods_path()?;

		let local_mods = {
			let mut local_mods = find_mods(&installed_mods_path, UnityScriptingBackend::Il2Cpp);
			local_mods.extend(find_mods(&installed_mods_path, UnityScriptingBackend::Mono));
			local_mods
		};

		Ok(local_mods)
	}
}

fn is_legacy(engine: &GameEngine) -> bool {
	engine.version.as_ref().map_or(false, |version| {
		version.numbers.major < 5
			|| (version.numbers.major == 5 && version.numbers.minor.is_some_and(|minor| minor < 5))
	})
}

fn find_mods(
	installed_mods_path: &Path,
	scripting_backend: UnityScriptingBackend,
) -> HashMap<String, LocalMod> {
	let mods_folder_path = installed_mods_path.join(scripting_backend.to_string());

	paths::glob_path(&mods_folder_path.join("*"))
		.iter()
		.filter_map(|mod_path| {
			if let Ok(local_mod) = LocalMod::new(
				BepInEx::ID,
				mod_path,
				Some(EngineBrand::Unity),
				Some(scripting_backend),
			) {
				Some((local_mod.common.id.clone(), local_mod))
			} else {
				None
			}
		})
		.collect()
}
