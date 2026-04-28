use std::{
	collections::HashMap,
	fs,
	io::Cursor,
	path::{
		Path,
		PathBuf,
	},
};

use rai_pal_proc_macros::serializable_struct;
use serde::Deserialize;
use zip::ZipArchive;

use super::mod_loader::ModLoaderStatic;
use crate::{
	architecture::Architecture,
	files::copy_dir_all,
	game::DbGame,
	game_engines::{
		game_engine::EngineBrand,
		unity::UnityBackend,
	},
	game_mod::CommonModData,
	http,
	local_mod::{
		LocalMod,
		ModKind,
	},
	mod_loaders::{
		mod_database::ModConfigs,
		mod_loader::{
			ModLoaderActions,
			ModLoaderData,
		},
	},
	paths,
	result::{
		Error,
		Result,
	},
};

const BEPINEX_DB_URL: &str = "https://raicuparta.github.io/rai-pal-db/loader-db/0/bepinex.json";

#[derive(Deserialize)]
struct BepInExBuild {
	arch: String,
	os: String,
	backend: String,
	#[serde(rename = "downloadUrl")]
	download_url: String,
}

#[derive(Deserialize)]
struct BepInExEntry {
	version: String,
	builds: Vec<BepInExBuild>,
}

async fn get_download_url(
	unity_backend: UnityBackend,
	architecture: Architecture,
) -> Result<String> {
	let entries: Vec<BepInExEntry> = http::CLIENT
		.get(BEPINEX_DB_URL)
		.send()
		.await?
		.json()
		.await?;

	let major_version = match unity_backend {
		UnityBackend::Il2Cpp => "6",
		UnityBackend::Mono => "5",
	};

	let backend_str = match unity_backend {
		UnityBackend::Il2Cpp => "IL2CPP",
		UnityBackend::Mono => "Mono",
	};

	let arch_str = match architecture {
		Architecture::X64 => "x64",
		Architecture::X86 => "x86",
	};

	entries
		.iter()
		.find(|entry| entry.version.starts_with(&format!("{major_version}.")))
		.and_then(|entry| {
			entry.builds.iter().find(|build| {
				build.os == "win" && build.backend == backend_str && build.arch == arch_str
			})
		})
		.map(|build| build.download_url.clone())
		.ok_or_else(|| {
			Error::ModInstallInfoInsufficient(
				format!("bepinex_{backend_str}_{arch_str}"),
				String::new(),
			)
		})
}

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

impl ModLoaderActions for BepInEx {
	fn get_data(&self) -> &ModLoaderData {
		&self.data
	}

	async fn install(&self, game: &DbGame) -> Result {
		let exe_path = game.try_get_exe_path()?;
		let unity_backend = game.unity_backend.ok_or_else(|| {
			Error::ModInstallInfoInsufficient(
				"unity_backend".to_string(),
				game.display_title.clone(),
			)
		})?;
		let architecture = game.architecture.ok_or_else(|| {
			Error::ModInstallInfoInsufficient(
				"architecture".to_string(),
				game.display_title.clone(),
			)
		})?;

		let unity_backend_path = &self.data.path.join(unity_backend.to_string());

		let download_url = get_download_url(unity_backend, architecture).await?;
		let zip_bytes = http::CLIENT
			.get(&download_url)
			.send()
			.await?
			.bytes()
			.await?;

		let game_mods_folder = game.get_installed_mods_folder()?;
		fs::create_dir_all(&game_mods_folder)?;
		ZipArchive::new(Cursor::new(zip_bytes))?.extract(&game_mods_folder)?;

		let game_folder = paths::path_parent(exe_path)?;

		fs::copy(
			game_mods_folder.join("winhttp.dll"),
			game_folder.join("winhttp.dll"),
		)?;

		let config_origin_path = &self.data.path.join("config").join(if is_legacy(game) {
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
		self.install(game).await?;

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

	async fn uninstall_mod(&self, game: &DbGame, local_mod: &LocalMod) -> Result {
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
			let mut local_mods = find_mods(&installed_mods_path, UnityBackend::Il2Cpp);
			local_mods.extend(find_mods(&installed_mods_path, UnityBackend::Mono));
			local_mods
		};

		Ok(local_mods)
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

fn find_mods(installed_mods_path: &Path, unity_backend: UnityBackend) -> HashMap<String, LocalMod> {
	let mods_folder_path = installed_mods_path.join(unity_backend.to_string());

	paths::glob_path(&mods_folder_path.join("*"))
		.iter()
		.filter_map(|mod_path| {
			if let Ok(local_mod) = LocalMod::new(
				BepInEx::ID,
				mod_path,
				Some(EngineBrand::Unity),
				Some(unity_backend),
			) {
				Some((local_mod.common.id.clone(), local_mod))
			} else {
				None
			}
		})
		.collect()
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
