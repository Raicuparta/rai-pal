use lazy_regex::regex_replace_all;

use crate::{
	game_engines::game_engine::GameEngineBrand,
	serializable_struct,
};

serializable_struct!(PCGamingWikiTitle {
	#[serde(rename = "Engines")]
  // "Engines" is a comma-separated string
	engines: Option<String>,
});

serializable_struct!(PCGamingWikiQueryItem {
	title: PCGamingWikiTitle,
});

serializable_struct!(PCGamingWikiQueryResponse {
	cargoquery: Vec<PCGamingWikiQueryItem>,
});

pub async fn get_engine(where_query: &str) -> Option<GameEngineBrand> {
	let url = format!("https://www.pcgamingwiki.com/w/api.php?action=cargoquery&tables=Infobox_game&fields=Engines&where={where_query}&format=json");
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
						.find_map(|query_item| query_item.title.engines)
						.and_then(|first_engine_group| {
							first_engine_group
              .split_once(',')
              .or_else(|| Some((&first_engine_group, "")))
              .and_then(|(engine, _)|
                // I don't feel like figuring out the exact format,
                // since it can sometimes have the engine version included, sometimes not.
                // TODO take the engine version from here when available.
                if engine.contains("Unreal") {
                  Some(GameEngineBrand::Unreal)
                } else if engine.contains("Unity") {
                  Some(GameEngineBrand::Unity)
                } else if engine.contains("Godot") {
                  Some(GameEngineBrand::Godot)
                } else {
                  None
                }
              )
						})
				}
				Err(err) => None,
			}
		}
		Err(err) => None,
	}
}

pub async fn get_engine_from_game_title(title: &str) -> Option<GameEngineBrand> {
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
