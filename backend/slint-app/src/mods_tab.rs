use std::{collections::BTreeMap, rc::Rc};

use rai_pal_core::{
	local_database::{app_database::DbMutex, mod_database::ModDatabase},
	mods::{game_mod::GameMod, mod_family::ModFamily},
};

use crate::{MainWindow, ModsTabItemData, ModsTabModData, mods, notifications};

struct Group {
	item: ModsTabItemData,
	release_date: Option<String>,
	is_family: bool,
}

pub fn load(window: &MainWindow, database: &DbMutex) {
	let mods = match database.get_mod_map() {
		Ok(mods) => mods,
		Err(err) => {
			notifications::error(window, format!("Failed to load mods: {err}"));
			window.set_mods_tab_items(
				Rc::new(slint::VecModel::from(Vec::<ModsTabItemData>::new())).into(),
			);
			return;
		}
	};
	let families = database.get_mod_families().unwrap_or_default();

	let mut family_mods = BTreeMap::<String, Vec<&GameMod>>::new();
	let mut groups = Vec::<Group>::new();

	for game_mod in mods.values() {
		if let Some(family) = &game_mod.family {
			family_mods
				.entry(family.clone())
				.or_default()
				.push(game_mod);
		} else {
			groups.push(mod_group(game_mod));
		}
	}

	for (family_id, mut mods) in family_mods {
		mods.sort_by(|a, b| compare_release_dates(a, b));

		let family = families.get(&family_id);
		if mods.len() > 1
			&& let Some(family) = family
		{
			groups.push(family_group(family, &mods));
		} else {
			groups.extend(mods.into_iter().map(mod_group));
		}
	}

	groups.sort_by(|a, b| {
		b.is_family
			.cmp(&a.is_family)
			.then_with(|| compare_group_dates(a, b))
	});

	let items = groups
		.into_iter()
		.map(|group| group.item)
		.collect::<Vec<_>>();
	window.set_mods_tab_items(Rc::new(slint::VecModel::from(items)).into());
}

fn mod_group(game_mod: &GameMod) -> Group {
	Group {
		item: ModsTabItemData {
			is_family: false,
			family_title: "".into(),
			family_description: "".into(),
			mods: Default::default(),
			single_mod: mod_data(game_mod),
		},
		release_date: mods::mod_release_date(game_mod).map(str::to_string),
		is_family: false,
	}
}

fn family_group(family: &ModFamily, mods: &[&GameMod]) -> Group {
	let mod_items = mods
		.iter()
		.map(|game_mod| mod_data(game_mod))
		.collect::<Vec<_>>();

	Group {
		item: ModsTabItemData {
			is_family: true,
			family_title: family.title.clone().into(),
			family_description: family.description.clone().into(),
			mods: Rc::new(slint::VecModel::from(mod_items)).into(),
			single_mod: Default::default(),
		},
		release_date: mods
			.first()
			.and_then(|game_mod| mods::mod_release_date(game_mod))
			.map(str::to_string),
		is_family: true,
	}
}

fn mod_data(game_mod: &GameMod) -> ModsTabModData {
	ModsTabModData {
		id: game_mod.id.clone().into(),
		title: game_mod.title.clone().into(),
		description: game_mod.description.clone().into(),
		version_text: game_mod
			.download
			.as_ref()
			.map(|download| download.id.clone())
			.unwrap_or_default()
			.into(),
		release_date: mods::release_date(game_mod).into(),
		author: game_mod.author.clone().into(),
		engine_text: mods::engine_text(game_mod).into(),
		deprecated: game_mod.deprecated == Some(true),
		source_code: game_mod.source_code.clone().into(),
		can_run: game_mod.run_standalone.is_some(),
		can_install: game_mod.run_standalone.is_some() && game_mod.install.is_some(),
	}
}

fn compare_release_dates(a: &GameMod, b: &GameMod) -> std::cmp::Ordering {
	mods::mod_release_date(b)
		.unwrap_or_default()
		.cmp(mods::mod_release_date(a).unwrap_or_default())
}

fn compare_group_dates(a: &Group, b: &Group) -> std::cmp::Ordering {
	b.release_date
		.as_deref()
		.unwrap_or_default()
		.cmp(a.release_date.as_deref().unwrap_or_default())
}
