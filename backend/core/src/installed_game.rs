use std::{
	collections::HashMap,
	fs::{self},
	path::{Path, PathBuf},
};

use log::error;
use rai_pal_proc_macros::serializable_struct;

use crate::{
	game_executable::GameExecutable,
	mod_manifest,
	paths::{self, glob_path, hash_path},
	providers::provider_command::ProviderCommand,
	result::{Error, Result},
};

#[serializable_struct]
pub struct InstalledGame {
	pub id: String,
	pub executable: GameExecutable,
	pub installed_mod_versions: InstalledModVersions,
	pub discriminator: Option<String>,
	pub start_command: Option<ProviderCommand>,
	pub has_outdated_mod: bool,
}

type InstalledModVersions = HashMap<String, String>;

impl InstalledGame {
	pub fn new(path: &Path) -> Option<Self> {
		// Games exported by Unity always have one of these extensions.
		const VALID_EXTENSIONS: [&str; 3] = ["exe", "x86_64", "x86"];

		if !path.is_file() {
			return None;
		}

		// We ignore games that don't have an extension.
		let extension = path.extension()?.to_str()?;

		if !VALID_EXTENSIONS.contains(&extension) {
			return None;
		}

		if extension == "x86" && path.with_extension("x86_64").is_file() {
			// If there's an x86_64 version, we ignore the x86 version.
			// I'm just gonna presume there are no x86 modders out there,
			// if someone cries about it I'll make this smarter.
			return None;
		}

		let executable = GameExecutable::new(path)?;

		let game_id = hash_path(&executable.path);

		let mut installed_game = Self {
			id: game_id,
			installed_mod_versions: HashMap::default(),
			executable: GameExecutable::new(path)?,
			discriminator: None,
			start_command: None,
			has_outdated_mod: false,
		};

		installed_game.refresh_installed_mods();

		Some(installed_game)
	}

	pub fn set_discriminator(&mut self, discriminator: &str) -> &Self {
		self.discriminator = Some(discriminator.to_string());
		self
	}

	pub fn set_start_command_string(&mut self, program: &str) -> &Self {
		self.start_command = Some(ProviderCommand::String(program.to_string()));
		self
	}

	pub fn set_start_command_path(&mut self, path: &Path, args: Vec<String>) -> &Self {
		self.start_command = Some(ProviderCommand::Path(path.to_path_buf(), args));
		self
	}

	pub fn refresh_executable(&mut self) -> Result {
		if let Some(executable) = GameExecutable::new(&self.executable.path) {
			self.executable = executable;
		} else {
			return Err(Error::FailedToGetGameFromPath(self.executable.path.clone()));
		}

		Ok(())
	}

	pub fn refresh_installed_mods(&mut self) {
		self.installed_mod_versions = self.get_available_mods();
	}

	pub fn open_game_folder(&self) -> Result {
		Ok(open::that_detached(paths::path_parent(
			&self.executable.path,
		)?)?)
	}

	pub fn open_mods_folder(&self) -> Result {
		Ok(open::that_detached(self.get_installed_mods_folder()?)?)
	}

	pub fn uninstall_all_mods(&self) -> Result {
		Ok(fs::remove_dir_all(self.get_installed_mods_folder()?)?)
	}

	pub fn start(&self) -> Result {
		self.start_command
			.as_ref()
			.map_or_else(|| self.start_exe(), ProviderCommand::run)
	}

	pub fn start_exe(&self) -> Result {
		Ok(open::that_detached(&self.executable.path)?)
	}

	pub fn get_manifest_paths(&self) -> Vec<PathBuf> {
		match self.get_installed_mod_manifest_path("*") {
			Ok(manifests_path) => {
				if !manifests_path.parent().is_some_and(Path::exists) {
					return Vec::default();
				}
				glob_path(&manifests_path)
			}
			Err(err) => {
				error!(
					"Failed to get mod manifests glob path for game {}. Error: {}",
					self.id, err
				);
				Vec::default()
			}
		}
	}

	pub fn get_available_mods(&self) -> InstalledModVersions {
		self.get_manifest_paths()
			.iter()
			.filter_map(|manifest_path| {
				let manifest = mod_manifest::get(manifest_path)?;

				Some((
					manifest_path.file_stem()?.to_str()?.to_string(),
					manifest.version,
				))
			})
			.collect()
	}

	pub fn get_installed_mod_manifest_path(&self, mod_id: &str) -> Result<PathBuf> {
		Ok(self
			.get_installed_mods_folder()?
			.join("manifests")
			.join(format!("{mod_id}.json")))
	}

	pub fn get_installed_mods_folder(&self) -> Result<PathBuf> {
		let installed_mods_folder = paths::app_data_path()?
			.join("installed-mods")
			.join(&self.id);
		fs::create_dir_all(&installed_mods_folder)?;

		Ok(installed_mods_folder)
	}
}
