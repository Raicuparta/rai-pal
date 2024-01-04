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

use crate::{
	game_executable::GameExecutable,
	game_mod,
	game_mode::GameMode,
	mod_manifest,
	paths::{
		self,
		hash_path,
	},
	providers::provider::ProviderId,
	serializable_struct,
	steam::{
		self,
		appinfo::SteamLaunchOption,
	},
	Error,
	Result,
};

serializable_struct!(InstalledGame {
	pub id: String,
	pub name: String,
	pub provider_id: ProviderId,
	pub discriminator: Option<String>,
	pub steam_launch: Option<SteamLaunchOption>,
	pub executable: GameExecutable,
	pub thumbnail_url: Option<String>,
	pub installed_mod_versions: InstalledModVersions,
	pub game_mode: GameMode,
});

pub type Map = HashMap<String, InstalledGame>;
type InstalledModVersions = HashMap<String, Option<String>>;

impl InstalledGame {
	pub fn new(
		path: &Path,
		name: &str,
		provider_id: ProviderId,
		discriminator: Option<String>,
		steam_launch: Option<&SteamLaunchOption>,
		thumbnail_url: Option<String>,
	) -> Option<Self> {
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

		let game_mode = steam_launch.map_or(GameMode::Flat, SteamLaunchOption::get_game_mode);

		Some(Self {
			id: hash_path(path),
			name: name.to_string(),
			provider_id,
			discriminator,
			steam_launch: steam_launch.cloned(),
			installed_mod_versions: HashMap::default(),
			executable: GameExecutable::new(path)?,
			thumbnail_url,
			game_mode,
		})
	}

	pub fn refresh_executable(&mut self) -> Result {
		if let Some(executable) = GameExecutable::new(&self.executable.path) {
			self.executable = executable;
		} else {
			return Err(Error::FailedToGetGameFromPath(self.executable.path.clone()));
		}

		Ok(())
	}

	pub fn update_available_mods(&mut self, data_map: &game_mod::CommonDataMap) {
		self.installed_mod_versions = self.get_available_mods(data_map);
	}

	pub fn open_game_folder(&self) -> Result {
		Ok(open::that_detached(paths::path_parent(
			&self.executable.path,
		)?)?)
	}

	pub fn open_mods_folder(&self) -> Result {
		Ok(open::that_detached(self.get_installed_mods_folder()?)?)
	}

	pub fn start(&self, handle: &tauri::AppHandle) -> Result {
		self.steam_launch.as_ref().map_or_else(
			|| self.start_exe(),
			|steam_launch| {
				if self.discriminator.is_none() {
					// If a game has no discriminator, it means we're probably using the default launch option.
					// For those, we use the steam://rungameid command, since that one will make steam show a nice
					// loading popup, wait for game updates, etc.
					return steam::command::run(
						&format!("rungameid/{}", steam_launch.app_id),
						handle,
					);
				}
				// For the few cases where we're showing an alternative launch option, we use the steam://launch command.
				// This one will show an error if the game needs an update, and doesn't show the nice loading popup,
				// but it allows us to specify the specific launch option to run.
				// This one also supports passing "dialog" instead of the app_type, (steam://launch/{app_id}/dialog)
				// which makes Steam show the launch selection dialog, but that dialog stops showing if the user
				// selects the "don't ask again" checkbox.
				steam::command::run(
					&format!(
						"launch/{}/{}",
						steam_launch.app_id,
						steam_launch.app_type.as_deref().unwrap_or("")
					),
					handle,
				)
			},
		)
	}

	pub fn start_exe(&self) -> Result {
		Ok(open::that_detached(&self.executable.path)?)
	}

	pub fn uninstall_mod(&self, mod_id: &str) -> Result {
		// TODO this should be handled by each mod loader.
		let installed_mods_folder = self.get_installed_mods_folder()?;
		let bepinex_folder = installed_mods_folder.join("BepInEx");

		let plugins_folder = bepinex_folder.join("plugins").join(mod_id);
		if plugins_folder.is_dir() {
			fs::remove_dir_all(plugins_folder)?;
		}

		let patchers_folder = bepinex_folder.join("patchers").join(mod_id);
		if patchers_folder.is_dir() {
			fs::remove_dir_all(patchers_folder)?;
		}

		let manifest_path = self.get_installed_mod_manifest_path(mod_id)?;
		if manifest_path.is_file() {
			fs::remove_file(manifest_path)?;
		}

		Ok(())
	}

	pub fn refresh_mods(&mut self, data_map: &game_mod::CommonDataMap) {
		self.installed_mod_versions = self.get_available_mods(data_map);
	}

	pub fn get_installed_mods_folder(&self) -> Result<PathBuf> {
		let installed_mods_folder = paths::app_data_path()?
			.join("installed-mods")
			.join(&self.id);
		fs::create_dir_all(&installed_mods_folder)?;

		Ok(installed_mods_folder)
	}

	pub fn get_installed_mod_manifest_path(&self, mod_id: &str) -> Result<PathBuf> {
		Ok(self
			.get_installed_mods_folder()?
			.join("manifests")
			.join(format!("{mod_id}.json")))
	}

	pub fn get_installed_mod_version(&self, mod_id: &str) -> Option<String> {
		let manifest_path = self.get_installed_mod_manifest_path(mod_id).ok()?;
		let manifest_file = File::open(manifest_path).ok()?;
		let manifest: mod_manifest::Manifest = serde_json::from_reader(manifest_file).ok()?;
		Some(manifest.version)
	}

	pub fn get_available_mods(&self, data_map: &game_mod::CommonDataMap) -> InstalledModVersions {
		data_map
			.iter()
			.filter_map(|(mod_id, mod_data)| {
				if equal_or_none(
					mod_data.engine,
					self.executable.engine.as_ref().map(|engine| engine.brand),
				) && equal_or_none(mod_data.unity_backend, self.executable.scripting_backend)
				{
					Some((mod_id.clone(), self.get_installed_mod_version(mod_id)))
				} else {
					None
				}
			})
			.collect()
	}
}

fn equal_or_none<T: PartialEq>(a: Option<T>, b: Option<T>) -> bool {
	match (a, b) {
		(Some(value_a), Some(value_b)) => value_a == value_b,
		_ => true,
	}
}
