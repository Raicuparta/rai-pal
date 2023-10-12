use std::{
	collections::HashMap,
	fs::{self,},
	hash::Hash,
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
	mod_loaders::mod_loader,
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
	pub available_mods: HashMap<String, bool>,
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
		mod_loaders: &mod_loader::DataMap,
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

		let executable = game_executable::get(path);
		let available_mods = get_available_mods(id, mod_loaders, &executable);

		Some(Self {
			id: id.to_string(),
			name: name.to_string(),
			discriminator,
			steam_launch: steam_launch.cloned(),
			available_mods,
			executable,
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

fn get_installed_mods_folder(game_id: &str) -> Result<PathBuf> {
	let installed_mods_folder = paths::app_data_path()?.join("games").join(game_id);
	fs::create_dir_all(&installed_mods_folder)?;

	Ok(installed_mods_folder)
}

fn is_mod_installed(game_id: &str, mod_id: &str) -> bool {
	if let Ok(installed_mods_folder) = get_installed_mods_folder(game_id) {
		return installed_mods_folder
			.join("BepInEx")
			.join("plugins")
			.join(mod_id)
			.is_dir();
	}

	false
}

fn get_available_mods(
	game_id: &str,
	mod_loaders: &mod_loader::DataMap,
	executable: &GameExecutable,
) -> HashMap<String, bool> {
	mod_loaders
		.iter()
		.flat_map(|(_, mod_loader)| &mod_loader.mods)
		.filter_map(|game_mod| {
			if game_mod.engine? == executable.engine.as_ref()?.brand
				&& game_mod.scripting_backend? == executable.scripting_backend?
			{
				Some((game_mod.id.clone(), is_mod_installed(game_id, &game_mod.id)))
			} else {
				None
			}
		})
		.collect()
}
