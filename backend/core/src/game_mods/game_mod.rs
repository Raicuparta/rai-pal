use std::{
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

use rai_pal_proc_macros::{
	serializable_enum,
	serializable_struct,
};
use zip::ZipArchive;

use crate::{
	architecture::Architecture,
	files,
	game::DbGame,
	game_engines::{
		game_engine::{
			EngineBrand,
			EngineVersionRange,
		},
		unity::UnityBackend,
	},
	game_mods::{
		mod_config::ModConfig,
		mod_database::ModDatabase,
	},
	operating_system::OperatingSystem,
	paths,
	result::{
		Error,
		LogErrExt,
		Result,
	},
};

#[serializable_struct]
pub struct GameMod {
	pub id: String,
	pub title: String,
	pub author: String,
	pub source_code: String,
	pub description: String,
	pub latest_version: ModDownload,
	pub engine: Option<EngineBrand>,
	pub engine_version_range: Option<EngineVersionRange>,
	pub unity_backend: Option<UnityBackend>,
	pub architecture: Option<Architecture>,
	pub game_os: Option<OperatingSystem>,
	pub host_os: Option<OperatingSystem>,
	pub deprecated: Option<bool>,
	pub config: Option<ModConfig>,
	pub dependencies: Option<Vec<ModDependency>>,
	pub install: Option<ModInstall>,
	pub run_for_game: Option<ModRunForGame>,
}

#[serializable_struct]
pub struct ModInstallExtract {
	pub source: String,
	pub destination: String,
}

#[serializable_struct]
pub struct ModInstallWrite {
	pub content: String,
	pub destination: String,
}

#[serializable_struct]
pub struct ModInstall {
	pub extract: Option<Vec<ModInstallExtract>>,
	pub write: Option<Vec<ModInstallWrite>>,
	pub wine_dll_overrides: Option<Vec<String>>,
	pub main_installed_folder_path: Option<String>,
}

#[serializable_struct]
pub struct ModRunForGame {
	pub path: Option<String>,
	pub args: Option<Vec<String>>,
	pub wine_environment: Option<HashMap<String, String>>,
}

#[serializable_struct]
pub struct ModDownload {
	pub id: String,
	pub url: String,
}

#[serializable_struct]
pub struct ModDependency {
	pub mod_id: String,
}

#[serializable_enum]
pub enum ModConfigDestinationType {
	File,
	Folder,
}

impl GameMod {
	pub const FILE_NAME: &'static str = "rai-pal-manifest.json";

	pub fn from_file(path: &Path) -> Option<Self> {
		match fs::read_to_string(path)
			.and_then(|manifest_bytes| Ok(serde_json::from_str::<Self>(&manifest_bytes)?))
		{
			Ok(manifest) => Some(manifest),
			Err(error) => {
				log::error!(
					"Error getting manifest in path '{}': {}",
					path.display(),
					error
				);
				None
			}
		}
	}
	pub fn open_folder(&self) -> Result {
		paths::open_folder_or_parent(&self.get_local_folder_path()?)
	}

	pub async fn install(&self, game: &DbGame) -> Result {
		todo!();
	}

	pub async fn run(&self, game: &DbGame) -> Result {
		todo!();
	}

	pub async fn uninstall(&self, game: &DbGame) -> Result {
		todo!();
	}

	pub async fn run_without_game(&self) -> Result {
		todo!();
	}

	pub fn update_installed_mod_manifest(&self, game: &DbGame) -> Result {
		// TODO: make sure it doesn't happen for runnables.
		let manifest_path = game.get_installed_mod_manifest_path(&self.id)?;
		fs::create_dir_all(paths::path_parent(&manifest_path)?)?;
		let manifest_contents = serde_json::to_string_pretty(&self)?;
		fs::write(manifest_path, manifest_contents)?;

		Ok(())
	}

	pub fn get_config_path(&self, config: &ModConfig, _game: &DbGame) -> Result<PathBuf> {
		// TODO: handle tokens and game.
		Ok(PathBuf::from(&config.destination_path))
	}

	pub fn configure_mod(&self, game: &DbGame, open_folder: bool) -> Result {
		if let Some(config) = self.config.as_ref() {
			let config_path = self.get_config_path(config, game)?;
			if open_folder {
				paths::open_folder_or_parent(&config_path)?;
			} else {
				open::that_detached(config_path)?;
			}
		}

		Ok(())
	}

	pub fn open_installed_mod_folder(&self, _game: &DbGame) -> Result {
		todo!();
	}

	pub fn delete_local(&self) -> Result {
		let path = self.get_local_folder_path()?;
		if path.exists() {
			fs::remove_dir_all(&path)?;
		}

		Ok(())
	}

	pub fn get_local_folder_path(&self) -> Result<PathBuf> {
		Ok(paths::local_mods_path()?.join(&self.id))
	}

	pub fn get_local_manifest_path(&self) -> Result<PathBuf> {
		Ok(Self::get_manifest_path(&self.get_local_folder_path()?))
	}

	pub async fn download(&self) -> Result {
		let target_path = paths::local_mods_path()?.join(&self.id);
		let downloads_path = paths::downloads_path()?;
		let mod_id = &self.id;

		let response = reqwest::get(&self.latest_version.url).await?;

		fs::create_dir_all(&downloads_path)?;

		let zip_path = downloads_path.join(format!("{mod_id}.zip"));

		// TODO Stream to disk instead of keeping it all in memory.
		fs::write(&zip_path, response.bytes().await?)?;
		let file = File::open(&zip_path)?;

		let mut zip_archive = ZipArchive::new(file)?;

		files::extract(&mut zip_archive, &target_path)?;

		fs::write(
			Self::get_manifest_path(&target_path),
			serde_json::to_string_pretty(&self)?,
		)?;

		Ok(())
	}

	pub async fn get_all_remote<F>(error_handler: F) -> HashMap<String, Self>
	where
		F: Fn(Error) + Send,
	{
		let database = ModDatabase::get().await.unwrap_or_else(|error| {
			error_handler(error);
			ModDatabase { mods: Vec::new() }
		});

		let mut mods_map = HashMap::new();
		let local_mods = Self::get_all_local().ok_or_log("Failed to get local mods");

		for remote_mod in database.mods {
			// If there's a local mod with the same ID, update its manifest with remote info
			if let Some(local_mod) = local_mods
				.as_ref()
				.and_then(|local_mods| local_mods.get(&remote_mod.id))
				&& let Some(manifest_path) = local_mod
					.get_local_manifest_path()
					.ok_or_log("Failed to get manifest path for local mod.")
			{
				// Only update if the manifest file exists (mod has been downloaded before)
				if manifest_path.exists()
					&& let Ok(manifest_contents) = serde_json::to_string_pretty(&remote_mod)
				{
					// TODO what's going on with this result?
					let _ = fs::write(&manifest_path, manifest_contents);
				}
			}

			mods_map.insert(remote_mod.id.clone(), remote_mod);
		}

		mods_map
	}

	pub fn get_all_local() -> Result<HashMap<String, Self>> {
		Ok(
			paths::glob_path(&paths::local_mods_path()?.join("*").join(Self::FILE_NAME))
				.iter()
				.filter_map(|manifest_path| {
					Self::from_file(manifest_path)
						.map(|local_mod| (local_mod.id.clone(), local_mod))
				})
				.collect(),
		)
	}

	fn get_manifest_path(target_path: &Path) -> PathBuf {
		target_path.join(Self::FILE_NAME)
	}
}
