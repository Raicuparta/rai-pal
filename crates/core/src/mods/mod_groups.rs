use std::collections::BTreeMap;

use crate::{
	local_database::mod_database::ModDatabase,
	mods::{manifest::Mod, mod_family::ModFamily},
	result::Result,
};

/// A mods-tab entry: either one standalone mod or a family with several mods.
#[derive(Clone, Debug)]
pub struct ModGroup {
	pub family: Option<ModFamily>,
	pub mods: Vec<Mod>,
}

struct Group {
	family: Option<ModFamily>,
	mods: Vec<Mod>,
	release_date: Option<String>,
}

pub fn get_mod_groups<D: ModDatabase>(database: &D) -> Result<Vec<ModGroup>> {
	let all_mods = database.get_mod_map()?;
	let families = database.get_mod_families().unwrap_or_default();

	let mut family_mods = BTreeMap::<String, Vec<Mod>>::new();
	let mut groups = Vec::<Group>::new();

	for game_mod in all_mods.values() {
		if let Some(family) = &game_mod.family {
			family_mods
				.entry(family.clone())
				.or_default()
				.push(game_mod.clone());
		} else {
			groups.push(mod_group(game_mod.clone()));
		}
	}

	for (family_id, mut mods) in family_mods {
		mods.sort_by(|a, b| {
			b.release_date()
				.unwrap_or_default()
				.cmp(a.release_date().unwrap_or_default())
		});

		let family = families.get(&family_id);
		if mods.len() > 1
			&& let Some(family) = family
		{
			groups.push(family_group(family, mods));
		} else {
			groups.extend(mods.into_iter().map(mod_group));
		}
	}

	groups.sort_by(|a, b| {
		b.family
			.is_some()
			.cmp(&a.family.is_some())
			.then_with(|| compare_group_dates(a, b))
	});

	Ok(groups
		.into_iter()
		.map(|group| ModGroup {
			family: group.family,
			mods: group.mods,
		})
		.collect())
}

fn mod_group(game_mod: Mod) -> Group {
	Group {
		release_date: game_mod.release_date().map(str::to_string),
		family: None,
		mods: vec![game_mod],
	}
}

fn family_group(family: &ModFamily, mods: Vec<Mod>) -> Group {
	Group {
		release_date: mods
			.first()
			.and_then(|game_mod| game_mod.release_date())
			.map(str::to_string),
		family: Some(family.clone()),
		mods,
	}
}

fn compare_group_dates(a: &Group, b: &Group) -> std::cmp::Ordering {
	b.release_date
		.as_deref()
		.unwrap_or_default()
		.cmp(a.release_date.as_deref().unwrap_or_default())
}
