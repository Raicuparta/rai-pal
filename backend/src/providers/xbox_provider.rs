use std::path::PathBuf;

use async_trait::async_trait;
use winreg::{
	enums::HKEY_LOCAL_MACHINE,
	RegKey,
};

use super::provider::ProviderId;
use crate::{
	installed_game::InstalledGame,
	owned_game::OwnedGame,
	paths::file_name_without_extension,
	provider::{
		ProviderActions,
		ProviderStatic,
	},
	Result,
};

pub struct XboxProvider {}

impl ProviderStatic for XboxProvider {
	const ID: &'static ProviderId = &ProviderId::Xbox;

	fn new() -> Result<Self>
	where
		Self: Sized,
	{
		Ok(Self {})
	}
}

#[async_trait]
impl ProviderActions for XboxProvider {
	fn get_installed_games(&self) -> Result<Vec<InstalledGame>> {
		let gaming_services = RegKey::predef(HKEY_LOCAL_MACHINE)
			.open_subkey("SOFTWARE\\Microsoft\\GamingServices")?;
		let package_roots = gaming_services.open_subkey("PackageRepository\\Root")?;
		let game_configs = gaming_services.open_subkey("GameConfig")?;

		let mut result = Vec::default();

		for key_result in package_roots.enum_keys() {
			if let Ok(key) = key_result {
				if let Ok(child) = package_roots.open_subkey(key) {
					if let Some(Ok(subkey)) = child.enum_keys().nth(0) {
						if let Ok(subchild) = child.open_subkey(subkey) {
							if let Ok(package_id) = subchild.get_value::<String, _>("Package") {
								if let Ok(root_path) = subchild.get_value::<String, _>("Root") {
									if let Ok(executables) = game_configs
										.open_subkey(format!("{}\\Executable", package_id))
									{
										if let Some(Ok(first_executable_key)) =
											executables.enum_keys().nth(0)
										{
											if let Ok(first_executable) =
												executables.open_subkey(first_executable_key)
											{
												if let Ok(executable_name) =
													first_executable.get_value::<String, _>("Name")
												{
													let executable_path = PathBuf::from(root_path)
														.join(executable_name);
													println!(
														"Found xbox game in path {}",
														executable_path.display().to_string()
													);
													if let Ok(game_name) =
														file_name_without_extension(
															&executable_path,
														) {
														if let Some(game) = InstalledGame::new(
															&executable_path,
															game_name,
															*Self::ID,
															None,
															None,
															None,
														) {
															result.push(game);
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
				}
			}
		}

		Ok(result)
	}

	async fn get_owned_games(&self) -> Result<Vec<OwnedGame>> {
		Ok(Vec::default())

		// TODO figure out if this is worth implementing.
		// Ok(game_scanner::gog::games()
		// 	.unwrap_or_default()
		// 	.iter()
		// 	.map(|game| OwnedGame {
		// 		// TODO should add a constructor to OwnedGame to avoid ID collisions and stuff.
		// 		id: game.id.clone(),
		// 		provider_id: *Self::ID,
		// 		name: game.name.clone(),
		// 		installed: false, // TODO
		// 		os_list: HashSet::default(),
		// 		// Make engine optional?
		// 		engine: GameEngineBrand::Unity,
		// 		release_date: 0,
		// 		thumbnail_url: String::default(),
		// 		game_mode: GameMode::Flat,
		// 		uevr_score: None,
		// 	})
		// 	.collect())
	}
}
