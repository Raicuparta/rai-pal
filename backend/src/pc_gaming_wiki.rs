use lazy_regex::regex_replace_all;

use crate::game_engines::game_engine::GameEngineBrand;

pub async fn get_engine(where_query: &str) -> Option<GameEngineBrand> {
	let url = format!("https://www.pcgamingwiki.com/w/api.php?action=cargoquery&tables=Infobox_game&fields=Engines,Infobox_game._pageName=Page&where={where_query}&format=json");
	let result = reqwest::get(url).await;

	match result {
		Ok(response) => {
			let body = response.text().await;

			match body {
				Ok(text) => {
					if text.contains("Unity") {
						Some(GameEngineBrand::Unity)
					} else if text.contains("Unreal") {
						Some(GameEngineBrand::Unreal)
					} else if text.contains("Godot") {
						Some(GameEngineBrand::Godot)
					} else {
						None
					}
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
	// This will never be perfect, but seems to work pretty ok for a good number of games.
	let lowercase_title = title.to_ascii_lowercase();
	let clean_title = regex_replace_all!(r"[^a-zA-Z0-9]+", &lowercase_title, "%25");
	get_engine(&format!(
		"Infobox_game._pageName%20LIKE%20%22{}%22",
		clean_title
	))
	.await
}
