use std::{
	collections::HashSet,
	fs::{
		self,
		File,
	},
	io::Read,
	path::PathBuf,
};

use async_trait::async_trait;
use base64::{
	alphabet,
	engine::{
		self,
		general_purpose,
	},
	Engine as _,
};

use super::provider::ProviderId;
use crate::{
	installed_game::InstalledGame,
	owned_game::OwnedGame,
	provider::{
		ProviderActions,
		ProviderStatic,
	},
	serializable_enum,
	serializable_struct,
	Result,
};

pub struct EpicProvider {}

impl ProviderStatic for EpicProvider {
	const ID: &'static ProviderId = &ProviderId::Epic;

	fn new() -> Result<Self>
	where
		Self: Sized,
	{
		Ok(Self {})
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
impl ProviderActions for EpicProvider {
	fn get_installed_games(&self) -> Result<Vec<InstalledGame>> {
		Ok(game_scanner::epicgames::games()
			.unwrap_or_default()
			.iter()
			.filter_map(|game| {
				InstalledGame::new(
					game.path.as_ref()?,
					&game.name,
					Self::ID.to_owned(),
					None,
					None,
					None,
				)
			})
			.collect())
	}

	async fn get_owned_games(&self) -> Result<Vec<OwnedGame>> {
		// TODO get path
		let catalog_path =
			PathBuf::from(r"C:\ProgramData\Epic\EpicGamesLauncher\Data\Catalog\catcache.bin");

		println!("catalog_path");
		let mut file = File::open(catalog_path)?;

		let mut decoder = base64::read::DecoderReader::new(&mut file, &general_purpose::STANDARD);
		let mut json = String::default();
		decoder.read_to_string(&mut json)?;
		println!("decoder");

		let items = serde_json::from_str::<Vec<EpicCatalogItem>>(&json)?;
		println!("items");

		Ok(items
			.iter()
			.filter_map(|catalog_item| {
				if catalog_item
					.categories
					.iter()
					.all(|category| category.path != "games")
				{
					return None;
				}

				Some(OwnedGame {
					engine: None,
					game_mode: None,
					id: catalog_item.id.clone(),
					name: catalog_item.title.clone(),
					thumbnail_url: catalog_item.get_thumbnail_url().unwrap_or_default(),
					installed: false,
					os_list: HashSet::default(),
					provider_id: *Self::ID,
					release_date: catalog_item.get_release_date().unwrap_or(0),
					uevr_score: None,
				})
			})
			.collect())
	}
}
