use slint::ComponentHandle as _;
use std::rc::Rc;

use rai_pal_core::{
	local_database::app_database::DbMutex,
	mods::{manifest::Mod, mod_groups},
};

use crate::{
	MainWindow, ModsTabItemData, ModsTabModData,
	mods::{download_version, engine_text, release_date_display},
	notifications,
};

pub fn load(window: &MainWindow, database: &DbMutex) {
	let groups = match mod_groups::get_mod_groups(database) {
		Ok(groups) => groups,
		Err(err) => {
			notifications::error(window, format!("Failed to load mods: {err}"));
			window
				.global::<crate::ModsState>()
				.set_items(Rc::new(slint::VecModel::from(Vec::<ModsTabItemData>::new())).into());
			return;
		}
	};

	let items = groups
		.into_iter()
		.map(|group| {
			let family = group.family;
			let mods = group.mods;

			if let Some(family) = family {
				ModsTabItemData {
					is_family: true,
					family_title: family.title.into(),
					family_description: family.description.into(),
					mods: Rc::new(slint::VecModel::from(
						mods.into_iter().map(mod_data).collect::<Vec<_>>(),
					))
					.into(),
					single_mod: ModsTabModData::default(),
				}
			} else {
				ModsTabItemData {
					is_family: false,
					family_title: "".into(),
					family_description: "".into(),
					mods: Default::default(),
					single_mod: mods
						.into_iter()
						.next()
						.map_or_else(ModsTabModData::default, mod_data),
				}
			}
		})
		.collect::<Vec<_>>();

	window
		.global::<crate::ModsState>()
		.set_items(Rc::new(slint::VecModel::from(items)).into());
}

fn mod_data(game_mod: Mod) -> ModsTabModData {
	let version_text = download_version(&game_mod);
	let release_date = release_date_display(&game_mod);
	let engine_text = engine_text(&game_mod);

	ModsTabModData {
		id: game_mod.id.into(),
		title: game_mod.title.into(),
		description: game_mod.description.into(),
		version_text: version_text.into(),
		release_date: release_date.into(),
		author: game_mod.author.into(),
		engine_text: engine_text.into(),
		deprecated: game_mod.deprecated == Some(true),
		source_code: game_mod.source_code.into(),
		can_run: game_mod.run_standalone.is_some(),
		can_install: game_mod.run_standalone.is_some() && game_mod.install.is_some(),
	}
}
