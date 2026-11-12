use std::{collections::HashSet, rc::Rc};

use rai_pal_backend::{
	filter_views::{FilterGroupView, FilterValueView, build_filter_views},
	game_providers::game_provider::GameProviderId,
	games_query::{FilterGroupKey, GamesFilter},
	local_database::{
		app_database::DbMutex, game_database::GameDatabase, mod_database::ModDatabase,
	},
	result::LogErrExt,
};

use crate::{FilterGroupData, FilterValueData};

fn value_data(value: FilterValueView) -> FilterValueData {
	FilterValueData {
		value: value.value.into(),
		label: value.label.into(),
		display: value.display.into(),
		note: value.note.into(),
		enabled: value.enabled,
		locked: value.locked,
		available: value.available,
	}
}

fn group_data(group: FilterGroupView) -> FilterGroupData {
	FilterGroupData {
		key: group.key.into(),
		values: Rc::new(slint::VecModel::from(
			group.values.into_iter().map(value_data).collect::<Vec<_>>(),
		))
		.into(),
		has_changes: group.has_changes,
		has_locked: group.has_locked,
		expanded: group.expanded,
	}
}

pub fn build_groups(
	filter: &GamesFilter,
	mod_families: &[String],
	collapsed_filters: &HashSet<FilterGroupKey>,
	providers_with_games: &HashSet<GameProviderId>,
) -> Vec<FilterGroupData> {
	build_filter_views(
		filter,
		mod_families,
		collapsed_filters,
		providers_with_games,
	)
	.into_iter()
	.map(group_data)
	.collect()
}

pub fn load_providers_with_games(database: &DbMutex) -> HashSet<GameProviderId> {
	database.get_providers_with_games().unwrap_or_else(|err| {
		log::warn!("Failed to load providers with games: {err}");
		HashSet::new()
	})
}

pub fn load_mod_families(database: &DbMutex) -> Vec<String> {
	let Some(mods) = database.get_mod_map().ok_or_log("Failed to load mod map") else {
		return Vec::new();
	};

	let mut families: Vec<String> = mods
		.values()
		.filter_map(|game_mod| game_mod.family.clone())
		.collect();
	families.sort_by_key(|family| family.to_lowercase());
	families.dedup();
	families
}
