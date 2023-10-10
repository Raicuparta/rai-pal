use std::{
	collections::HashMap,
	fs::{self,},
	path::{
		Path,
		PathBuf,
	},
};

use goblin::{
	elf::Elf,
	pe::PE,
};

use crate::{
	game_engines::{
		game_engine::GameEngine,
		unity::{
			self,
			UnityScriptingBackend,
		},
		unreal,
	},
	paths::{self,},
	serializable_enum,
	serializable_struct,
	steam::appinfo::SteamLaunchOption,
	Result,
};

serializable_enum!(Architecture { X64, X86 });

serializable_enum!(OperatingSystem { Linux, Windows });

serializable_struct!(Game {
	pub id: String,
	pub name: String,
	pub discriminator: Option<String>,
	pub full_path: PathBuf,
	pub architecture: Option<Architecture>,
	pub scripting_backend: Option<UnityScriptingBackend>,
	pub operating_system: Option<OperatingSystem>,
	pub steam_launch: Option<SteamLaunchOption>,
	pub installed_mods: Vec<String>,
	pub engine: Option<GameEngine>,
	pub thumbnail_url: Option<String>,
});

pub type Map = HashMap<String, Game>;

impl Game {
	pub fn new(
		id: &str,
		name: &str,
		discriminator: Option<String>,
		full_path: &Path,
		steam_launch: Option<&SteamLaunchOption>,
		thumbnail_url: Option<String>,
	) -> Option<Self> {
		// Games exported by Unity always have one of these extensions.
		const VALID_EXTENSIONS: [&str; 3] = ["exe", "x86_64", "x86"];

		// We ignore games that don't have an extension.
		let extension = full_path.extension()?.to_str()?;

		if !VALID_EXTENSIONS.contains(&extension) {
			return None;
		}

		if extension == "x86" && full_path.with_extension("x86_64").is_file() {
			// If there's an x86_64 version, we ignore the x86 version.
			// I'm just gonna presume there are no x86 modders out there,
			// if someone cries about it I'll make this smarter.
			return None;
		}

		let (operating_system, architecture) = get_os_and_architecture(full_path).ok()?;

		let installed_mods = match get_installed_mods(id) {
			Ok(mods) => mods,
			Err(err) => {
				println!("Failed to get installed mods for game {id}: {err}");
				vec![]
			}
		};

		let engine = get_engine(full_path, architecture.unwrap_or(Architecture::X64));
		let scripting_backend = unity::get_scripting_backend(full_path, &engine);

		Some(Self {
			architecture,
			full_path: full_path.to_path_buf(),
			id: id.to_string(),
			operating_system,
			name: name.to_string(),
			discriminator,
			scripting_backend,
			steam_launch: steam_launch.cloned(),
			installed_mods,
			engine,
			thumbnail_url,
		})
	}

	pub fn open_game_folder(&self) -> Result {
		Ok(open::that_detached(paths::path_parent(&self.full_path)?)?)
	}

	pub fn get_installed_mods_folder(&self) -> Result<PathBuf> {
		get_installed_mods_folder(&self.id)
	}

	pub fn open_mods_folder(&self) -> Result {
		Ok(open::that_detached(self.get_installed_mods_folder()?)?)
	}

	pub fn start(&self) -> Result {
		Ok(self.steam_launch.as_ref().map_or_else(
			|| open::that_detached(&self.full_path),
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

fn get_engine(game_path: &Path, architecture: Architecture) -> Option<GameEngine> {
	unity::get_engine(game_path).or_else(|| unreal::get_engine(game_path, architecture))
}

fn get_os_and_architecture(
	file_path: &Path,
) -> Result<(Option<OperatingSystem>, Option<Architecture>)> {
	fs::read(file_path).map(|file| {
		let elf_result = match Elf::parse(&file) {
			Ok(elf) => match elf.header.e_machine {
				goblin::elf::header::EM_X86_64 => {
					Ok((Some(OperatingSystem::Linux), Some(Architecture::X64)))
				}
				goblin::elf::header::EM_386 => {
					Ok((Some(OperatingSystem::Linux), Some(Architecture::X86)))
				}
				_ => Ok((Some(OperatingSystem::Linux), None)),
			},
			Err(err) => Err(err),
		};

		if elf_result.is_ok() {
			return Ok(elf_result?);
		}

		let pe_result = match PE::parse(&file) {
			Ok(pe) => match pe.header.coff_header.machine {
				goblin::pe::header::COFF_MACHINE_X86_64 => {
					Ok((Some(OperatingSystem::Windows), Some(Architecture::X64)))
				}
				goblin::pe::header::COFF_MACHINE_X86 => {
					Ok((Some(OperatingSystem::Windows), Some(Architecture::X86)))
				}
				_ => Ok((Some(OperatingSystem::Windows), None)),
			},
			Err(err) => Err(err),
		};

		if pe_result.is_ok() {
			return Ok(pe_result?);
		}

		println!("Failed to parse exe as ELF or PE");
		if let Err(err) = elf_result {
			println!("ELF error: {err}");
		}
		if let Err(err) = pe_result {
			println!("PE error: {err}");
		}

		Ok((None, None))
	})?
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
