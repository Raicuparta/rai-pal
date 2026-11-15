use std::{
	collections::{HashMap, HashSet},
	path::{Path, PathBuf},
};

use fs_err as fs;

use crate::{
	game::Game,
	game_providers::{
		epic::{EpicCatalogItem, add_start_via_provider_command, new_game},
		game_provider::ProviderParser,
	},
	path_extensions::PathExt,
	result::{ContextResult, Result},
};

#[derive(serde::Deserialize)]
struct LibraryCache {
	items: Vec<LibraryItem>,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct LibraryItem {
	app_id: EpicAppId,
	title: String,
	owned: bool,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct EpicAppId {
	namespace_id: String,
	item_id: String,
	app_name: String,
}

#[derive(serde::Deserialize)]
struct CatalogCache {
	items: HashMap<String, CatalogCacheEntry>,
}

#[derive(serde::Deserialize)]
struct CatalogCacheEntry {
	item: EpicCatalogItem,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct InstallationRecord {
	installation_id: String,
	complete: Option<bool>,
	dir: Option<String>,
	item_id: Option<String>,
	launch_executable: Option<String>,
}

struct Installation {
	dir: PathBuf,
	launch_executable: String,
}

pub struct ModernEpic {}

impl ProviderParser for ModernEpic {
	type Source = PathBuf;

	fn parse_games(&self, data_path: &Self::Source) -> Result<Vec<Game>> {
		let catalog = read_catalog_cache(&data_path.join("cache/prod/catalog-cache.json"))?;
		let installations = read_installations(&data_path.join("installations.jsonl"));

		let mut games = Vec::new();
		let mut seen_item_ids = HashSet::new();

		for library_item in read_library(&data_path.join("cache/prod/account"))? {
			if !library_item.owned || !seen_item_ids.insert(library_item.app_id.item_id.clone()) {
				continue;
			}

			let catalog_item = catalog.get(&library_item.app_id.item_id);
			if catalog_item.is_some_and(|catalog_item| !catalog_item.is_game()) {
				continue;
			}

			let game = match catalog_item {
				Some(catalog_item) => catalog_item.to_game(),
				None => new_game(
					&library_item.app_id.namespace_id,
					&library_item.app_id.item_id,
					&library_item.app_id.app_name,
					&library_item.title,
				),
			};

			let Some(mut game) = game else {
				continue;
			};

			if let Some(installation) = installations.get(&library_item.app_id.item_id) {
				game.set_executable(&installation.dir.join(&installation.launch_executable));
				add_start_via_provider_command(
					&mut game,
					&library_item.app_id.namespace_id,
					&library_item.app_id.item_id,
					&library_item.app_id.app_name,
				);
			}

			games.push(game);
		}

		Ok(games)
	}
}

#[cfg(target_os = "windows")]
pub fn data_path() -> Option<PathBuf> {
	let path = PathBuf::from(std::env::var_os("LOCALAPPDATA")?).join("com.epicgames.eglv2");
	path.is_dir().then_some(path)
}

fn read_catalog_cache(catalog_path: &Path) -> Result<HashMap<String, EpicCatalogItem>> {
	if !catalog_path.is_file() {
		log::info!(
			"Epic Games Launcher catalog cache not found at `{}`. Game metadata will be missing.",
			catalog_path.display()
		);
		return Ok(HashMap::new());
	}

	let json = fs::read_to_string(catalog_path)?;
	let cache = serde_json::from_str::<CatalogCache>(&json).with_context(|| {
		format!(
			"failed to parse Epic Games Launcher catalog cache `{}`",
			catalog_path.display()
		)
	})?;

	Ok(cache
		.items
		.into_iter()
		.map(|(item_id, entry)| (item_id, entry.item))
		.collect())
}

fn read_library(account_path: &Path) -> Result<Vec<LibraryItem>> {
	let mut items = Vec::new();

	for library_path in account_path.join("*").join("library-cache.json").glob() {
		let json = fs::read_to_string(&library_path)?;
		let cache = serde_json::from_str::<LibraryCache>(&json).with_context(|| {
			format!(
				"failed to parse Epic Games Launcher library cache `{}`",
				library_path.display()
			)
		})?;
		items.extend(cache.items);
	}

	if items.is_empty() {
		log::info!(
			"Epic Games Launcher library cache not found in `{}`. Probably means the user hasn't signed in yet.",
			account_path.display()
		);
	}

	Ok(items)
}

fn read_installations(installations_path: &Path) -> HashMap<String, Installation> {
	let mut records: HashMap<String, InstallationRecord> = HashMap::new();

	if installations_path.is_file() {
		match fs::read_to_string(installations_path) {
			Ok(jsonl) => {
				for line in jsonl.lines() {
					let Ok(record) = serde_json::from_str::<InstallationRecord>(line) else {
						continue;
					};
					records.insert(record.installation_id.clone(), record);
				}
			}
			Err(err) => log::error!("Failed to read Epic installations file: {err}"),
		}
	}

	records
		.into_values()
		.filter(|record| record.complete == Some(true))
		.filter_map(|record| {
			Some((
				record.item_id?,
				Installation {
					dir: PathBuf::from(record.dir?),
					launch_executable: record.launch_executable?,
				},
			))
		})
		.collect()
}
