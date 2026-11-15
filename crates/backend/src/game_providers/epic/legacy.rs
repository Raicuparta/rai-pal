use fs_err as fs;
use fs_err::File;
use std::{
	collections::HashMap,
	io::Read,
	path::{Path, PathBuf},
};

use base64::engine::general_purpose;
use log::error;

#[cfg(target_os = "windows")]
use winreg::{RegKey, enums::HKEY_LOCAL_MACHINE};

use crate::{
	game::Game,
	game_providers::{
		epic::{EpicCatalogItem, add_start_via_provider_command},
		game_provider::ProviderParser,
	},
	path_extensions::PathExt,
	result::{ContextResult, Result},
};

#[derive(serde::Deserialize)]
pub struct EpicManifest {
	#[serde(rename = "LaunchExecutable")]
	launch_executable: String,
	#[serde(rename = "InstallLocation")]
	install_location: String,
	#[serde(rename = "CatalogItemId")]
	catalog_item_id: String,
}

pub struct LegacyEpic {}

impl ProviderParser for LegacyEpic {
	type Source = PathBuf;

	fn parse_games(&self, app_data_path: &Self::Source) -> Result<Vec<Game>> {
		let mut games = Vec::new();
		let mut exe_paths: HashMap<String, PathBuf> = HashMap::new();

		let manifests_path = app_data_path.join("Manifests");
		if manifests_path.is_dir() {
			let manifest_paths = manifests_path.join("*.item").glob();
			for manifest_path in manifest_paths {
				match read_manifest(&manifest_path) {
					Ok(manifest) => {
						let path = PathBuf::from(manifest.install_location)
							.join(manifest.launch_executable);

						exe_paths.insert(manifest.catalog_item_id.clone(), path);
					}
					Err(err) => {
						error!("Failed to parse epic manifest: {err}");
					}
				}
			}
		} else {
			log::info!(
				"Epic Games Launcher manifests folder not found. Probably means Epic Games Launcher isn't installed, or maybe user hasn't installed any games dunno."
			);
		}

		let catalog_path = app_data_path.join("Catalog").join("catcache.bin");
		if catalog_path.is_file() {
			let mut catalog_cache_file = File::open(&catalog_path)?;
			let mut decoder = base64::read::DecoderReader::new(
				&mut catalog_cache_file,
				&general_purpose::STANDARD,
			);
			let mut json = String::default();
			decoder.read_to_string(&mut json).with_context(|| {
				format!(
					"failed to decode Epic catalog cache `{}`",
					catalog_path.display()
				)
			})?;

			let catalog =
				serde_json::from_str::<Vec<EpicCatalogItem>>(&json).with_context(|| {
					format!(
						"failed to parse Epic catalog cache `{}`",
						catalog_path.display()
					)
				})?;
			for catalog_item in catalog {
				if !catalog_item.is_game() {
					continue;
				}

				let Some(mut game) = catalog_item.to_game() else {
					continue;
				};

				if let Some(exe_path) = exe_paths.remove(&game.external_id) {
					game.set_executable(&exe_path);
					add_start_via_provider_command(
						&mut game,
						&catalog_item.namespace,
						&catalog_item.id,
						catalog_item.artifact_id(),
					);
				}

				games.push(game);
			}
		} else {
			log::info!(
				"Epic Games Launcher catalog cache file not found. Probably means user hasn't installed Epic Games Launcher, or the cache file hasn't been created yet."
			);
		}

		Ok(games)
	}
}

#[cfg(target_os = "windows")]
pub fn app_data_path() -> Option<PathBuf> {
	match RegKey::predef(HKEY_LOCAL_MACHINE)
		.open_subkey(r"SOFTWARE\Epic Games\EpicGamesLauncher")
		.and_then(|launcher_reg| launcher_reg.get_value::<String, _>("AppDataPath"))
		.map(PathBuf::from)
	{
		Ok(path) => Some(path),
		Err(err) => {
			log::info!(
				"Epic Games Launcher not found in registry. Probably means Epic Games Launcher isn't installed. Error: {err}"
			);
			None
		}
	}
}

fn read_manifest(path: &Path) -> Result<EpicManifest> {
	let json = fs::read_to_string(path)?;
	let manifest = serde_json::from_str::<EpicManifest>(&json)
		.with_context(|| format!("failed to parse Epic manifest `{}`", path.display()))?;
	Ok(manifest)
}
