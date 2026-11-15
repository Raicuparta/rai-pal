use std::path::PathBuf;

use crate::{
	game::Game,
	game_providers::{
		game_provider::{GameProviderId, ProviderParser},
		provider_command::{ProviderCommandAction, ProviderCommandKind},
	},
	result::{LogErrExt, Result},
};

#[cfg(target_os = "windows")]
use crate::game_providers::game_provider::{ProviderActions, WineProviderActions};

pub mod legacy;
pub mod modern;

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

impl EpicCatalogItem {
	pub fn is_game(&self) -> bool {
		self.categories
			.iter()
			.any(|category| category.path == "games")
	}

	pub fn to_game(&self) -> Option<Game> {
		let mut game = new_game(&self.namespace, &self.id, self.artifact_id(), &self.title)?;

		game.thumbnail_url = self.get_thumbnail_url();
		game.release_date_rfc3339 = self.get_release_date();

		Some(game)
	}

	fn artifact_id(&self) -> &str {
		self.release_info
			.first()
			.map(|release_info| release_info.app_id.as_str())
			.unwrap_or_default()
	}

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

pub fn new_game(namespace: &str, item_id: &str, artifact_id: &str, title: &str) -> Option<Game> {
	let mut game = Game::new(GameProviderId::Epic, item_id.to_string(), title.to_string());

	game.add_provider_command(
		ProviderCommandAction::Install,
		ProviderCommandKind::String(format!(
			"com.epicgames.launcher://apps/{namespace}%3A{item_id}%3A{artifact_id}?action=install",
		)),
	)
	.add_provider_command(
		ProviderCommandAction::OpenInBrowser,
		ProviderCommandKind::String(format!(
			"https://store.epicgames.com/browse?{}",
			serde_urlencoded::to_string([("sortBy", "relevancy"), ("q", title)])
				.ok_or_log("Failed to encode Epic browser URL query")?,
		)),
	);

	Some(game)
}

pub fn add_start_via_provider_command(game: &mut Game) {
	game.add_provider_command(
		ProviderCommandAction::StartViaProvider,
		ProviderCommandKind::String(format!(
			"com.epicgames.launcher://apps/{}?action=launch&silent=true",
			game.external_id
		)),
	);
}

#[derive(Clone)]
pub struct Epic {}

impl Epic {
	pub fn parse_games_from_sources(
		modern_data_path: Option<PathBuf>,
		legacy_app_data_path: Option<PathBuf>,
	) -> Result<Vec<Game>> {
		if let Some(path) = modern_data_path {
			match (modern::ModernEpic {}).parse_games(&path) {
				Ok(games) if !games.is_empty() => return Ok(games),
				Ok(_) => log::info!(
					"Epic Games Launcher v2 found, but it has no games. Trying the legacy launcher data."
				),
				Err(err) => log::error!("Failed to parse Epic Games Launcher v2 data: {err}"),
			}
		}

		legacy_app_data_path.map_or_else(
			|| Ok(Vec::new()),
			|path| (legacy::LegacyEpic {}).parse_games(&path),
		)
	}
}

#[cfg(target_os = "windows")]
impl ProviderActions for Epic {
	fn get_games(&self) -> Result<Vec<Game>> {
		Self::parse_games_from_sources(modern::data_path(), legacy::app_data_path())
	}
}

#[cfg(target_os = "windows")]
impl WineProviderActions for Epic {}
