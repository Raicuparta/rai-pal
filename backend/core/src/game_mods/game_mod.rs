use std::{
	collections::BTreeMap,
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
use uuid::Uuid;
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
	pub download: Option<ModDownload>,
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

	pub fn get_install(&self) -> Result<&ModInstall> {
		self.install
			.as_ref()
			.ok_or_else(|| Error::ModInfoMissing(self.id.clone(), "install".to_string()))
	}

	pub async fn install(
		&self,
		game: &DbGame,
		on_download_status: impl Fn(DownloadStatus) + Send,
	) -> Result {
		let install = self
			.install
			.as_ref()
			.ok_or_else(|| Error::ModInfoMissing(self.id.clone(), "install".to_string()))?;

		if let Some(extract_actions) = install.extract.as_ref() {
			let download = self
				.download
				.as_ref()
				.ok_or_else(|| Error::ModInfoMissing(self.id.clone(), "download".to_string()))?;
			let source_dir = download
				.download_to_temp(&self.id, on_download_status)
				.await?;

			for extract_action in extract_actions {
				let source_path = source_dir.join(&extract_action.source);
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

	// TODO: can only run if installed.
	pub fn run(&self, game: &DbGame) -> Result {
		let run_for_game = self
			.run_for_game
			.as_ref()
			.ok_or_else(|| Error::ModInfoMissing(self.id.clone(), "run_for_game".to_string()))?;

		// TODO check this path, probably needs changing in DB to be absolute.
		let run_path = PathBuf::from(replace_tokens(
			run_for_game.path.as_ref().ok_or_else(|| {
				Error::ModInfoMissing(self.id.clone(), "run_for_game.path".to_string())
			})?,
			game,
			self,
		));

		if !run_path.try_exists()? {
			return Err(Error::ModNotInstalled(self.id.clone()));
		}

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
				.current_dir(source_dir)
				.args(&args)
				.spawn()?;
		}

		Ok(())
	}

	pub fn update_installed_mod_manifest(&self, game: &DbGame) -> Result {
		let manifest_path = game.get_installed_mod_manifest_path(&self.id)?;
		fs::create_dir_all(manifest_path.try_parent()?)?;
		let manifest_contents = serde_json::to_string_pretty(&self)?;
		fs::write(manifest_path, manifest_contents)?;

		Ok(())
	}
}

impl ModDownload {
	async fn download_to_temp(
		&self,
		mod_id: &str,
		on_download_status: impl Fn(DownloadStatus) + Send,
	) -> Result<PathBuf> {
		let temp_dir = app_paths::temp_dir(&format!("mod-{mod_id}"))?;
		// TODO cache should be per version.
		let zip_path = temp_dir.join(format!("{mod_id}.zip"));

		http::download(&self.url, &zip_path, on_download_status).await?;

		let file = File::open(&zip_path)?;
		let mut zip_archive = ZipArchive::new(file)?;

		files::extract(&mut zip_archive, &temp_dir)?;

		fs::remove_file(&zip_path)?;

		Ok(temp_dir)
	}
}
