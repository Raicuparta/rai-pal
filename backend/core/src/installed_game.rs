use std::{
	collections::{HashMap, HashSet},
	fs::{self},
	path::{Path, PathBuf},
};

use log::error;
use rai_pal_proc_macros::serializable_struct;

use crate::{
	game_engines::{game_engine::EngineBrand, unity::UnityScriptingBackend},
	game_executable::{Architecture, GameExecutable},
	game_tag::GameTag,
	game_title::GameTitle,
	mod_manifest, owned_game,
	paths::{self, glob_path, hash_path},
	providers::{provider::ProviderId, provider_command::ProviderCommand},
	result::{Error, Result},
};

#[serializable_struct]
pub struct InstalledGame {
	pub id: String,
	pub title: GameTitle,
	pub provider: ProviderId,
	pub executable: GameExecutable,
	pub installed_mod_versions: InstalledModVersions,
	pub discriminator: Option<String>,
	pub thumbnail_url: Option<String>,
	pub owned_game_id: Option<String>,
	pub start_command: Option<ProviderCommand>,
}

#[serializable_struct]
pub struct InstalledGamesFilter {
	pub providers: HashMap<ProviderId, bool>,
	pub tags: HashMap<GameTag, bool>,
	pub architectures: HashMap<Architecture, bool>,
	pub unity_scripting_backends: HashMap<UnityScriptingBackend, bool>,
	pub engines: HashMap<EngineBrand, bool>,
}

impl InstalledGamesFilter {
	pub fn matches(&self, game: &InstalledGame) -> bool {
		if !self.providers.get(&game.provider).unwrap_or(&true) {
			return false;
		}

		// if !self.tags.iter().any(|(tag, enabled)| {
		// 	*enabled && game.title.tags.contains(tag)
		// }) {
		// 	matches = false;
		// }

		let mut architectures = self.architectures.iter();
		if architectures.any(|(_, enabled)| !enabled)
			&& !self.architectures.iter().any(|(architecture, enabled)| {
				*enabled
					&& game
						.executable
						.architecture
						.is_some_and(|a| a == *architecture)
			}) {
			return false;
		}

		let mut engines = self.engines.iter();
		if engines.any(|(_, enabled)| !enabled)
			&& !engines.any(|(engine, enabled)| {
				*enabled
					&& game
						.executable
						.engine
						.as_ref()
						.is_some_and(|e| e.brand == *engine)
			}) {
			return false;
		}

		let mut scripting_backends = self.unity_scripting_backends.iter();
		if scripting_backends.any(|(_, enabled)| !enabled)
			&& !scripting_backends.any(|(backend, enabled)| {
				*enabled
					&& game
						.executable
						.scripting_backend
						.is_some_and(|b| b == *backend)
			}) {
			return false;
		}

		true
	}
}

impl Default for InstalledGamesFilter {
	fn default() -> Self {
		Self {
			architectures: Architecture::variants()
				.into_iter()
				.map(|variant| (variant, true))
				.collect(),
			engines: EngineBrand::variants()
				.into_iter()
				.map(|variant| (variant, true))
				.collect(),
			providers: ProviderId::variants()
				.into_iter()
				.map(|variant| (variant, true))
				.collect(),
			tags: GameTag::variants()
				.into_iter()
				.map(|variant| (variant, true))
				.collect(),
			unity_scripting_backends: UnityScriptingBackend::variants()
				.into_iter()
				.map(|variant| (variant, true))
				.collect(),
		}
	}
}

type InstalledModVersions = HashMap<String, String>;

impl InstalledGame {
	pub fn new(path: &Path, name: &str, provider_id: ProviderId) -> Option<Self> {
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
			title: GameTitle::new(name),
			provider: provider_id,
			installed_mod_versions: HashMap::default(),
			executable: GameExecutable::new(path)?,
			discriminator: None,
			thumbnail_url: None,
			start_command: None,
			owned_game_id: None,
		};

		installed_game.refresh_installed_mods();

		Some(installed_game)
	}

	pub fn set_discriminator(&mut self, discriminator: &str) -> &Self {
		self.discriminator = Some(discriminator.to_string());
		self
	}

	pub fn set_thumbnail_url(&mut self, thumbnail_url: &str) -> &Self {
		self.thumbnail_url = Some(thumbnail_url.to_string());
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

	pub fn set_provider_game_id(&mut self, provider_game_id: &str) -> &Self {
		self.owned_game_id = Some(owned_game::get_global_id(self.provider, provider_game_id));
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
