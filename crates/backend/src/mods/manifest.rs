use std::{
	collections::BTreeMap,
	hash::{DefaultHasher, Hash, Hasher},
	path::{Path, PathBuf},
};

use fs_err::PathExt as FsErrPathExt;
use rai_pal_proc_macros::serde_enum;

use crate::{
	app_paths,
	architecture::Architecture,
	files,
	game::Game,
	game_engines::{
		game_engine::{EngineBrand, EngineVersionRange},
		unity::UnityBackend,
	},
	game_launch::{GameLaunch, spawn_game},
	game_providers::game_provider,
	http::{self},
	mods::replacement_token::replace_tokens,
	open_better::open_detached_better,
	operating_system::OperatingSystem,
	path_extensions::PathExt,
	progress_status::ProgressStatus,
	result::{Error, LogErrExt, Result},
};

#[serde_enum]
pub enum ModPathType {
	File,
	Folder,
}

#[serde_enum]
pub enum InstallKind {
	Shared,
	PerGame,
}

#[derive(Clone, Debug)]
pub struct ModPathState {
	pub path: PathBuf,
	kind: ModPathKind,
}

#[derive(Clone, Copy, Debug)]
enum ModPathKind {
	File { exists: bool, folder_exists: bool },
	Folder { exists: bool },
}

impl ModPathState {
	pub const fn exists(&self, open_folder: bool) -> bool {
		match self.kind {
			ModPathKind::File {
				exists,
				folder_exists,
			} => {
				if open_folder {
					folder_exists
				} else {
					exists
				}
			}
			ModPathKind::Folder { exists } => exists,
		}
	}
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ModPath {
	pub path: String,
	#[serde(rename = "type")]
	pub path_type: ModPathType,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, Default)]
