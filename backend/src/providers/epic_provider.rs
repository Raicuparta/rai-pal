#![cfg(target_os = "windows")]

use std::{
	fs::{
		self,
		File,
	},
	io::Read,
	path::PathBuf,
};

use async_trait::async_trait;
use base64::engine::general_purpose;
use log::error;
use winreg::{
	enums::HKEY_LOCAL_MACHINE,
	RegKey,
};

use super::{
	provider::ProviderId,
	provider_command::{
		ProviderCommand,
		ProviderCommandAction,
	},
};
use crate::{
	installed_game::InstalledGame,
	owned_game::OwnedGame,
	paths::glob_path,
	pc_gaming_wiki,
	provider::{
		ProviderActions,
		ProviderStatic,
	},
	remote_game::{
		self,
		RemoteGame,
	},
	serializable_struct,
	Result,
};

#[derive(Clone)]
pub struct Epic {
	app_data_path: PathBuf,
	catalog: Vec<EpicCatalogItem>,
	remote_game_cache: remote_game::Map,
}

impl ProviderStatic for Epic {
	const ID: &'static ProviderId = &ProviderId::Epic;

	fn new() -> Result<Self>
	where
		Self: Sized,
	{
		let app_data_path = RegKey::predef(HKEY_LOCAL_MACHINE)
			.open_subkey(r"SOFTWARE\WOW6432Node\Epic Games\EpicGamesLauncher")
			.and_then(|launcher_reg| launcher_reg.get_value::<String, _>("AppDataPath"))
			.map(PathBuf::from)?;

		let remote_game_cache = Self::try_get_remote_game_cache();

		let mut file = File::open(app_data_path.join("Catalog").join("catcache.bin"))?;

		let mut decoder = base64::read::DecoderReader::new(&mut file, &general_purpose::STANDARD);
		let mut json = String::default();
		decoder.read_to_string(&mut json)?;

		let catalog = serde_json::from_str::<Vec<EpicCatalogItem>>(&json)?;

		Ok(Self {
			app_data_path,
			catalog,
			remote_game_cache,
		})
	}
}

serializable_struct!(EpicManifest {
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
});

serializable_struct!(EpicCatalogCategory { path: String });

serializable_struct!(EpicCatalogReleaseInfo {
	app_id: String,
	platform: Vec<String>,
	date_added: Option<String>,
});

serializable_struct!(EpicCatalogImage {
	height: i32,
	url: String,
});

serializable_struct!(EpicCatalogItem {
	id: String,
	namespace: String,
	title: String,
	categories: Vec<EpicCatalogCategory>,
	release_info: Vec<EpicCatalogReleaseInfo>,
	key_images: Vec<EpicCatalogImage>,
});

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

#[async_trait]
impl ProviderActions for Epic {
	fn get_installed_games(&self) -> Result<Vec<InstalledGame>> {
		let manifests = glob_path(&self.app_data_path.join("Manifests").join("*.item"))?;

		Ok(manifests
			.iter()
			.filter_map(
				|manifest_path_result| match read_manifest(manifest_path_result) {
					Ok(manifest) => {
						let path = PathBuf::from(manifest.install_location)
							.join(manifest.launch_executable);

						let mut game =
							InstalledGame::new(&path, &manifest.display_name, Self::ID.to_owned())?;
						game.set_start_command_string(&format!(
							"com.epicgames.launcher://apps/{}?action=launch&silent=true",
							manifest.app_name
						));
						game.set_provider_game_id(&manifest.catalog_item_id);

						Some(game)
					}
					Err(err) => {
						error!("Failed to parse manifest: {err}");
						None
					}
				},
			)
			.collect())
	}

	fn get_owned_games(&self) -> Result<Vec<OwnedGame>> {
		let owned_games = self.catalog.iter().filter_map(|catalog_item| {
			if catalog_item
				.categories
				.iter()
				.all(|category| category.path != "games")
			{
				return None;
			}

			let mut game = OwnedGame::new(&catalog_item.id, *Self::ID, &catalog_item.title);

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
					serde_urlencoded::to_string([
						("sortBy", "relevancy"),
						("q", &catalog_item.title)
					])
					.ok()?,
				)),
			);

			if let Some(thumbnail_url) = catalog_item.get_thumbnail_url() {
				game.set_thumbnail_url(&thumbnail_url);
			}

			if let Some(release_date) = catalog_item.get_release_date() {
				game.set_release_date(release_date);
			}

			Some(game)
		});

		Ok(owned_games.collect())
	}

	async fn get_remote_games(&self) -> Result<Vec<RemoteGame>> {
		let remote_games: Vec<RemoteGame> =
			futures::future::join_all(self.catalog.iter().map(|catalog_item| async {
				let mut remote_game = RemoteGame::new(*Self::ID, &catalog_item.id);

				if let Some(cached_remote_game) = self.remote_game_cache.get(&remote_game.id) {
					return cached_remote_game.clone();
				}

				match pc_gaming_wiki::get_engine_from_game_title(&catalog_item.title).await {
					Ok(Some(engine)) => {
						remote_game.set_engine(engine);
					}
					Ok(None) => {}
					Err(_) => {
						remote_game.set_skip_cache(true);
					}
				}

				remote_game
			}))
			.await;

		Self::try_save_remote_game_cache(&remote_games);

		Ok(remote_games)
	}
}

fn read_manifest(path: &PathBuf) -> Result<EpicManifest> {
	let json = fs::read_to_string(path)?;
	let manifest = serde_json::from_str::<EpicManifest>(&json)?;
	Ok(manifest)
}
