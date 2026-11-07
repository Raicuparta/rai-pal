use std::rc::Rc;

use rai_pal_core::{
	local_database::app_database::DbMutex,
	mods::{manifest::Mod, mod_groups},
};

use crate::{
	MainWindow, ModsTabItemView, ModsTabModView,
	mods::{download_version, engine_text, release_date_display},
	notifications,
};

pub fn load(window: &MainWindow, database: &DbMutex) {
	let groups = match mod_groups::get_mod_groups(database) {
		Ok(groups) => groups,
		Err(err) => {
			notifications::error(window, format!("Failed to load mods: {err}"));
			window.set_mods_tab_items(
				Rc::new(slint::VecModel::from(Vec::<ModsTabItemView>::new())).into(),
			);
			return;
		}
	};

	let items = groups
		.into_iter()
		.map(|group| {
			let family = group.family;
			let mods = group.mods;

			if let Some(family) = family {
				ModsTabItemView {
					is_family: true,
					family_title: family.title.into(),
					family_description: family.description.into(),
					mods: Rc::new(slint::VecModel::from(
						mods.into_iter().map(mod_data).collect::<Vec<_>>(),
					))
					.into(),
					single_mod: ModsTabModView::default(),
				}
			} else {
				ModsTabItemView {
					is_family: false,
					family_title: "".into(),
					family_description: "".into(),
					mods: Default::default(),
					single_mod: mods
						.into_iter()
						.next()
						.map_or_else(ModsTabModView::default, mod_data),
				}
			}
		})
		.collect::<Vec<_>>();

	window.set_mods_tab_items(Rc::new(slint::VecModel::from(items)).into());
}

fn mod_data(game_mod: Mod) -> ModsTabModView {
	let version_text = download_version(&game_mod);
	let release_date = release_date_display(&game_mod);
	let engine_text = engine_text(&game_mod);
	let debug_json = serde_json::to_string_pretty(&game_mod).unwrap_or_default();

	ModsTabModView {
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
		debug_json: debug_json.into(),
	}
}
