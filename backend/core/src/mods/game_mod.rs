use std::{
	collections::BTreeMap,
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
	},
	mods::{
		mod_config::ModConfig,
		replacement_token::replace_tokens,
	},
	operating_system::OperatingSystem,
	path_extensions::PathExt,
	progress_status::ProgressStatus,
	result::{
		Error,
		LogErrExt,
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
	pub optional_dependencies: Option<Vec<ModDependency>>,
	pub required_dependencies: Option<Vec<ModDependency>>,
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

#[derive(serde::Serialize, serde::Deserialize, specta::Type, Clone, Debug, PartialEq, Eq, Hash)]
#[serde(untagged)]
pub enum ModDependency {
	#[serde(rename_all = "camelCase")]
	ModId { mod_id: String },
	#[serde(rename_all = "camelCase")]
	Family { family: String },
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
		match std::fs::read_to_string(path)
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
		on_progress: impl Fn(ProgressStatus) + Send + Sync,
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

			let source_dir = download.download(&self.id, &on_progress).await?;

			for extract_action in extract_actions {
				let source_path = source_dir.join(&extract_action.source);
				let destination_path = PathBuf::from(replace_tokens(
					&extract_action.destination,
					game_option,
					self,
				));

				if source_path.is_dir() {
					files::copy_dir_all(&source_path, &destination_path).await?;
				} else {
					tokio::fs::create_dir_all(destination_path.try_parent()?).await?;
					tokio::fs::copy(&source_path, &destination_path).await?;
				}
			}
		}

		if let Some(write_actions) = install.write.as_ref() {
			for write_action in write_actions {
				let destination_path =
					PathBuf::from(replace_tokens(&write_action.destination, game_option, self));
				let content = replace_tokens(&write_action.content, game_option, self);

				tokio::fs::create_dir_all(destination_path.try_parent()?).await?;
				tokio::fs::write(destination_path, content).await?;
			}
		}

		if let Some(wine_dll_overrides) = install.wine_dll_overrides.as_ref() {
			let game = game_option.ok_or_else(Error::GameNeeded)?;
			game_provider::get_provider(game.provider_id)?
				.set_wine_dll_overrides(game, wine_dll_overrides)?;
		}

		self.update_installed_mod_manifest(game_option).await?;

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

		log::info!(
			"Running mod `{}` at `{}` with args `{}`",
			self.id,
			run_path.display(),
			args.join(" ")
		);

		#[cfg(target_os = "linux")]
		{
			if mod_run.os == Some(OperatingSystem::Windows) {
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
			} else {
				make_executable(&run_path)?;
				std::process::Command::new(&run_path)
					.current_dir(run_path.try_parent()?)
					.args(&args)
					.spawn()?;
			}
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

	pub async fn update_installed_mod_manifest(&self, game: Option<&DbGame>) -> Result {
		let manifest_path = self.get_manifest_target_path(game)?;
		tokio::fs::create_dir_all(manifest_path.try_parent()?).await?;
		let manifest_contents = serde_json::to_string_pretty(&self)?;
		tokio::fs::write(manifest_path, manifest_contents).await?;

		Ok(())
	}
}

#[cfg(target_os = "linux")]
fn make_executable(path: &Path) -> Result {
	use std::os::unix::fs::PermissionsExt;

	let mut permissions = std::fs::metadata(path)?.permissions();
	permissions.set_mode(permissions.mode() | 0o111);
	std::fs::set_permissions(path, permissions)?;

	Ok(())
}

impl ModDownload {
	async fn download(
		&self,
		mod_id: &str,
		on_progress: &(impl Fn(ProgressStatus) + Send + Sync),
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
			tokio::fs::create_dir_all(&extracted_folder).await?;

			let part_path = temp_dir.join("download.zip.part");
			let zip_path = temp_dir.join("download.zip");

			if !zip_path.is_file() {
				http::download(
					&self.url,
					&part_path,
					&format!("{mod_id}:download"),
					on_progress,
				)
				.await?;
				tokio::fs::rename(&part_path, &zip_path).await?;
			}

			let extract_result = files::extract(
				&zip_path,
				&extracted_folder,
				&|extracted_bytes, total_uncompressed| {
					#[expect(
						clippy::cast_precision_loss,
						reason = "Precision loss is irrelevant for progress display"
					)]
					let percentage = if total_uncompressed > 0 {
						extracted_bytes as f64 / total_uncompressed as f64
					} else {
						0.0
					};
					on_progress(ProgressStatus::InProgress {
						id: format!("{mod_id}:extract"),
						progress: percentage,
					});
				},
			);

			match extract_result {
				Ok(()) => return Ok(extracted_folder),
				Err(err) => {
					tokio::fs::remove_dir_all(&temp_dir)
						.await
						.ok_or_log("Failed to remove temp dir");
					if attempts >= 1 {
						return Err(err.into());
					}
					attempts += 1;
				}
			}
		}
	}
}
