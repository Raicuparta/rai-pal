use lazy_regex::{
	regex_captures,
	regex_replace,
	regex_replace_all,
};
use log::error;

use crate::{
	game_engines::game_engine::{
		GameEngine,
		GameEngineBrand,
		GameEngineVersion,
	},
	serializable_struct,
};

serializable_struct!(PCGamingWikiTitle {
	#[serde(rename = "Engine")]
	engine: Option<String>,
	#[serde(rename = "Build")]
	build: Option<String>,
});

serializable_struct!(PCGamingWikiQueryItem {
	title: PCGamingWikiTitle,
});

serializable_struct!(PCGamingWikiQueryResponse {
	cargoquery: Vec<PCGamingWikiQueryItem>,
});

// We could try to parse more version parts here, but I find that engine versions
// on PCGamingWiki are very commonly outdated. So if we just do major+minor, it's
// less likely to be outdated.
fn parse_version(version_text: &str) -> Option<GameEngineVersion> {
	let (_, major, _, minor) = regex_captures!(r".*?(\d+)(\.(\d+))?.*", version_text)?;

	Some(GameEngineVersion {
		display: if minor.is_empty() {
			major.to_string()
		} else {
			[major, minor].join(".")
		},
		major: major.parse::<u32>().ok()?,
		minor: minor.parse::<u32>().ok().unwrap_or(0),
		patch: 0,
		suffix: None,
	})
}

pub async fn get_engine(where_query: &str) -> Option<GameEngine> {
	let url = format!("https://www.pcgamingwiki.com/w/api.php?action=cargoquery&tables=Infobox_game,Infobox_game_engine&fields=Infobox_game_engine.Engine,Infobox_game_engine.Build&where={where_query}&format=json&join%20on=Infobox_game._pageName%20=%20Infobox_game_engine._pageName");

	let result = reqwest::get(url).await;

	match result {
		Ok(response) => {
			match response.json::<PCGamingWikiQueryResponse>().await {
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
									brand: GameEngineBrand::Unreal,
									version,
								})
							} else if engine.contains("Unity") {
								Some(GameEngine {
									brand: GameEngineBrand::Unity,
									version,
								})
							} else if engine.contains("Godot") {
								Some(GameEngine {
									brand: GameEngineBrand::Godot,
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
			}
		}
		Err(_) => {
			// We ignore this error, since it's very likely to happen.
			// TODO: need to distinguish between a network error (pc offline) and a 404.
			// 404 should store a None in the cache, network error shouldn't.
			None
		}
	}
}

// Since there's no way to get a game by every provider ID from PCGamingWiki, we try with the game title.
pub async fn get_engine_from_game_title(title: &str) -> Option<GameEngine> {
	// Use only ascii and lowercase to make it easier to find by title.
	let lowercase_title = title.to_ascii_lowercase();

	// Remove "demo" suffix so that demos can match with the main game.
	let non_demo_title = regex_replace!(r" demo$", &lowercase_title, "");

	// Replace anything that isn't alphanumeric with a % character, the wildcard for the LIKE query.
	// This way we can still match even if the game has slight differences in the presented title punctuation.
	// Problem is, this can easily cause false flags (and it does).
	// In this case we use %25, which is % encoded for url components.
	let clean_title = regex_replace_all!(r"[^a-zA-Z0-9]+", &non_demo_title, "%25");

	// Finally do the query by page title.
	get_engine(&format!(
		"Infobox_game._pageName%20LIKE%20%22{}%22",
		clean_title
	))
	.await
}
