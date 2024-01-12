use std::collections::HashMap;

use lazy_regex::{
	regex_captures,
	regex_replace_all,
};

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

	println!("### fetching the one and only {url}");
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
				Err(err) => None,
			}
		}
		Err(err) => None,
	}
}

pub async fn get_engine_from_gog_id(gog_id: &str) -> Option<GameEngine> {
	get_engine(&format!("GOGcom_ID%20HOLDS%20%22{gog_id}%22")).await
}

pub async fn get_engine_from_game_title(title: &str) -> Option<GameEngine> {
	// Since there's no way to get a game by every provider ID from PCGamingWiki, we try with the game title.
	// This is obviously not ideal, since titles can be formatted differently.
	// So I'm trying to clean them up a bit, and then to a LIKE query for finding the game.
	// For instance, a Batman: Arkham Knight would be turned into batman%arkham%knight.
	// The % character means "any characters" when making the query, so this would match Batman Arkham Knight or Batman™ Arkham Knight.
	// Has a bunch of wrong results, but better than nothing I guess?
	let lowercase_title = title.to_ascii_lowercase();
	let clean_title = regex_replace_all!(r"[^a-zA-Z0-9]+", &lowercase_title, "%25");
	get_engine(&format!(
		"Infobox_game._pageName%20LIKE%20%22{}%22",
		clean_title
	))
	.await
}
