use lazy_regex::{regex_captures, regex_replace, regex_replace_all};
use log::error;
use rai_pal_proc_macros::serializable_struct;

use crate::{
	game_engines::game_engine::{EngineBrand, EngineVersion, EngineVersionNumbers, GameEngine},
	Result,
};

#[serializable_struct]
pub struct PCGamingWikiTitle {
	#[serde(rename = "Engine")]
	engine: Option<String>,
	#[serde(rename = "Build")]
	build: Option<String>,
}

#[serializable_struct]
pub struct PCGamingWikiQueryItem {
	title: PCGamingWikiTitle,
}

#[serializable_struct]
pub struct PCGamingWikiQueryResponse {
	cargoquery: Vec<PCGamingWikiQueryItem>,
}

// We could try to parse more version parts here, but I find that engine versions
// on PCGamingWiki are very commonly outdated. So if we just do major+minor, it's
// less likely to be outdated.
fn parse_version(version_text: &str) -> Option<EngineVersion> {
	let (_, major, _, minor) = regex_captures!(r".*?(\d+)(\.(\d+))?.*", version_text)?;

	Some(EngineVersion {
		display: if minor.is_empty() {
			major.to_string()
		} else {
			[major, minor].join(".")
		},
		numbers: EngineVersionNumbers {
			major: major.parse::<u32>().ok()?,
			minor: minor.parse::<u32>().ok(),
			patch: None,
		},
		suffix: None,
	})
}

pub async fn get_engine(where_query: &str) -> Result<Option<GameEngine>> {
	let url = format!(
		"https://www.pcgamingwiki.com/w/api.php?{}",
		serde_urlencoded::to_string([
			("action", "cargoquery"),
			("tables", "Infobox_game,Infobox_game_engine"),
			(
				"fields",
				"Infobox_game_engine.Engine,Infobox_game_engine.Build",
			),
			("where", where_query),
			(
				"join on",
				"Infobox_game._pageName = Infobox_game_engine._pageName",
			),
			("format", "json"),
		])?
	);

	Ok(
		match reqwest::get(url)
			.await?
			.json::<PCGamingWikiQueryResponse>()
			.await
		{
			Ok(parsed_response) => {
				parsed_response
					.cargoquery
					.into_iter()
					// This has high potential for false positives, since we search by title kinda fuzzily,
					// and then just pick the first one with an engine.
					.find_map(|item| Some((item.title.engine?, item.title.build)))
					.and_then(|(engine, build)| {
						let version = build
							.and_then(|version_text| parse_version(&version_text))
							// On PCGamingWiki, each Unreal major version is considered a separate engine.
							// So we can parse the engine name to get the major version.
							.or_else(|| parse_version(&engine));

						// I don't feel like figuring out the exact format,
						// since it can sometimes have the engine version included, sometimes not.
						if engine.contains("Unreal") {
							Some(GameEngine {
								brand: EngineBrand::Unreal,
								version,
							})
						} else if engine.contains("Unity") {
							Some(GameEngine {
								brand: EngineBrand::Unity,
								version,
							})
						} else if engine.contains("Godot") {
							Some(GameEngine {
								brand: EngineBrand::Godot,
								version,
							})
						} else if engine.contains("GameMaker") {
							Some(GameEngine {
								brand: EngineBrand::GameMaker,
								version,
							})
						} else {
							None
						}
					})
			}
			Err(err) => {
				error!("Error parsing PCGamingWiki response: {err}");
				None
			}
		},
	)
}

// Since there's no way to get a game by every provider ID from PCGamingWiki, we try with the game title.
pub async fn get_engine_from_game_title(title: &str) -> Result<Option<GameEngine>> {
	// Use only ascii and lowercase to make it easier to find by title.
	let lowercase_title = title.to_ascii_lowercase();

	// Remove "demo" suffix so that demos can match with the main game.
	#[allow(clippy::trivial_regex)]
	let non_demo_title = regex_replace!(r" demo$", &lowercase_title, "");

	// Replace anything that isn't alphanumeric with a % character, the wildcard for the LIKE query.
	// This way we can still match even if the game has slight differences in the presented title punctuation.
	// Problem is, this can easily cause false flags (and it does).
	let clean_title = regex_replace_all!(r"[^a-zA-Z0-9]+", &non_demo_title, "%");

	// Finally do the query by page title.
	get_engine(&format!("Infobox_game._pageName LIKE \"{clean_title}\"")).await
}
