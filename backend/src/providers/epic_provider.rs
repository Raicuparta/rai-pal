use std::{
	collections::HashSet,
	fs::File,
	io::Read,
	path::PathBuf,
};

use async_trait::async_trait;
use base64::engine::general_purpose;
use winreg::{
	enums::HKEY_LOCAL_MACHINE,
	RegKey,
};

use super::{
	provider::{
		self,
		ProviderId,
	},
	provider_command::ProviderCommand,
};
use crate::{
	game_engines::game_engine::GameEngine,
	installed_game::InstalledGame,
	owned_game::OwnedGame,
	pc_gaming_wiki,
	provider::{
		ProviderActions,
		ProviderStatic,
	},
	serializable_struct,
	Result,
};

pub struct Epic {
	app_data_path: PathBuf,
	engine_cache: provider::EngineCache,
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

		let engine_cache = Self::try_get_engine_cache();

		Ok(Self {
			app_data_path,
			engine_cache,
		})
	}
}

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
		// TODO stop using game_scanner,
		// just implement it here since I have to make so many changes anyway.
		Ok(game_scanner::epicgames::games()
			.unwrap_or_default()
			.iter()
			.filter_map(|manifest_entry| {
				let path = manifest_entry.path.as_ref()?;
				let mut game = InstalledGame::new(path, &manifest_entry.name, Self::ID.to_owned())?;
				game.set_start_command_string(&format!(
					"com.epicgames.launcher://apps/{}?action=launch&silent=true",
					manifest_entry.id
				));

				Some(game)
			})
			.collect())
	}

	async fn get_owned_games(&self) -> Result<Vec<OwnedGame>> {
		let mut file = File::open(self.app_data_path.join("Catalog").join("catcache.bin"))?;

		let mut decoder = base64::read::DecoderReader::new(&mut file, &general_purpose::STANDARD);
		let mut json = String::default();
		decoder.read_to_string(&mut json)?;

		let items = serde_json::from_str::<Vec<EpicCatalogItem>>(&json)?;

		let owned_games = futures::future::join_all(items.iter().map(|catalog_item| async {
			if catalog_item
				.categories
				.iter()
				.all(|category| category.path != "games")
			{
				return None;
			}

			Some(OwnedGame {
				engine: get_engine(&catalog_item.title, &self.engine_cache).await,
				game_mode: None,
				id: catalog_item.id.clone(),
				name: catalog_item.title.clone(),
				thumbnail_url: catalog_item.get_thumbnail_url().unwrap_or_default(),
				installed: false, // TODO
				os_list: HashSet::default(),
				provider_id: *Self::ID,
				release_date: catalog_item.get_release_date().unwrap_or(0),
				uevr_score: None,
				show_library_command: None,
				open_page_command: None,
				install_command: Some(ProviderCommand::String(format!(
					"com.epicgames.launcher://apps/{}%3A{}%3A{}?action=install",
					catalog_item.namespace,
					catalog_item.id,
					catalog_item
						.release_info
						.first()
						.map(|release_info| release_info.app_id.clone())
						.unwrap_or_default(),
				))),
			})
		}))
		.await
		.into_iter()
		.flatten();

		Self::try_save_engine_cache(
			&owned_games
				.clone()
				.map(|owned_game| (owned_game.name.clone(), owned_game.engine))
				.collect(),
		);

		Ok(owned_games.collect())
	}
}

async fn get_engine(title: &str, cache: &provider::EngineCache) -> Option<GameEngine> {
	if let Some(cached_engine) = cache.get(title) {
		return cached_engine.clone();
	}

	pc_gaming_wiki::get_engine_from_game_title(title).await
}
