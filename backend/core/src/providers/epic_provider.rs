#![cfg(target_os = "windows")]

use std::{
	fs::{self, File},
	io::Read,
	path::{Path, PathBuf},
};

use base64::engine::general_purpose;
use log::error;
use rai_pal_proc_macros::serializable_struct;
use winreg::{enums::HKEY_LOCAL_MACHINE, RegKey};

use super::{
	provider::ProviderId,
	provider_command::{ProviderCommand, ProviderCommandAction},
};
use crate::{
	game::Game,
	installed_game::InstalledGame,
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
	fn get_installed_game(manifest_path: &Path) -> Option<InstalledGame> {
		match read_manifest(manifest_path) {
			Ok(manifest) => {
				let path =
					PathBuf::from(manifest.install_location).join(manifest.launch_executable);

				let mut game =
					InstalledGame::new(&path, &manifest.display_name, Self::ID.to_owned())?;
				game.set_start_command_string(&format!(
					"com.epicgames.launcher://apps/{}?action=launch&silent=true",
					manifest.app_name
				));

				Some(game)
			}
			Err(err) => {
				error!("Failed to parse epic manifest: {err}");
				None
			}
		}
	}

	// fn get_owned_game(catalog_item: &EpicCatalogItem) -> Option<OwnedGame> {
	// 	if catalog_item
	// 		.categories
	// 		.iter()
	// 		.all(|category| category.path != "games")
	// 	{
	// 		return None;
	// 	}

	// 	let mut game = OwnedGame::new(&catalog_item.id, *Self::ID, &catalog_item.title);

	// 	game.add_provider_command(
	// 		ProviderCommandAction::Install,
	// 		ProviderCommand::String(format!(
	// 			"com.epicgames.launcher://apps/{}%3A{}%3A{}?action=install",
	// 			catalog_item.namespace,
	// 			catalog_item.id,
	// 			catalog_item
	// 				.release_info
	// 				.first()
	// 				.map(|release_info| release_info.app_id.clone())
	// 				.unwrap_or_default(),
	// 		)),
	// 	)
	// 	.add_provider_command(
	// 		ProviderCommandAction::OpenInBrowser,
	// 		ProviderCommand::String(format!(
	// 			"https://store.epicgames.com/browse?{}",
	// 			serde_urlencoded::to_string([("sortBy", "relevancy"), ("q", &catalog_item.title)])
	// 				.ok()?,
	// 		)),
	// 	);

	// 	if let Some(thumbnail_url) = catalog_item.get_thumbnail_url() {
	// 		game.set_thumbnail_url(&thumbnail_url);
	// 	}

	// 	if let Some(release_date) = catalog_item.get_release_date() {
	// 		game.set_release_date(release_date);
	// 	}

	// 	Some(game)
	// }
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
	// async fn get_games<TInstalledCallback, TOwnedCallback>(
	// 	&self,
	// 	mut installed_callback: TInstalledCallback,
	// 	mut owned_callback: TOwnedCallback,
	// ) -> Result
	// where
	// 	TInstalledCallback: FnMut(InstalledGame) + Send + Sync,
	// 	TOwnedCallback: FnMut(OwnedGame) + Send + Sync,
	// {
	// 	let app_data_path = RegKey::predef(HKEY_LOCAL_MACHINE)
	// 		.open_subkey(r"SOFTWARE\WOW6432Node\Epic Games\EpicGamesLauncher")
	// 		.and_then(|launcher_reg| launcher_reg.get_value::<String, _>("AppDataPath"))
	// 		.map(PathBuf::from)?;

	// 	let manifests_path = app_data_path.join("Manifests");
	// 	if manifests_path.is_dir() {
	// 		let manifest_paths = glob_path(&manifests_path.join("*.item"));
	// 		for manifest_path in manifest_paths {
	// 			if let Some(installed_game) = Self::get_installed_game(&manifest_path) {
	// 				installed_callback(installed_game);
	// 			}
	// 		}
	// 	} else {
	// 		log::info!("Epic Games Launcher manifests folder not found. Probably means Epic Games Launcher isn't installed, or maybe user hasn't installed any games dunno.");
	// 	}

	// 	let catalog_path = app_data_path.join("Catalog").join("catcache.bin");
	// 	if catalog_path.is_file() {
	// 		let mut catalog_cache_file = File::open(catalog_path)?;
	// 		let mut decoder = base64::read::DecoderReader::new(
	// 			&mut catalog_cache_file,
	// 			&general_purpose::STANDARD,
	// 		);
	// 		let mut json = String::default();
	// 		decoder.read_to_string(&mut json)?;

	// 		let catalog = serde_json::from_str::<Vec<EpicCatalogItem>>(&json)?;
	// 		for catalog_item in catalog {
	// 			if let Some(owned_game) = Self::get_owned_game(&catalog_item) {
	// 				owned_callback(owned_game);
	// 			}
	// 		}
	// 	} else {
	// 		log::info!("Epic Games Launcher catalog cache file not found. Probably means user hasn't installed Epic Games Launcher, or the cache file hasn't been created yet.");
	// 	}

	// 	Ok(())
	// }

	async fn get_games_new<TCallback>(&self, callback: TCallback) -> Result
	where
		TCallback: FnMut(Game) + Send + Sync,
	{
		Ok(())
	}
}

fn read_manifest(path: &Path) -> Result<EpicManifest> {
	let json = fs::read_to_string(path)?;
	let manifest = serde_json::from_str::<EpicManifest>(&json)?;
	Ok(manifest)
}
