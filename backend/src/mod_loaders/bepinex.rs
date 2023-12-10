use std::{
	borrow::BorrowMut,
	collections::HashMap,
	fs::{
		self,
		File,
	},
	path::{
		Path,
		PathBuf,
	},
};

use async_trait::async_trait;
use zip::ZipArchive;

use super::mod_loader::ModLoaderStatic;
use crate::{
	files::copy_dir_all,
	game_engines::{
		game_engine::{
			GameEngine,
			GameEngineBrand,
		},
		unity::UnityScriptingBackend,
	},
	game_executable::OperatingSystem,
	game_mod::GameMod,
	installed_game::InstalledGame,
	local_mod::{
		LocalMod,
		ModKind,
	},
	mod_loaders::mod_loader::{
		ModLoaderActions,
		ModLoaderData,
	},
	paths,
	serializable_struct,
	Error,
	Result,
};

serializable_struct!(BepInEx {
	pub data: ModLoaderData,
	pub id: &'static str,
});

#[async_trait]
impl ModLoaderStatic for BepInEx {
	const ID: &'static str = "bepinex";

	async fn new(resources_path: &Path) -> Result<Self> {
		let path = resources_path.join(Self::ID);

		Ok(Self {
			id: Self::ID,
			data: ModLoaderData {
				id: Self::ID.to_string(),
				mods: HashMap::default(),
				path,
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

	fn get_data_mut(&mut self) -> &mut ModLoaderData {
		self.data.borrow_mut()
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
				game.executable
					.operating_system
					.ok_or_else(|| {
						Error::ModInstallInfoInsufficient(
							"operating_system".to_string(),
							game.executable.path.clone(),
						)
					})?
					.to_string(),
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
			doorstop_config.replace("{{MOD_FILES_PATH}}", paths::path_to_str(game_data_folder)?),
		)?;

		if let Some(operating_system) = game.executable.operating_system {
			if std::env::consts::OS != "windows" && operating_system == OperatingSystem::Windows {
				if let Some(steam_launch) = &game.steam_launch {
					ensure_wine_will_load_bepinex(&game.executable.path, steam_launch.app_id)?;
				}
			}
		}

		Ok(())
	}

	async fn install_mod(&self, game: &InstalledGame, mod_id: &str) -> Result {
		let game_mod = self
			.data
			.mods
			.get(mod_id)
			.ok_or_else(|| Error::ModNotFound(mod_id.to_string()))?;

		self.install(game)?;

		let bepinex_folder = game.get_installed_mods_folder()?.join("BepInEx");

		let mod_path = if let Some(local_mod) = &game_mod.local_mod {
			local_mod.path.clone()
		} else {
			self.download_mod(mod_id).await?;
			self.get_mod_path(mod_id)?
		};

		let mod_plugin_path = mod_path.join("plugins");
		if mod_plugin_path.is_dir() {
			copy_dir_all(mod_plugin_path, bepinex_folder.join("plugins").join(mod_id))?;
		}

		let mod_patch_path = mod_path.join("patchers");
		if mod_patch_path.is_dir() {
			copy_dir_all(mod_patch_path, bepinex_folder.join("patchers").join(mod_id))?;
		}

		Ok(())
	}

	fn open_mod_folder(&self, mod_id: &str) -> Result {
		let game_mod = self
			.data
			.mods
			.get(mod_id)
			.ok_or_else(|| Error::ModNotFound(mod_id.to_string()))?;

		game_mod.open_folder()
	}

	fn get_mod_path(&self, mod_id: &str) -> Result<PathBuf> {
		let game_mod = self
			.data
			.mods
			.get(mod_id)
			.ok_or_else(|| Error::ModNotFound(mod_id.to_string()))?;

		game_mod.common.unity_backend.map_or_else(
			|| Err(Error::ModNotFound(mod_id.to_string())),
			|unity_backend| {
				Ok(self
					.get_installed_mods_path()?
					.join(unity_backend.to_string())
					.join("mods")
					.join(mod_id))
			},
		)
	}

	fn update_local_mods(&mut self) -> Result {
		let data = self.get_data_mut();

		let local_mods = {
			let mut local_mods = find_mods(&data.path, UnityScriptingBackend::Il2Cpp)?;
			local_mods.extend(find_mods(&data.path, UnityScriptingBackend::Mono)?);
			local_mods
		};

		let mods: HashMap<_, _> = local_mods
			.into_iter()
			.map(|(mod_id, local_mod)| {
				(
					mod_id,
					GameMod {
						remote_mod: None,
						local_mod: Some(local_mod.data),
						common: local_mod.common,
					},
				)
			})
			.collect();

		self.get_data_mut().mods = mods;

		Ok(())
	}
}

const fn is_legacy(engine: &GameEngine) -> bool {
	if let Some(version) = &engine.version {
		version.major < 5 || (version.major == 5 && version.minor < 5)
	} else {
		false
	}
}

fn ensure_wine_will_load_bepinex(game_path: &Path, steam_app_id: u32) -> Result {
	let steam_apps_folder = {
		// I think since canonicalize gets rid of symbolic links then it shouldn't be possible to
		// get stuck in an infinite loop, but who knows. If you're currently stuck in
		// an infinite loop then I'm very sorry, hope infinity goes well for you.
		let mut steam_apps_folder = game_path.canonicalize()?;

		loop {
			if steam_apps_folder.ends_with("steamapps") {
				break;
			}

			steam_apps_folder = paths::path_parent(&steam_apps_folder)?.to_path_buf();
		}
		steam_apps_folder
	};

	let compat_data_dir = steam_apps_folder
		.join("compatdata")
		.join(steam_app_id.to_string());
	let pfx_folder = compat_data_dir.join("pfx");
	let user_reg = pfx_folder.join("user.reg");

	let user_reg_data = fs::read_to_string(&user_reg)?;

	let ensured_user_reg_data = reg_add_in_section(
		&user_reg_data,
		"[Software\\\\Wine\\\\DllOverrides]",
		"winhttp",
		"native,builtin",
	);

	if user_reg_data != ensured_user_reg_data {
		fs::copy(&user_reg, pfx_folder.join("user.reg.bak"))?;
		fs::write(&user_reg, ensured_user_reg_data)?;
	}

	Ok(())
}

fn reg_add_in_section(reg: &str, section: &str, key: &str, value: &str) -> String {
	let mut split = reg.split('\n').collect::<Vec<_>>();

	let mut begin = 0;

	for (index, line) in split.iter().enumerate() {
		if line.starts_with(section) {
			begin = index + 2;
			break;
		}
	}

	let mut end = 0;

	for (index, line) in split.iter().enumerate().skip(begin) {
		if line.is_empty() {
			end = index;
			break;
		}
	}

	let line_start = &format!("\"{key}\"");
	let new_line = format!("{line_start}=\"{value}\"");

	for (_, line) in split.iter_mut().enumerate().skip(begin) {
		if line.starts_with(line_start) {
			*line = &new_line;
			return split.join("\n");
		}
	}

	split.insert(end, &new_line);

	split.join("\n")
}

fn find_mods(
	mod_loader_path: &Path,
	scripting_backend: UnityScriptingBackend,
) -> Result<HashMap<String, LocalMod>> {
	let mods_folder_path = mod_loader_path
		.join(scripting_backend.to_string())
		.join("mods");

	let entries: Vec<_> = paths::glob_path(&mods_folder_path.join("*"))?.collect();

	Ok(entries
		.iter()
		.filter_map(|entry| {
			entry.as_ref().map_or(None, |mod_path| {
				if let Ok(local_mod) = LocalMod::new(
					mod_path,
					Some(GameEngineBrand::Unity),
					Some(scripting_backend),
				) {
					Some((local_mod.common.id.clone(), local_mod))
				} else {
					None
				}
			})
		})
		.collect())
}
