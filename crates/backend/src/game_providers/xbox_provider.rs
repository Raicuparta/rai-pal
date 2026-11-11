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
	local_database::{app_database::DbMutex, game_database::GameDatabase},
	path_extensions::PathExt,
	result::{LogErrExt, Result},
};

#[derive(Clone)]
pub struct Xbox {}

struct XboxGamepassGame {
	product_title: String,
	product_id: String,
	images: Option<XboxGamepassImages>,
	release_date: String,
	store_page: String,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(rename_all = "PascalCase")]
struct XboxGamepassImages {
	logo: Option<Vec<String>>,
	box_art: Option<Vec<String>>,
}

impl ProviderActions for Xbox {
	fn insert_games(&self, db: &DbMutex) -> Result {
		if let Err(error) = get_games(db) {
			if error.kind() == io::ErrorKind::NotFound {
				log::info!(
					"Failed to find installed Xbox PC games. This probably means the Xbox PC app isn't installed, or there are no Windows Store games or something. Error: {error}"
				);
				return Ok(());
			}
		}

		Ok(())
	}
}

impl WineProviderActions for Xbox {}

fn get_games(db: &DbMutex) -> io::Result<()> {
	let gaming_services =
		RegKey::predef(HKEY_LOCAL_MACHINE).open_subkey("SOFTWARE\\Microsoft\\GamingServices")?;
	let package_roots = gaming_services.open_subkey("PackageRepository\\Root")?;
	let game_configs = gaming_services.open_subkey("GameConfig")?;
	let app_packages = RegKey::predef(HKEY_CURRENT_USER)
			.open_subkey("Software\\Classes\\Local Settings\\Software\\Microsoft\\Windows\\CurrentVersion\\AppModel\\Repository\\Packages")?;

	for key in package_roots.enum_keys().flatten() {
		if let Some(child) = package_roots
			.open_subkey(key)
			.ok_or_log("Failed to open Xbox package root")
		{
			if let Some(Ok(subkey)) = child.enum_keys().next() {
				if let Some(subchild) = child
					.open_subkey(subkey)
					.ok_or_log("Failed to open Xbox package subkey")
				{
					if let Some(package_id) = subchild
						.get_value::<String, _>("Package")
						.ok_or_log("Failed to read Xbox package ID")
					{
						if let Some(root_path) = subchild
							.get_value::<String, _>("Root")
							.ok_or_log("Failed to read Xbox package root path")
						{
							if let Some(executables) = game_configs
								.open_subkey(format!("{package_id}\\Executable"))
								.ok_or_log("Failed to open Xbox game executables key")
							{
								if let Some(Ok(first_executable_key)) =
									executables.enum_keys().next()
								{
									if let Some(first_executable) = executables
										.open_subkey(first_executable_key)
										.ok_or_log("Failed to open Xbox executable key")
									{
										if let Some(executable_name) = first_executable
											.get_value::<String, _>("Name")
											.ok_or_log("Failed to read Xbox executable name")
										{
											let executable_path =
												PathBuf::from(&root_path).join(&executable_name);

											let display_name = app_packages
												.open_subkey(&package_id)
												.and_then(|package| {
													package.get_value::<String, _>("DisplayName")
												})
												.or_else(|error| {
													error!(
														"Failed to find display name for Xbox game: {error}"
													);
													(&executable_path)
														.file_name_without_extension()
														.map(ToString::to_string)
												})
												.unwrap_or_else(|error| {
													error!(
														"Failed to get game name from exe path: {error}"
													);
													"[Name Not Found]".to_string()
												});

											let mut game = Game::new(
												GameProviderId::Xbox,
												package_id.clone(),
												display_name,
											);

											game.set_executable(&executable_path);

											db.insert_game(&game);
										}
									}
								}
							}
						}
					}
				}
			}
		}
	}

	Ok(())
}
