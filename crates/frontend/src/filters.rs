use std::{
	collections::{BTreeMap, HashSet},
	rc::Rc,
};

use rai_pal_backend::{
	filter_views::{FilterGroupView, FilterValueView, build_filter_views, build_mods_filter_views},
	game_providers::game_provider::GameProviderId,
	games_query::{FilterGroupKey, GamesFilter},
	local_database::{app_database::Db, game_database::GameDatabase, mod_database::ModDatabase},
	mods_query::ModsFilter,
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
	mod_tags: &BTreeMap<String, String>,
	collapsed_filters: &HashSet<FilterGroupKey>,
	providers_with_games: &HashSet<GameProviderId>,
) -> Vec<FilterGroupData> {
	build_filter_views(filter, mod_tags, collapsed_filters, providers_with_games)
		.into_iter()
		.map(group_data)
		.collect()
}

pub fn build_mods_groups(
	filter: &ModsFilter,
	mod_tags: &BTreeMap<String, String>,
	collapsed_filters: &HashSet<FilterGroupKey>,
	has_untagged_mods: bool,
) -> Vec<FilterGroupData> {
	build_mods_filter_views(filter, mod_tags, collapsed_filters, has_untagged_mods)
		.into_iter()
		.map(group_data)
		.collect()
}

pub fn load_providers_with_games(database: &Db) -> HashSet<GameProviderId> {
	database.get_providers_with_games().unwrap_or_else(|err| {
		log::warn!("Failed to load providers with games: {err}");
		HashSet::new()
	})
}

pub fn load_mod_tags(database: &Db) -> BTreeMap<String, String> {
	load_tags(database, true)
}

pub fn load_all_mod_tags(database: &Db) -> BTreeMap<String, String> {
	load_tags(database, false)
}

fn load_tags(database: &Db, filterable_only: bool) -> BTreeMap<String, String> {
	let Some(mods) = database
		.get_catalog_mod_map()
		.ok_or_log("Failed to load mod map")
	else {
		return BTreeMap::new();
	};
	let tag_definitions = database
		.get_mod_tags()
		.ok_or_log("Failed to load mod tags")
		.unwrap_or_default();

	mods.values()
		.filter_map(|game_mod| game_mod.tags.as_ref())
		.flatten()
		.filter_map(|tag_id| {
			let tag = tag_definitions.get(tag_id)?;
			if filterable_only && tag.filterable != Some(true) {
				return None;
			}
			Some((tag_id.clone(), tag.title.clone()))
		})
		.collect()
}
