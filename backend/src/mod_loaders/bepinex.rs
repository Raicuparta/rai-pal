use std::{
	fs,
	path::Path,
};

use super::mod_loader::ModLoaderStatic;
use crate::{
	files::{
		copy_dir_all,
		unzip,
	},
	game::{
		Game,
		OperatingSystem,
	},
	game_engines::{
		game_engine::{
			GameEngine,
			GameEngineBrand,
		},
		unity::UnityScriptingBackend,
	},
	game_mod::Mod,
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
});

impl ModLoaderStatic for BepInEx {
	const ID: &'static str = "bepinex";

	fn new(resources_path: &Path) -> Result<Self> {
		let path = resources_path.join(Self::ID);

		let mods = {
			let mut mods = find_mods(&path, UnityScriptingBackend::Il2Cpp)?;
			mods.append(&mut find_mods(&path, UnityScriptingBackend::Mono)?);
			mods
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

impl ModLoaderActions for BepInEx {
	fn get_data(&self) -> &ModLoaderData {
		&self.data
	}

	fn install(&self, game: &Game) -> Result {
		let scripting_backend_path = &self.data.path.join(
			game.scripting_backend
				.ok_or_else(|| {
					Error::ModInstallInfoInsufficient(
						"scripting_backend".to_string(),
						game.full_path.clone(),
					)
				})?
				.to_string(),
		);
		let architecture_path = scripting_backend_path
			.join(
				game.operating_system
					.ok_or_else(|| {
						Error::ModInstallInfoInsufficient(
							"operating_system".to_string(),
							game.full_path.clone(),
						)
					})?
					.to_string(),
			)
			.join(
				game.architecture
					.ok_or_else(|| {
						Error::ModInstallInfoInsufficient(
							"architecture".to_string(),
							game.full_path.clone(),
						)
					})?
					.to_string(),
			);

		let mod_loader_archive = architecture_path.join("mod-loader.zip");
		let folder_to_copy_to_game = architecture_path.join("copy-to-game");
		let game_data_folder = &game.get_installed_mods_folder()?;

		unzip(&mod_loader_archive, game_data_folder)?;

		let game_folder = paths::path_parent(&game.full_path)?;

		copy_dir_all(folder_to_copy_to_game, game_folder)?;

		let is_legacy = game.engine.as_ref().map_or(false, is_legacy);

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

		if let Some(operating_system) = game.operating_system {
			if std::env::consts::OS != "windows" && operating_system == OperatingSystem::Windows {
				if let Some(steam_launch) = &game.steam_launch {
					ensure_wine_will_load_bepinex(&game.full_path, steam_launch.app_id)?;
				}
			}
		}

		Ok(())
	}

	fn install_mod(&self, game: &Game, mod_id: &str) -> Result {
		let game_mod = self
			.data
			.mods
			.iter()
			.find(|game_mod| game_mod.id == mod_id)
			.ok_or_else(|| Error::ModNotFound(mod_id.to_string()))?;

		self.install(game)?;
		game_mod.install(
			&game
				.get_installed_mods_folder()?
				.join("BepInEx")
				.join("plugins"),
		)
	}

	fn open_mod_folder(&self, mod_id: &str) -> Result {
		let game_mod = self
			.data
			.mods
			.iter()
			.find(|game_mod| game_mod.id == mod_id)
			.ok_or_else(|| Error::ModNotFound(mod_id.to_string()))?;

		game_mod.open_folder()
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

fn find_mods(mod_loader_path: &Path, scripting_backend: UnityScriptingBackend) -> Result<Vec<Mod>> {
	let mods_folder_path = mod_loader_path
		.join(scripting_backend.to_string())
		.join("mods");

	let entries: Vec<_> = paths::glob_path(&mods_folder_path.join("*"))?.collect();

	Ok(entries
		.iter()
		.filter_map(|entry| match entry {
			Ok(mod_path) => Some(
				Mod::new(
					mod_path,
					Some(GameEngineBrand::Unity),
					Some(scripting_backend),
				)
				.ok()?,
			),
			Err(_) => None,
		})
		.collect())
}
