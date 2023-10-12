use std::{
	collections::HashMap,
	fs::{self,},
	path::{
		Path,
		PathBuf,
	},
};

use crate::{
	game_executable::{
		self,
		GameExecutable,
	},
	paths::{self,},
	serializable_struct,
	steam::appinfo::SteamLaunchOption,
	Result,
};

serializable_struct!(Game {
	pub id: String,
	pub name: String,
	pub discriminator: Option<String>,
	pub steam_launch: Option<SteamLaunchOption>,
	pub installed_mods: Vec<String>,
	pub executable: GameExecutable,
	pub thumbnail_url: Option<String>,
});

pub type Map = HashMap<String, Game>;

impl Game {
	pub fn new(
		id: &str,
		name: &str,
		discriminator: Option<String>,
		path: &Path,
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

		let installed_mods = match get_installed_mods(id) {
			Ok(mods) => mods,
			Err(err) => {
				println!("Failed to get installed mods for game {id}: {err}");
				vec![]
			}
		};

		Some(Self {
			id: id.to_string(),
			name: name.to_string(),
			discriminator,
			steam_launch: steam_launch.cloned(),
			installed_mods,
			executable: game_executable::get(path),
			thumbnail_url,
		})
	}

	pub fn open_game_folder(&self) -> Result {
		Ok(open::that_detached(paths::path_parent(
			&self.executable.path,
		)?)?)
	}

	pub fn get_installed_mods_folder(&self) -> Result<PathBuf> {
		get_installed_mods_folder(&self.id)
	}

	pub fn open_mods_folder(&self) -> Result {
		Ok(open::that_detached(self.get_installed_mods_folder()?)?)
	}

	pub fn start(&self) -> Result {
		Ok(self.steam_launch.as_ref().map_or_else(
			|| open::that_detached(&self.executable.path),
			|steam_launch| {
				if self.discriminator.is_none() {
					// If a game has no discriminator, it means we're probably using the default launch option.
					// For those, we use the steam://rungameid command, since that one will make steam show a nice
					// loading popup, wait for game updates, etc.
					return open::that_detached(format!(
						"steam://rungameid/{}",
						steam_launch.app_id
					));
				}
				// For the few cases where we're showing an alternative launch option, we use the steam://launch command.
				// This one will show an error if the game needs an update, and doesn't show the nice loading popup,
				// but it allows us to specify the specific launch option to run.
				// This one also supports passing "dialog" instead of the app_type, (steam://launch/{app_id}/dialog)
				// which makes Steam show the launch selection dialogue, but that dialogue stops showing if the user
				// selects the "don't ask again" checkbox.
				open::that_detached(format!(
					"steam://launch/{}/{}",
					steam_launch.app_id,
					steam_launch.app_type.as_deref().unwrap_or("")
				))
			},
		)?)
	}

	pub fn uninstall_mod(&self, mod_id: &str) -> Result {
		let installed_mods_folder = self.get_installed_mods_folder()?;
		let mod_files_folder = installed_mods_folder
			.join("BepInEx")
			.join("plugins")
			.join(mod_id);

		if mod_files_folder.is_dir() {
			fs::remove_dir_all(mod_files_folder)?;
		}

		Ok(())
	}
}

fn get_installed_mods_folder(id: &str) -> Result<PathBuf> {
	let installed_mods_folder = paths::app_data_path()?.join("games").join(id);
	fs::create_dir_all(&installed_mods_folder)?;

	Ok(installed_mods_folder)
}

fn get_installed_mods(id: &str) -> Result<Vec<String>> {
	let pattern = get_installed_mods_folder(id)?
		.join("BepInEx")
		.join("plugins")
		.join("*");
	let entries: Vec<_> = paths::glob_path(&pattern)?.collect();

	Ok(entries
		.iter()
		.filter_map(|entry| match entry {
			Ok(mod_path) => Some(mod_path.file_name()?.to_str()?.to_string()),
			Err(_) => None,
		})
		.collect())
}
