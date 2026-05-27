use std::{
	collections::{
		BTreeMap,
		HashMap,
	},
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
	app_paths,
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
		replacement_token::replace_tokens,
	},
	http::{
		self,
		DownloadStatus,
	},
	operating_system::OperatingSystem,
	path_extensions::PathExt,
	providers::provider,
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
	pub latest_version: Option<ModDownload>,
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
	pub hash: Option<String>,
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
	pub wine_environment: Option<BTreeMap<String, String>>,
	pub os: Option<OperatingSystem>,
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

	pub fn open_local_folder(&self) -> Result {
		self.get_local_folder_path()?.open_folder_or_parent()
	}

	pub fn get_install(&self) -> Result<&ModInstall> {
		self.install
			.as_ref()
			.ok_or_else(|| Error::ModInfoMissing(self.id.clone(), "install".to_string()))
	}

	pub fn install(&self, game: &DbGame) -> Result {
		let install = self.get_install()?;
		let local_mod_path = self.get_local_folder_path()?;

		if let Some(extract_actions) = install.extract.as_ref() {
			for extract_action in extract_actions {
				let source_path = local_mod_path.join(&extract_action.source);
				let destination_path =
					PathBuf::from(replace_tokens(&extract_action.destination, game, self));

				if source_path.is_dir() {
					files::copy_dir_all(&source_path, &destination_path)?;
				} else {
					fs::create_dir_all(destination_path.try_parent()?)?;
					fs::copy(&source_path, &destination_path)?;
				}
			}
		}

		if let Some(write_actions) = install.write.as_ref() {
			for write_action in write_actions {
				let destination_path =
					PathBuf::from(replace_tokens(&write_action.destination, game, self));
				let content = replace_tokens(&write_action.content, game, self);

				fs::create_dir_all(destination_path.try_parent()?)?;
				fs::write(destination_path, content)?;
			}
		}

		if let Some(wine_dll_overrides) = install.wine_dll_overrides.as_ref() {
			provider::get_provider(game.provider_id)?
				.set_wine_dll_overrides(game, wine_dll_overrides)?;
		}

		self.update_installed_mod_manifest(game)?;

		Ok(())
	}

	pub fn run(&self, game: &DbGame) -> Result {
		let run_for_game = self
			.run_for_game
			.as_ref()
			.ok_or_else(|| Error::ModInfoMissing(self.id.clone(), "run_for_game".to_string()))?;
		let local_mod_path = self.get_local_folder_path()?;

		let run_path = local_mod_path.join(PathBuf::from(replace_tokens(
			run_for_game.path.as_ref().ok_or_else(|| {
				Error::ModInfoMissing(self.id.clone(), "run_for_game.path".to_string())
			})?,
			game,
			self,
		)));
		let args: Vec<String> = run_for_game
			.args
			.clone()
			.unwrap_or_default()
			.iter()
			.map(|arg| replace_tokens(arg, game, self))
			.collect();

		#[cfg(target_os = "linux")]
		{
			let wine_environment: BTreeMap<String, String> = run_for_game
				.wine_environment
				.clone()
				.unwrap_or_default()
				.iter()
				.map(|(key, value)| (key.clone(), replace_tokens(value, game, self)))
				.collect();

			provider::get_provider(game.provider_id)?.run_with_wine(
				game,
				&run_path,
				&args,
				&wine_environment,
			)?;
		}

		#[cfg(target_os = "windows")]
		{
			std::process::Command::new(&run_path)
				.current_dir(&local_mod_path)
				.args(&args)
				.spawn()?;
		}

		Ok(())
	}

	pub async fn run_without_game(&self) -> Result {
		todo!();
	}

	pub fn update_installed_mod_manifest(&self, game: &DbGame) -> Result {
		// TODO: make sure it doesn't happen for runnables.
		let manifest_path = game.get_installed_mod_manifest_path(&self.id)?;
		fs::create_dir_all(manifest_path.try_parent()?)?;
		let manifest_contents = serde_json::to_string_pretty(&self)?;
		fs::write(manifest_path, manifest_contents)?;

		Ok(())
	}

	pub fn delete_local(&self) -> Result {
		let path = self.get_local_folder_path()?;
		if path.exists() {
			fs::remove_dir_all(&path)?;
		}

		Ok(())
	}

	pub fn get_local_folder_path(&self) -> Result<PathBuf> {
		Ok(app_paths::local_mods_path()?.join(&self.id))
	}

	pub fn get_local_manifest_path(&self) -> Result<PathBuf> {
		Ok(Self::get_manifest_path(&self.get_local_folder_path()?))
	}

	pub async fn download(&self, status_callback: impl Fn(DownloadStatus) + Send) -> Result {
		if let Some(latest_version) = &self.latest_version {
			let target_path = app_paths::local_mods_path()?.join(&self.id);
			let mod_id = &self.id;

			let zip_path = app_paths::downloads_path()?.join(format!("{mod_id}.zip"));

			http::download(&latest_version.url, &zip_path, status_callback).await?;

			let file = File::open(&zip_path)?;

			let mut zip_archive = ZipArchive::new(file)?;

			files::extract(&mut zip_archive, &target_path)?;

			fs::write(
				Self::get_manifest_path(&target_path),
				serde_json::to_string_pretty(&self)?,
			)?;
		}

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
			// If there's a local mod with the same ID, refresh its manifest if out of sync
			if let Some(local_mod) = local_mods
				.as_ref()
				.and_then(|local_mods| local_mods.get(&remote_mod.id))
				&& let Some(manifest_path) = local_mod
					.get_local_manifest_path()
					.ok_or_log("Failed to get manifest path for local mod.")
			{
				// Only refresh if the manifest file exists (mod has been downloaded before),
				// the latest version ID matches, and the hash differs
				if manifest_path.exists()
					&& local_mod.latest_version.as_ref().map(|v| v.id.clone())
						== remote_mod.latest_version.as_ref().map(|v| v.id.clone())
					&& local_mod.hash != remote_mod.hash
					&& let Ok(manifest_contents) = serde_json::to_string_pretty(&remote_mod)
				{
					let _ = fs::write(&manifest_path, manifest_contents);
				}
			}

			mods_map.insert(remote_mod.id.clone(), remote_mod);
		}

		mods_map
	}

	pub fn get_all_local() -> Result<HashMap<String, Self>> {
		Ok(app_paths::local_mods_path()?
			.join("*")
			.join(Self::FILE_NAME)
			.glob()
			.iter()
			.filter_map(|manifest_path| {
				Self::from_file(manifest_path).map(|local_mod| (local_mod.id.clone(), local_mod))
			})
			.collect())
	}

	fn get_manifest_path(target_path: &Path) -> PathBuf {
		target_path.join(Self::FILE_NAME)
	}
}
