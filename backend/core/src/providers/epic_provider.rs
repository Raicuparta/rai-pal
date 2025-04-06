#![cfg(target_os = "windows")]

use std::{
	collections::HashMap,
	fs::{self, File},
	io::Read,
	path::{Path, PathBuf},
};

use base64::engine::general_purpose;
use log::error;
use rai_pal_proc_macros::serializable_struct;
use winreg::{RegKey, enums::HKEY_LOCAL_MACHINE};

use super::{
	provider::ProviderId,
	provider_command::{ProviderCommand, ProviderCommandAction},
};
use crate::{
	game::{DbGame, InsertGame},
	game_executable::GameExecutable,
	paths::glob_path,
	providers::provider::{ProviderActions, ProviderStatic},
	result::Result,
};

#[serializable_struct]
pub struct EpicManifest {
	#[serde(rename = "DisplayName")]
	display_name: String,
	#[serde(rename = "LaunchExecutable")]
	launch_executable: String,
	#[serde(rename = "InstallLocation")]
	install_location: String,
	#[serde(rename = "CatalogNamespace")]
	catalog_namespace: String,
	#[serde(rename = "CatalogItemId")]
	catalog_item_id: String,
	#[serde(rename = "AppName")]
	app_name: String,
}

#[serializable_struct]
pub struct EpicCatalogCategory {
	path: String,
}

#[serializable_struct]
pub struct EpicCatalogReleaseInfo {
	app_id: String,
	platform: Vec<String>,
	date_added: Option<String>,
}

#[serializable_struct]
pub struct EpicCatalogImage {
	height: i32,
	url: String,
}

#[serializable_struct]
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
	fn get_game(catalog_item: &EpicCatalogItem) -> Option<DbGame> {
		if catalog_item
			.categories
			.iter()
			.all(|category| category.path != "games")
		{
			return None;
		}

		let mut game = DbGame::new(
			*Self::ID,
			catalog_item.id.clone(),
			catalog_item.title.clone(),
		);

		game.add_provider_command(
			ProviderCommandAction::Install,
			ProviderCommand::String(format!(
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
			ProviderCommand::String(format!(
				"https://store.epicgames.com/browse?{}",
				serde_urlencoded::to_string([("sortBy", "relevancy"), ("q", &catalog_item.title)])
					.ok()?,
			)),
		);

		game.thumbnail_url = catalog_item.get_thumbnail_url();
		game.release_date = catalog_item.get_release_date();

		Some(game)
	}
}

impl ProviderStatic for Epic {
	const ID: &'static ProviderId = &ProviderId::Epic;

	fn new() -> Result<Self>
	where
		Self: Sized,
	{
		Ok(Self {})
	}
}

impl EpicCatalogItem {
	fn get_release_date(&self) -> Option<i64> {
		Some(
			self.release_info
				.first()?
				.date_added
				.as_ref()?
				.parse::<chrono::DateTime<chrono::Utc>>()
				.ok()?
				.timestamp(),
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

impl ProviderActions for Epic {
	async fn insert_games(&self, pool: &sqlx::Pool<sqlx::Sqlite>) -> Result {
		let app_data_path = RegKey::predef(HKEY_LOCAL_MACHINE)
			.open_subkey(r"SOFTWARE\WOW6432Node\Epic Games\EpicGamesLauncher")
			.and_then(|launcher_reg| launcher_reg.get_value::<String, _>("AppDataPath"))
			.map(PathBuf::from)?;

		let mut executables: HashMap<String, GameExecutable> = HashMap::new();

		let manifests_path = app_data_path.join("Manifests");
		if manifests_path.is_dir() {
			let manifest_paths = glob_path(&manifests_path.join("*.item"));
			for manifest_path in manifest_paths {
				match read_manifest(&manifest_path) {
					Ok(manifest) => {
						let path = PathBuf::from(manifest.install_location)
							.join(manifest.launch_executable);

						if let Some(executable) = GameExecutable::new(&path) {
							executables.insert(manifest.catalog_item_id.clone(), executable);
						}
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
			let mut catalog_cache_file = File::open(catalog_path)?;
			let mut decoder = base64::read::DecoderReader::new(
				&mut catalog_cache_file,
				&general_purpose::STANDARD,
			);
			let mut json = String::default();
			decoder.read_to_string(&mut json)?;

			let catalog = serde_json::from_str::<Vec<EpicCatalogItem>>(&json)?;
			for catalog_item in catalog {
				if let Some(mut game) = Self::get_game(&catalog_item) {
					if let Some(executable) = executables.remove(&game.external_id) {
						game.set_executable(&executable);
						game.add_provider_command(
							ProviderCommandAction::StartViaProvider,
							ProviderCommand::String(format!(
								"com.epicgames.launcher://apps/{}?action=launch&silent=true",
								&game.external_id
							)),
						);
					}

					pool.insert_game(&game).await?; // TODO prevent whole thing crashing if one game fails to insert.
				}
			}
		} else {
			log::info!(
				"Epic Games Launcher catalog cache file not found. Probably means user hasn't installed Epic Games Launcher, or the cache file hasn't been created yet."
			);
		}

		Ok(())
	}
}

fn read_manifest(path: &Path) -> Result<EpicManifest> {
	let json = fs::read_to_string(path)?;
	let manifest = serde_json::from_str::<EpicManifest>(&json)?;
	Ok(manifest)
}
