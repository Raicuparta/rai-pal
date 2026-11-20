use crate::{games_query::FilterGroup, mods::manifest::Mod};

#[derive(serde::Serialize, serde::Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct ModsFilter {
	pub mod_families: FilterGroup<String>,
}

impl ModsFilter {
	pub fn matches(&self, game_mod: &Mod) -> bool {
		self.mod_families.item(game_mod.family.as_ref()).enabled
	}
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct ModsQuery {
	pub filter: ModsFilter,
	#[serde(skip)]
	pub search: String,
}

impl ModsQuery {
	pub fn matches(&self, game_mod: &Mod) -> bool {
		self.filter.matches(game_mod) && self.matches_search(game_mod)
	}

	fn matches_search(&self, game_mod: &Mod) -> bool {
		let search = self.search.trim().to_lowercase();

		search.is_empty()
			|| game_mod.title.to_lowercase().contains(&search)
			|| game_mod.author.to_lowercase().contains(&search)
			|| game_mod.description.to_lowercase().contains(&search)
	}
}
