use std::{
	collections::BTreeMap,
	fs,
	hash::{
		DefaultHasher,
		Hash,
		Hasher,
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
	game_providers::game_provider,
	http::{
		self,
		DownloadStatus,
	},
	mods::{
		mod_config::ModConfig,
		replacement_token::replace_tokens,
	},
	operating_system::OperatingSystem,
	path_extensions::PathExt,
	result::{
		Error,
		Result,
	},
};

#[serializable_struct]
pub struct GameMod {
	pub id: String,
	pub scope: Option<String>,
	pub family: Option<String>,
	pub title: String,
	pub hide_from_game_mods_list: Option<bool>,
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
	pub run_for_game: Option<ModRun>,
	pub run_standalone: Option<ModRun>,
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
	pub manifest_path: String,
	pub extract: Option<Vec<ModInstallExtract>>,
	pub write: Option<Vec<ModInstallWrite>>,
	pub wine_dll_overrides: Option<Vec<String>>,
	pub main_installed_folder_path: Option<String>,
}

#[serializable_struct]
pub struct ModRun {
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

	pub fn get_manifest_target_path(&self, game: Option<&DbGame>) -> Result<PathBuf> {
		Ok(PathBuf::from(replace_tokens(
			&self.get_install()?.manifest_path,
			game,
			self,
		)))
	}

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
		game_option: Option<&DbGame>,
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

			let source_dir = download.download(on_download_status).await?;

			for extract_action in extract_actions {
				let source_path = source_dir.join(&extract_action.source);
				let destination_path = PathBuf::from(replace_tokens(
					&extract_action.destination,
					game_option,
					self,
				));

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
					PathBuf::from(replace_tokens(&write_action.destination, game_option, self));
				let content = replace_tokens(&write_action.content, game_option, self);

				fs::create_dir_all(destination_path.try_parent()?)?;
				fs::write(destination_path, content)?;
			}
		}

		if let Some(wine_dll_overrides) = install.wine_dll_overrides.as_ref() {
			let game = game_option.ok_or_else(Error::GameNeeded)?;
			game_provider::get_provider(game.provider_id)?
				.set_wine_dll_overrides(game, wine_dll_overrides)?;
		}

		self.update_installed_mod_manifest(game_option)?;

		Ok(())
	}

	pub fn run(&self, game_option: Option<&DbGame>) -> Result {
		let mod_run = if game_option.is_some() {
			self.run_for_game
				.as_ref()
				.ok_or_else(|| Error::ModInfoMissing(self.id.clone(), "run_for_game".to_string()))?
		} else {
			self.run_standalone.as_ref().ok_or_else(|| {
				Error::ModInfoMissing(self.id.clone(), "run_standalone".to_string())
			})?
		};

		let run_path = PathBuf::from(replace_tokens(
			mod_run.path.as_ref().ok_or_else(|| {
				Error::ModInfoMissing(self.id.clone(), "mod_run.path".to_string())
			})?,
			game_option,
			self,
		));

		if !run_path.try_exists()? {
			return Err(Error::ModNotInstalled(self.id.clone()));
		}

		let args: Vec<String> = mod_run
			.args
			.clone()
			.unwrap_or_default()
			.iter()
			.map(|arg| replace_tokens(arg, game_option, self))
			.collect();

		#[cfg(target_os = "linux")]
		{
			let game = game_option.ok_or_else(Error::GameNeeded)?;

			let wine_environment: BTreeMap<String, String> = mod_run
				.wine_environment
				.clone()
				.unwrap_or_default()
				.iter()
				.map(|(key, value)| (key.clone(), replace_tokens(value, game_option, self)))
				.collect();

			game_provider::get_provider(game.provider_id)?.run_with_wine(
				game,
				&run_path,
				&args,
				&wine_environment,
			)?;
		}

		#[cfg(target_os = "windows")]
		{
			std::process::Command::new(&run_path)
				.current_dir(run_path.try_parent()?)
				.args(&args)
				.spawn()?;
		}

		Ok(())
	}

	pub fn update_installed_mod_manifest(&self, game: Option<&DbGame>) -> Result {
		let manifest_path = self.get_manifest_target_path(game)?;
		fs::create_dir_all(manifest_path.try_parent()?)?;
		let manifest_contents = serde_json::to_string_pretty(&self)?;
		fs::write(manifest_path, manifest_contents)?;

		Ok(())
	}
}

impl ModDownload {
	async fn download(
		&self,
		on_download_status: impl Fn(DownloadStatus) + Send,
	) -> Result<PathBuf> {
		if let Some(local_path) = self.url.strip_prefix("file://") {
			let source_path = PathBuf::from(local_path);

			if source_path.is_dir() {
				return Ok(source_path);
			}

			return Ok(source_path);
		}

		let mut hasher = DefaultHasher::new();
		self.url.hash(&mut hasher);
		let url_hash = hasher.finish().to_string();

		let temp_dir = app_paths::temp_dir(&url_hash)?;
		let extracted_folder = temp_dir.join("extracted");

		let mut attempts = 0;
		loop {
			fs::create_dir_all(&extracted_folder)?;

			let part_path = temp_dir.join("download.zip.part");
			let zip_path = temp_dir.join("download.zip");

			if !zip_path.is_file() {
				fs::remove_file(&part_path).ok();
				http::download(&self.url, &part_path, &on_download_status).await?;
				fs::rename(&part_path, &zip_path)?;
			}

			let extract_result = files::extract(
				&zip_path,
				&extracted_folder,
				&|extracted_bytes, total_uncompressed| {
					on_download_status(DownloadStatus::new(
						self.url.clone(),
						extracted_folder.to_string_lossy().to_string(),
						extracted_bytes as usize,
						Some(total_uncompressed),
					));
				},
			);

			match extract_result {
				Ok(()) => return Ok(extracted_folder),
				Err(err) => {
					fs::remove_dir_all(&temp_dir).ok();
					if attempts >= 1 {
						return Err(err.into());
					}
					attempts += 1;
				}
			}
		}
	}
}
