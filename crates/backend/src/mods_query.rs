use rai_pal_proc_macros::serde_enum;

use crate::{games_query::FilterGroup, mods::manifest::Mod};

#[serde_enum]
pub enum ModType {
	Installable,
	LinkOnly,
}

impl ModType {
	pub const fn of(game_mod: &Mod) -> Self {
		if game_mod.install.is_some() {
			Self::Installable
		} else {
			Self::LinkOnly
		}
	}
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct ModsFilter {
	#[serde(alias = "modFamilies")]
	pub mod_tags: FilterGroup<String>,
	#[serde(default)]
	pub mod_types: FilterGroup<ModType>,
}

impl ModsFilter {
	pub fn matches(&self, game_mod: &Mod) -> bool {
		self.matches_type(game_mod) && self.matches_tags(game_mod)
	}

	fn matches_type(&self, game_mod: &Mod) -> bool {
		self.mod_types.item(Some(&ModType::of(game_mod))).enabled
	}

	fn matches_tags(&self, game_mod: &Mod) -> bool {
		match game_mod.tags.as_deref() {
			Some(tags) if !tags.is_empty() => {
				tags.iter().any(|tag| self.mod_tags.item(Some(tag)).enabled)
			}
			_ => self.mod_tags.item(None).enabled,
		}
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
