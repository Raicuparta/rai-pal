#![cfg(target_os = "linux")]

use std::{
	collections::HashMap,
	fmt::Debug,
	path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use super::provider_command::{ProviderCommand, ProviderCommandAction};
use crate::{
	game::DbGame,
	game_providers::{
		game_provider::{GameProviderId, ProviderActions, ProviderParser, WineProviderActions},
		heroic_provider,
	},
	local_database::{app_database::DbMutex, game_database::GameDatabase},
	result::Result,
};

#[derive(Debug, Serialize, Deserialize)]
struct ParsedGame {
	app_name: String,
	title: String,
	install: Option<Install>,
	art_cover: Option<String>,
	store_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Install {
	executable: Option<String>,
	install_path: Option<PathBuf>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Root {
	library: Option<Vec<ParsedGame>>,
}

#[derive(Debug, Deserialize)]
struct LegendaryMetadata {
	app_name: String,
	metadata: LegendaryMetadataInner,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LegendaryMetadataInner {
	#[serde(default)]
	release_info: Vec<LegendaryReleaseInfo>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LegendaryReleaseInfo {
	date_added: Option<String>,
}

fn get_detected_games(root: &Path) -> Result<Option<Vec<ParsedGame>>> {
	Ok(
		heroic_provider::read_json_from::<Root>(root, "store_cache/legendary_library.json")?
			.and_then(|root| root.library),
	)
}

#[derive(Clone)]
pub struct HeroicEpic;

impl HeroicEpic {
	fn get_exe_path(entry: &ParsedGame) -> Option<PathBuf> {
		let install = entry.install.as_ref()?;
		let game_path = install.install_path.as_ref()?;
		Some(game_path.join(install.executable.as_ref()?))
	}

	// Legendary caches Epic metadata here, and `releaseInfo` is the only local
	// source of release dates.
	fn get_release_dates(root: &Path) -> Result<HashMap<String, String>> {
		let mut release_dates = HashMap::new();

		for metadata in heroic_provider::read_json_files_from::<LegendaryMetadata>(
			root,
			"legendaryConfig/legendary/metadata",
		)? {
			let earliest_release_date = metadata
				.metadata
				.release_info
				.iter()
				.filter_map(|release_info| release_info.date_added.as_deref())
				.filter_map(heroic_provider::parse_release_date)
				.min();

			if let Some(release_date) = earliest_release_date {
				release_dates.insert(metadata.app_name, release_date);
			}
		}

		Ok(release_dates)
	}
}

impl ProviderParser for HeroicEpic {
	type Source = PathBuf;

	fn parse_games(&self, root: &Self::Source, insert: &mut impl FnMut(DbGame)) -> Result {
		let release_dates = Self::get_release_dates(root)?;
		if let Some(parsed_games) = get_detected_games(root)? {
			for parsed_game in parsed_games {
				let mut game = DbGame::new(
					GameProviderId::Epic,
					parsed_game.app_name.clone(),
					parsed_game.title.clone(),
				);
				if let Some(art_cover) = &parsed_game.art_cover {
					game.thumbnail_url = Some(format!("{art_cover}?h=100&resize=1"));
				}
				game.release_date_rfc3339 = release_dates.get(&parsed_game.app_name).cloned();

				if let Some(exe_path) = Self::get_exe_path(&parsed_game) {
					game.set_executable(&exe_path);
					game.add_provider_command(
						ProviderCommandAction::StartViaProvider,
						heroic_provider::launch_command(&parsed_game.app_name, Some("legendary")),
					);
				}

				if let Some(store_url) = &parsed_game.store_url {
					game.add_provider_command(
						ProviderCommandAction::OpenInBrowser,
						ProviderCommand::String(store_url.clone()),
					);
				}

				insert(game);
			}
		}
		Ok(())
	}
}

impl ProviderActions for HeroicEpic {
	fn insert_games(&self, db: &DbMutex) -> Result {
		self.parse_games(&heroic_provider::config_path()?, &mut |game| {
			db.insert_game(&game);
		})
	}
}

impl WineProviderActions for HeroicEpic {
	fn get_wine_prefix_path(&self, game: &DbGame) -> Result<PathBuf> {
		heroic_provider::get_wine_prefix_path(&game.external_id)
	}

	fn set_wine_dll_overrides(&self, game: &DbGame, dll_overrides: &[String]) -> Result {
		let prefix_path = self.get_wine_prefix_path(game)?;
		crate::wine::set_wine_dll_overrides_in_reg(&prefix_path, dll_overrides)
	}
}
