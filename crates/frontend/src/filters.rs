use std::{collections::HashSet, rc::Rc};

use rai_pal_backend::{
	filter_views::{FilterGroupView, FilterValueView, build_filter_views},
	games_query::{FilterGroupKey, GamesFilter},
	local_database::{app_database::DbMutex, mod_database::ModDatabase},
	result::LogErrExt,
};

use crate::{FilterGroupData, FilterValueData, FilterValueLabel};

fn value_data(value: FilterValueView) -> FilterValueData {
	FilterValueData {
		value: value.value.into(),
		label: value.label.map_or(FilterValueLabel::None, Into::into),
		display: value.display.into(),
		note: value.note.into(),
		enabled: value.enabled,
		locked: value.locked,
	}
}

fn group_data(group: FilterGroupView) -> FilterGroupData {
	FilterGroupData {
		key: group.key.into(),
		label: group.label.into(),
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
	expanded_filters: &HashSet<FilterGroupKey>,
) -> Vec<FilterGroupData> {
	build_filter_views(filter, mod_families, expanded_filters)
		.into_iter()
		.map(group_data)
		.collect()
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
