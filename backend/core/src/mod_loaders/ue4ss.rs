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
use zip::ZipArchive;

use super::mod_loader::{
	LoaderDatabase,
	ModLoaderActions,
	ModLoaderData,
	ModLoaderId,
	ModLoaderStatic,
	ModLoaderStatus,
};
use crate::{
	game::DbGame,
	game_engines::game_engine::EngineBrand,
	game_mod::CommonModData,
	http,
	local_mod::{
		LocalMod,
		ModKind,
	},
	mod_loaders::mod_database::ModConfigs,
	mod_manifest,
	paths,
	result::{
		Error,
		Result,
	},
};

const UE4SS_DB_URL: &str = "https://raicuparta.github.io/rai-pal-db/loader-db/1/ue4ss.json";

struct Ue4ssVersionData {
	version: String,
	download_url: String,
}

async fn get_version_data() -> Result<Ue4ssVersionData> {
	let database: LoaderDatabase = http::CLIENT.get(UE4SS_DB_URL).send().await?.json().await?;

	database
		.releases
		.iter()
		.find(|release| release.builds.iter().any(|build| build.os == "win"))
		.and_then(|release| {
			release
				.builds
				.iter()
				.find(|build| build.os == "win")
				.map(|build| Ue4ssVersionData {
					version: release.version.clone(),
					download_url: build.download_url.clone(),
				})
		})
		.ok_or_else(|| Error::ModInstallInfoInsufficient("ue4ss_win".to_string(), String::new()))
}

fn get_installed_version(game: &DbGame) -> Option<String> {
	Ue4ss::get_installed_loader_version(game)
}

fn update_installed_manifest(game: &DbGame, version: String) -> Result {
	Ue4ss::update_installed_loader_manifest(
		game,
		&mod_manifest::Manifest {
			title: Some("UE4SS".to_string()),
			is_loader: Some(true),
			version,
			runnable: None,
			engine: Some(EngineBrand::Unreal),
			engine_version_range: None,
			architecture: None,
			unity_backend: None,
			configs: None,
		},
	)
}

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

	async fn get_status(&self, game: &DbGame) -> Result<Option<ModLoaderStatus>> {
		if game.exe_path.is_none() || game.engine_brand != Some(EngineBrand::Unreal) {
			return Ok(None);
		}

		let installed_version = get_installed_version(game);

		let latest_version = match get_version_data().await {
			Ok(version_data) => Some(version_data.version),
			Err(error) => {
				log::error!(
					"Failed to get latest UE4SS version for game '{}': {}",
					game.display_title,
					error
				);
				None
			}
		};

		Ok(Some(ModLoaderStatus {
			installed_version,
			latest_version,
		}))
	}

	async fn install(&self, game: &DbGame) -> Result {
		let exe_path = game.try_get_exe_path()?;

		let version_data = get_version_data().await?;
		let zip_bytes = http::CLIENT
			.get(&version_data.download_url)
			.send()
			.await?
			.bytes()
			.await?;

		let game_mods_folder = game.get_installed_mods_folder()?;
		fs::create_dir_all(&game_mods_folder)?;
		ZipArchive::new(Cursor::new(zip_bytes))?.extract(&game_mods_folder)?;

		let game_folder = paths::path_parent(exe_path)?;
		fs::create_dir_all(game_folder)?;

		fs::copy(
			game_mods_folder.join("dwmapi.dll"),
			game_folder.join("dwmapi.dll"),
		)?;

		let ue4ss_path = game_mods_folder.join("ue4ss").join("UE4SS");
		fs::write(
			game_folder.join("override.txt"),
			ue4ss_path.to_string_lossy().as_ref(),
		)?;

		update_installed_manifest(game, version_data.version)?;

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

	fn get_mod_path(&self, mod_data: &CommonModData) -> Result<PathBuf> {
		Ok(Self::get_installed_mods_path()?.join(&mod_data.id))
	}

	fn get_local_mods(&self) -> Result<HashMap<String, LocalMod>> {
		Ok(HashMap::new())
	}

	fn get_config_path(&self, game: &DbGame, mod_configs: &ModConfigs) -> Result<PathBuf> {
		Ok(game
			.get_installed_mods_folder()?
			.join("ue4ss")
			.join("UE4SS")
			.join(&mod_configs.destination_path))
	}
}