#[serde(transparent)]
pub struct GameIds(pub BTreeMap<String, Vec<String>>);

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Mod {
	pub id: String,
	pub tags: Option<Vec<String>>,
	pub game_ids: Option<GameIds>,
	pub title: String,
	pub author: String,
	pub website: Option<String>,
	pub description: String,
	pub download: Option<ModDownload>,
	pub engine: Option<EngineBrand>,
	pub engine_version_range: Option<EngineVersionRange>,
	pub unity_backend: Option<UnityBackend>,
	pub architecture: Option<Architecture>,
	pub game_os: Option<OperatingSystem>,
	pub host_os: Option<OperatingSystem>,
	pub deprecated: Option<bool>,
	pub paths: Option<Vec<ModPath>>,
	pub optional_dependencies: Option<Vec<ModDependency>>,
	pub required_dependencies: Option<Vec<ModDependency>>,
	pub install: Option<ModInstall>,
	pub run_for_game: Option<ModRun>,
	pub run_standalone: Option<ModRun>,
	pub game_environment: Option<BTreeMap<String, String>>,
	pub hash: Option<String>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ModInstallExtract {
	pub source: String,
	pub destination: String,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ModInstallWrite {
	pub content: String,
	pub destination: String,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ModInstallCopy {
	pub destination: String,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ModInstall {
	pub install_kind: Option<InstallKind>,
	#[serde(default)]
	pub dependency_only: bool,
	pub extract: Option<Vec<ModInstallExtract>>,
	pub copy: Option<Vec<ModInstallCopy>>,
	pub write: Option<Vec<ModInstallWrite>>,
	pub wine_dll_overrides: Option<Vec<String>>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ModRun {
	pub path: Option<String>,
	pub args: Option<Vec<String>>,
	pub wine_environment: Option<BTreeMap<String, String>>,
	pub os: Option<OperatingSystem>,
}

pub struct PreparedModRun {
	pub path: PathBuf,
	pub args: Vec<String>,
	pub wine_environment: BTreeMap<String, String>,
	pub os: Option<OperatingSystem>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ModDownload {
	pub id: String,
	pub url: String,
	pub release_date: Option<String>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[serde(untagged)]
pub enum ModDependency {
	#[serde(rename_all = "camelCase")]
	ModId { mod_id: String },
	#[serde(rename_all = "camelCase")]
	Tag { tag: String },
}

#[derive(Clone, Debug)]
pub struct InstalledManifest {
	pub id: String,
	pub manifest: Option<Mod>,
}

impl InstalledManifest {
	pub fn from_file(path: &Path, install_kind: InstallKind) -> Option<Self> {
		let Ok(contents) = fs_err::read_to_string(path) else {
			log::warn!("Failed to read installed mod manifest `{}`", path.display());
			return None;
		};

		match serde_json::from_str::<Mod>(&contents) {
			Ok(mut manifest) => {
				manifest.normalize_legacy_scope();

				if let Some(install) = &mut manifest.install
					&& install.install_kind.is_none()
				{
					install.install_kind = Some(install_kind);
				}

				Some(Self {
					id: manifest.id.clone(),
					manifest: Some(manifest),
				})
			}
			Err(error) => {
				log::debug!(
					"Failed to parse installed mod manifest `{}`: {error}",
					path.display()
				);

				let name_source = if install_kind == InstallKind::Shared {
					path.parent()
				} else {
					Some(path)
				};

				let file_name = name_source
					.and_then(Path::file_name)
					.and_then(|file_name| file_name.to_str())
					.unwrap_or_default()
					.to_string();

				Some(Self {
					id: file_name,
					manifest: None,
				})
			}
		}
	}
}

fn strip_scope_prefix(value: &str) -> &str {
	let mut parts = value.splitn(3, ':');

	if let (Some("Folder" | "Url"), Some(hash), Some(rest)) =
		(parts.next(), parts.next(), parts.next())
		&& !hash.is_empty()
		&& hash.bytes().all(|byte| byte.is_ascii_digit())
	{
		return rest;
	}

	value
}

impl Mod {
	pub const FILE_NAME: &'static str = "rai-pal-manifest.json";

	fn normalize_legacy_scope(&mut self) {
		self.id = strip_scope_prefix(&self.id).to_string();

		if let Some(tags) = &mut self.tags {
			for tag in tags.iter_mut() {
				*tag = strip_scope_prefix(tag).to_string();
			}
		}

		for dependencies in [
			&mut self.required_dependencies,
			&mut self.optional_dependencies,
		]
		.into_iter()
		.flatten()
		{
			for dependency in dependencies.iter_mut() {
				if let ModDependency::ModId { mod_id } = dependency {
					*mod_id = strip_scope_prefix(mod_id).to_string();
				}
			}
		}
	}

	pub fn release_date(&self) -> Option<&str> {
		self.download
			.as_ref()
			.and_then(|download| download.release_date.as_deref())
	}

	pub fn get_manifest_target_path(&self, game: Option<&Game>) -> Result<PathBuf> {
		let install_kind = self.get_install()?.install_kind.ok_or_else(|| {
			Error::ModInfoMissing(self.id.clone(), "install.installKind".to_string())
		})?;

		let path = match install_kind {
			InstallKind::Shared => app_paths::shared_mods_path()?
				.join(&self.id)
				.join(Self::FILE_NAME),
			InstallKind::PerGame => game
				.ok_or(Error::GameNeeded())?
				.get_installed_mods_folder_path()?
				.join("manifests")
				.join(format!("{}.json", self.id)),
		};

		Ok(path)
	}

	pub fn from_file(path: &Path) -> Option<Self> {
		read_manifest(path)
	}

	pub fn get_install(&self) -> Result<&ModInstall> {
		self.install
			.as_ref()
			.ok_or_else(|| Error::ModInfoMissing(self.id.clone(), "install".to_string()))
	}

	pub async fn install(
		&self,
		game_option: Option<&Game>,
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

			let source_dir = download
				.download(
					&replace_tokens(&download.url, game_option, self),
					&self.id,
					true,
					&on_progress,
				)
				.await?;

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
					fs_err::tokio::create_dir_all(destination_path.try_parent()?).await?;
					fs_err::tokio::copy(&source_path, &destination_path).await?;
				}
			}
		}

		if let Some(copy_actions) = install.copy.as_ref() {
			let download = self
				.download
				.as_ref()
				.ok_or_else(|| Error::ModInfoMissing(self.id.clone(), "download".to_string()))?;

			let source_path = download
				.download(
					&replace_tokens(&download.url, game_option, self),
					&self.id,
					false,
					&on_progress,
				)
				.await?;

			for copy_action in copy_actions {
				let destination_path =
					PathBuf::from(replace_tokens(&copy_action.destination, game_option, self));

				fs_err::tokio::create_dir_all(destination_path.try_parent()?).await?;
				fs_err::tokio::copy(&source_path, &destination_path).await?;
			}
		}

		if let Some(write_actions) = install.write.as_ref() {
			for write_action in write_actions {
				let destination_path =
					PathBuf::from(replace_tokens(&write_action.destination, game_option, self));
				let content = replace_tokens(&write_action.content, game_option, self);

				fs_err::tokio::create_dir_all(destination_path.try_parent()?).await?;
				fs_err::tokio::write(destination_path, content).await?;
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

	pub fn open_path(&self, path: &str, game: Option<&Game>, open_folder: bool) -> Result {
		let mod_path = self.get_path(path)?;
		let resolved_path = PathBuf::from(replace_tokens(&mod_path.path, game, self));

		if open_folder || matches!(&mod_path.path_type, ModPathType::Folder) {
			resolved_path.open_folder_or_parent()?;
		} else if resolved_path.is_file() {
			open_detached_better(resolved_path)?;
		} else {
			resolved_path.open_folder_or_parent()?;
		}

		Ok(())
	}

	pub fn get_path_state(&self, path: &str, game: Option<&Game>) -> Result<ModPathState> {
		let mod_path = self.get_path(path)?;
		let resolved_path = PathBuf::from(replace_tokens(&mod_path.path, game, self));

		let kind = match &mod_path.path_type {
			ModPathType::File => ModPathKind::File {
				exists: resolved_path.is_file(),
				folder_exists: resolved_path.parent().is_some_and(Path::is_dir),
			},
			ModPathType::Folder => ModPathKind::Folder {
				exists: resolved_path.is_dir(),
			},
		};

		Ok(ModPathState {
			path: resolved_path,
			kind,
		})
	}

	fn get_path(&self, path: &str) -> Result<&ModPath> {
		self.paths
			.as_ref()
			.and_then(|paths| paths.iter().find(|mod_path| mod_path.path == path))
			.ok_or_else(|| Error::ModInfoMissing(self.id.clone(), format!("path `{path}`")))
	}

	pub fn get_run(&self, game_option: Option<&Game>) -> Result<&ModRun> {
		if game_option.is_some() {
			self.run_for_game
				.as_ref()
				.ok_or_else(|| Error::ModInfoMissing(self.id.clone(), "run_for_game".to_string()))
		} else {
			self.run_standalone
				.as_ref()
				.ok_or_else(|| Error::ModInfoMissing(self.id.clone(), "run_standalone".to_string()))
		}
	}

	pub fn prepare_run(&self, game_option: Option<&Game>) -> Result<PreparedModRun> {
		self.prepare_run_inner(self.get_run(game_option)?, game_option)
	}

	fn prepare_run_inner(
		&self,
		mod_run: &ModRun,
		game_option: Option<&Game>,
	) -> Result<PreparedModRun> {
		let run_path = PathBuf::from(replace_tokens(
			mod_run.path.as_ref().ok_or_else(|| {
				Error::ModInfoMissing(self.id.clone(), "mod_run.path".to_string())
			})?,
			game_option,
			self,
		));

		if !run_path.fs_err_try_exists()? {
			return Err(Error::ModNotInstalled(self.id.clone()));
		}

		#[cfg(target_os = "linux")]
		if mod_run.os != Some(OperatingSystem::Windows) {
			make_executable(&run_path)?;
		}

		let args: Vec<String> = mod_run
			.args
			.clone()
			.unwrap_or_default()
			.iter()
			.map(|arg| replace_tokens(arg, game_option, self))
			.collect();

		Ok(PreparedModRun {
			path: run_path,
			args,
			wine_environment: replace_env_tokens(
				mod_run.wine_environment.as_ref(),
				game_option,
				self,
			),
			os: mod_run.os,
		})
	}

	pub fn get_game_environment(&self, game: Option<&Game>) -> BTreeMap<String, String> {
		replace_env_tokens(self.game_environment.as_ref(), game, self)
	}

	pub fn run(&self, game_option: Option<&Game>) -> Result {
		let run = self.prepare_run(game_option)?;

		log::info!(
			"Running mod `{}` at `{}` with args `{}`",
			self.id,
			run.path.display(),
			run.args.join(" ")
		);

		#[cfg(target_os = "linux")]
		{
			if run.os == Some(OperatingSystem::Windows) {
				let game = game_option.ok_or_else(Error::GameNeeded)?;

				game_provider::get_provider(game.provider_id)?.run_with_wine(
					game,
					&run.path,
					&run.args,
					&run.wine_environment,
				)?;
			} else {
				let mut launch = GameLaunch::new(&run.path);
				launch.args(&run.args);
				launch.cwd = Some(run.path.try_parent()?.to_path_buf());
				spawn_game(&launch)?;
			}
		}

		#[cfg(target_os = "windows")]
		{
			use crate::open_better::spawn_detached;

			let mut command = std::process::Command::new(&run.path);
			command.current_dir(run.path.try_parent()?).args(&run.args);
			spawn_detached(&mut command)?;
		}

		Ok(())
	}

	pub async fn update_installed_mod_manifest(&self, game: Option<&Game>) -> Result {
		let manifest_path = self.get_manifest_target_path(game)?;
		fs_err::tokio::create_dir_all(manifest_path.try_parent()?).await?;
		let manifest_contents = serde_json::to_string_pretty(&self)?;
		fs_err::tokio::write(manifest_path, manifest_contents).await?;

		Ok(())
	}
}

fn read_manifest<T: serde::de::DeserializeOwned>(path: &Path) -> Option<T> {
	match fs_err::read_to_string(path)
		.and_then(|manifest_bytes| Ok(serde_json::from_str::<T>(&manifest_bytes)?))
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

fn replace_env_tokens(
	environment: Option<&BTreeMap<String, String>>,
	game_option: Option<&Game>,
	game_mod: &Mod,
) -> BTreeMap<String, String> {
	environment.map_or_default(|environment| {
		environment
			.iter()
			.map(|(key, value)| (key.clone(), replace_tokens(value, game_option, game_mod)))
			.collect()
	})
}

#[cfg(target_os = "linux")]
fn make_executable(path: &Path) -> Result {
	use std::os::unix::fs::PermissionsExt;

	let mut permissions = fs_err::metadata(path)?.permissions();
	permissions.set_mode(permissions.mode() | 0o111);
	fs_err::set_permissions(path, permissions)?;

	Ok(())
}

impl ModDownload {
	async fn download(
		&self,
		url: &str,
		mod_id: &str,
		extract: bool,
		on_progress: &(impl Fn(ProgressStatus) + Send + Sync),
	) -> Result<PathBuf> {
		if let Some(local_path) = url.strip_prefix("file://") {
			return Ok(PathBuf::from(local_path));
		}

		let mut hasher = DefaultHasher::new();
		url.hash(&mut hasher);
		self.id.hash(&mut hasher);
		let url_hash = hasher.finish().to_string();

		let temp_dir = app_paths::temp_dir(&url_hash)?;
		let extracted_folder = temp_dir.join("extracted");
		let download_path = temp_dir.join(if extract { "download.zip" } else { "download" });
		let part_path = temp_dir.join("download.part");

		let mut attempts = 0;
		loop {
			if extract {
				fs_err::tokio::create_dir_all(&extracted_folder).await?;
			}

			if !download_path.is_file() {
				let download_step = InstallStep {
					mod_id: mod_id.to_string(),
					kind: InstallStepKind::Download,
				};

				http::download(url, &part_path, &download_step, on_progress).await?;
				fs_err::tokio::rename(&part_path, &download_path).await?;
			}

			if !extract {
				return Ok(download_path);
			}

			let extract_result = files::extract(
				&download_path,
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
					on_progress(ProgressStatus {
						step: InstallStep {
							mod_id: mod_id.to_string(),
							kind: InstallStepKind::Extract,
						},
						progress: percentage,
					});
				},
			);

			match extract_result {
				Ok(()) => return Ok(extracted_folder),
				Err(err) => {
					fs_err::tokio::remove_dir_all(&temp_dir)
						.await
						.ok_or_log("Failed to remove temp dir");
					if attempts >= 1 {
						return Err(err);
					}
					attempts += 1;
				}
			}
		}
	}
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InstallStepKind {
	Download,
	Extract,
}

#[derive(Clone, Debug)]
pub struct InstallStep {
	pub mod_id: String,
	pub kind: InstallStepKind,
}

pub fn install_steps(mods: &[Mod]) -> Vec<InstallStep> {
	let mut steps = Vec::new();

	for game_mod in mods {
		if game_mod.download.is_none() {
			continue;
		}

		steps.push(InstallStep {
			mod_id: game_mod.id.clone(),
			kind: InstallStepKind::Download,
		});

		if game_mod
			.install
			.as_ref()
			.and_then(|install| install.extract.as_ref())
			.is_some()
		{
			steps.push(InstallStep {
				mod_id: game_mod.id.clone(),
				kind: InstallStepKind::Extract,
			});
		}
	}

	steps
}
