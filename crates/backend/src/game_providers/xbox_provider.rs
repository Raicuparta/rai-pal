#![cfg(target_os = "windows")]

use std::{io, path::PathBuf};

use log::error;
use winreg::{
	RegKey,
	enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE},
};

use crate::{
	game::Game,
	game_providers::game_provider::{GameProviderId, ProviderActions, WineProviderActions},
	path_extensions::PathExt,
	result::{LogErrExt, Result},
};

#[derive(Clone)]
pub struct Xbox {}

impl ProviderActions for Xbox {
	fn get_games(&self) -> Result<Vec<Game>> {
		match read_games() {
			Ok(games) => Ok(games),
			Err(error) if error.kind() == io::ErrorKind::NotFound => {
				log::info!(
					"Failed to find installed Xbox PC games. This probably means the Xbox PC app isn't installed, or there are no Windows Store games or something. Error: {error}"
				);
				Ok(Vec::new())
			}
			Err(error) => Err(error.into()),
		}
	}
}

impl WineProviderActions for Xbox {}

fn read_games() -> io::Result<Vec<Game>> {
	let registry = Registry::open()?;

	Ok(registry
		.package_roots
		.enum_keys()
		.filter_map(|package_name| package_name.ok_or_log("Failed to read Xbox package name"))
		.filter_map(|package_name| registry.read_game(package_name))
		.collect())
}

struct Registry {
	package_roots: RegKey,
	game_configs: RegKey,
	app_packages: RegKey,
}

impl Registry {
	fn open() -> io::Result<Self> {
		let gaming_services = RegKey::predef(HKEY_LOCAL_MACHINE)
			.open_subkey("SOFTWARE\\Microsoft\\GamingServices")?;

		Ok(Self {
			package_roots: gaming_services.open_subkey("PackageRepository\\Root")?,
			game_configs: gaming_services.open_subkey("GameConfig")?,
			app_packages: RegKey::predef(HKEY_CURRENT_USER).open_subkey(
				"Software\\Classes\\Local Settings\\Software\\Microsoft\\Windows\\CurrentVersion\\AppModel\\Repository\\Packages",
			)?,
		})
	}

	fn read_game(&self, package_name: String) -> Option<Game> {
		self.try_read_game(&package_name)
			.ok_or_log(&format!("Failed to read Xbox package `{package_name}`"))
			.flatten()
	}

	fn try_read_game(&self, package_name: &str) -> io::Result<Option<Game>> {
		let package = self.package_roots.open_subkey(package_name)?;
		let version = first_subkey(&package)?;
		let package = package.open_subkey(version)?;
		let package_id = package.get_value::<String, _>("Package")?;
		let root_path = package.get_value::<String, _>("Root")?;

		let Some(executables) = self.game_executables(&package_id)? else {
			return Ok(None);
		};

		let executable_key = first_subkey(&executables)?;
		let executable = executables.open_subkey(executable_key)?;
		let executable_name = executable.get_value::<String, _>("Name")?;

		let executable_path = PathBuf::from(&root_path).join(&executable_name);

		let display_name = self
			.app_packages
			.open_subkey(&package_id)
			.and_then(|app_package| app_package.get_value::<String, _>("DisplayName"))
			.or_else(|error| {
				error!("Failed to find display name for Xbox game: {error}");
				executable_path
					.file_name_without_extension()
					.map(ToString::to_string)
			})
			.unwrap_or_else(|error| {
				error!("Failed to get game name from exe path: {error}");
				"[Name Not Found]".to_string()
			});

		let mut game = Game::new(GameProviderId::Xbox, package_id, display_name);
		game.set_executable(&executable_path);

		Ok(Some(game))
	}

	fn game_executables(&self, package_id: &str) -> io::Result<Option<RegKey>> {
		match self
			.game_configs
			.open_subkey(format!("{package_id}\\Executable"))
		{
			Ok(executables) => Ok(Some(executables)),
			Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
			Err(error) => Err(error),
		}
	}
}

fn first_subkey(key: &RegKey) -> io::Result<String> {
	let first = key
		.enum_keys()
		.next()
		.ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Registry key has no subkeys"))?;

	first
}
