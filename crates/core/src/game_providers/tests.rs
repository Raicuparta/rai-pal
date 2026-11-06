#![allow(clippy::unwrap_used)]

use std::{
	fs,
	path::{Path, PathBuf},
};

use crate::{
	game::DbGame,
	game_providers::{
		game_provider::ProviderParser, itch_provider::Itch, steam::steam_provider::Steam,
	},
};

#[cfg(target_os = "linux")]
use crate::game_providers::{
	heroic_amazon_provider::HeroicAmazon, heroic_epic_provider::HeroicEpic,
	heroic_gog_provider::HeroicGog,
};

use crate::game_providers::{
	epic_provider::Epic,
	gog_provider::{Gog, GogParseSource},
};

fn fixture(name: &str) -> PathBuf {
	let path = Path::new(env!("CARGO_MANIFEST_DIR"))
		.join("tests/fixtures")
		.join(name);

	assert!(
		path.exists(),
		"missing provider fixture: {}. Add real provider files here.",
		path.display()
	);

	path
}

fn expected_games(root: &Path) -> Vec<String> {
	let path = root.join("expected.json");
	let json = fs::read_to_string(&path).unwrap_or_else(|error| {
		panic!(
			"failed to read expected games from `{}`: {error}",
			path.display()
		)
	});

	serde_json::from_str(&json).unwrap_or_else(|error| {
		panic!(
			"failed to parse expected games from `{}`: {error}",
			path.display()
		)
	})
}

fn parse_path_fixture<TParser>(parser: &TParser, name: &str) -> (PathBuf, Vec<DbGame>)
where
	TParser: ProviderParser<Source = PathBuf>,
{
	let root = fixture(name);
	let mut games = Vec::new();
	parser
		.parse_games(&root, &mut |game| games.push(game))
		.expect("provider parse failed");
	(root, games)
}

fn find_game<'a>(root: &Path, games: &'a [DbGame], external_id: &str) -> &'a DbGame {
	games
		.iter()
		.find(|game| game.external_id == external_id)
		.unwrap_or_else(|| {
			panic!(
				"expected game `{external_id}` not found in `{}`. Parsed games: {:#?}",
				root.display(),
				games
					.iter()
					.map(|game| (&game.external_id, &game.display_title))
					.collect::<Vec<_>>()
			)
		})
}

fn assert_expected_games(root: &Path, games: &[DbGame]) {
	let expected = expected_games(root);

	assert!(
		!expected.is_empty(),
		"expected.json for `{}` must list at least one game",
		root.display()
	);

	for external_id in expected {
		find_game(root, games, &external_id);
	}
}

#[test]
fn steam_appinfo_basic() {
	let (root, games) = parse_path_fixture(&Steam, "steam/appinfo-basic");
	assert_expected_games(&root, &games);
}

#[test]
fn itch_database_basic() {
	let root = fixture("itch/database-basic");
	let db_path = root.join("db/butler.db");
	let mut games = Vec::new();
	Itch {}
		.parse_games(&db_path, &mut |game| games.push(game))
		.expect("provider parse failed");
	assert_expected_games(&root, &games);
}

#[cfg(target_os = "linux")]
#[test]
fn heroic_epic_library_basic() {
	let (root, games) = parse_path_fixture(&HeroicEpic, "heroic-epic/library-basic");
	assert_expected_games(&root, &games);
}

#[cfg(target_os = "linux")]
#[test]
fn heroic_gog_library_basic() {
	let (root, games) = parse_path_fixture(&HeroicGog {}, "heroic-gog/library-basic");
	assert_expected_games(&root, &games);
}

#[cfg(target_os = "linux")]
#[test]
fn heroic_amazon_library_basic() {
	let (root, games) = parse_path_fixture(&HeroicAmazon, "heroic-amazon/library-basic");
	assert_expected_games(&root, &games);
}

#[test]
fn epic_catalog_basic() {
	let (root, games) = parse_path_fixture(&Epic {}, "epic/catalog-basic");
	assert_expected_games(&root, &games);
}

#[test]
fn gog_with_installed() {
	let root = fixture("gog/with-installed");
	let source = GogParseSource {
		database_path: root.join("galaxy-2.0.db"),
		launcher_path: root.join("GalaxyClient.exe"),
	};
	let mut games = Vec::new();
	Gog {}
		.parse_games(&source, &mut |game| games.push(game))
		.expect("provider parse failed");
	assert_expected_games(&root, &games);
}

#[test]
fn gog_without_installed() {
	let root = fixture("gog/without-installed");
	let source = GogParseSource {
		database_path: root.join("galaxy-2.0.db"),
		launcher_path: root.join("GalaxyClient.exe"),
	};
	let mut games = Vec::new();
	Gog {}
		.parse_games(&source, &mut |game| games.push(game))
		.expect("provider parse failed");
	assert_expected_games(&root, &games);
}
