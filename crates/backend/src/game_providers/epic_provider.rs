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
		game_provider::{GameProviderId, ProviderParser},
		provider_command::{ProviderCommandAction, ProviderCommandKind},
	},
	path_extensions::PathExt,
	result::{ContextResult, LogErrExt, Result},
};

#[cfg(target_os = "windows")]
use crate::game_providers::game_provider::{ProviderActions, WineProviderActions};

#[derive(serde::Deserialize)]
pub struct EpicManifest {
	#[serde(rename = "LaunchExecutable")]
	launch_executable: String,
	#[serde(rename = "InstallLocation")]
	install_location: String,
	#[serde(rename = "CatalogItemId")]
	catalog_item_id: String,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EpicCatalogCategory {
	path: String,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EpicCatalogReleaseInfo {
	app_id: String,
	date_added: Option<String>,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EpicCatalogImage {
	height: i32,
	url: String,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EpicCatalogItem {
	id: String,
	namespace: String,
	title: String,
	categories: Vec<EpicCatalogCategory>,
	release_info: Vec<EpicCatalogReleaseInfo>,
	key_images: Vec<EpicCatalogImage>,
}

#[derive(Clone)]
pub struct Epic {}

impl Epic {
	fn get_game(catalog_item: &EpicCatalogItem) -> Option<Game> {
		if catalog_item
			.categories
			.iter()
			.all(|category| category.path != "games")
		{
			return None;
		}

		let mut game = Game::new(
			GameProviderId::Epic,
			catalog_item.id.clone(),
			catalog_item.title.clone(),
		);

		game.add_provider_command(
			ProviderCommandAction::Install,
			ProviderCommandKind::String(format!(
				"com.epicgames.launcher://apps/{}%3A{}%3A{}?action=install",
				catalog_item.namespace,
				catalog_item.id,
				catalog_item
					.release_info
					.first()
					.map(|release_info| release_info.app_id.clone())
					.unwrap_or_default(),
			)),
		)
		.add_provider_command(
			ProviderCommandAction::OpenInBrowser,
			ProviderCommandKind::String(format!(
				"https://store.epicgames.com/browse?{}",
				serde_urlencoded::to_string([("sortBy", "relevancy"), ("q", &catalog_item.title)])
					.ok_or_log("Failed to encode Epic browser URL query")?,
			)),
		);

		game.thumbnail_url = catalog_item.get_thumbnail_url();
		game.release_date_rfc3339 = catalog_item.get_release_date();

		Some(game)
	}
}

impl EpicCatalogItem {
	fn get_release_date(&self) -> Option<String> {
		Some(
			self.release_info
				.first()?
				.date_added
				.as_ref()?
				.parse::<chrono::DateTime<chrono::Utc>>()
				.ok_or_log("Failed to parse Epic release date")?
				.to_rfc3339(),
		)
	}

	fn get_thumbnail_url(&self) -> Option<String> {
		Some(
			self.key_images
				.iter()
				// The available images here seem to be a bit inconsistent, so we'll just pick the smallest one.
				.min_by_key(|image| image.height)?
				.url
				.clone(),
		)
	}
}

impl ProviderParser for Epic {
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
				if let Some(mut game) = Self::get_game(&catalog_item) {
					if let Some(exe_path) = exe_paths.remove(&game.external_id) {
						game.set_executable(&exe_path);
						game.add_provider_command(
							ProviderCommandAction::StartViaProvider,
							ProviderCommandKind::String(format!(
								"com.epicgames.launcher://apps/{}?action=launch&silent=true",
								game.external_id
							)),
						);
					}

					games.push(game);
				}
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
impl ProviderActions for Epic {
	fn get_games(&self) -> Result<Vec<Game>> {
		let app_data_path = match RegKey::predef(HKEY_LOCAL_MACHINE)
			.open_subkey(r"SOFTWARE\Epic Games\EpicGamesLauncher")
			.and_then(|launcher_reg| launcher_reg.get_value::<String, _>("AppDataPath"))
			.map(PathBuf::from)
		{
			Ok(path) => path,
			Err(err) => {
				log::info!(
					"Epic Games Launcher not found in registry. Probably means Epic Games Launcher isn't installed. Error: {err}"
				);
				return Ok(Vec::new());
			}
		};

		self.parse_games(&app_data_path)
	}
}

#[cfg(target_os = "windows")]
impl WineProviderActions for Epic {}

fn read_manifest(path: &Path) -> Result<EpicManifest> {
	let json = fs::read_to_string(path)?;
	let manifest = serde_json::from_str::<EpicManifest>(&json)
		.with_context(|| format!("failed to parse Epic manifest `{}`", path.display()))?;
	Ok(manifest)
}
