use slint::ComponentHandle as _;
use std::rc::Rc;

use rai_pal_backend::{
	local_database::app_database::Db,
	mods::{manifest::Mod, mod_groups},
};

use crate::{MainWindow, MessageContext, ModsTabItemData, ModsTabModData, notifications};

use super::{
	CommandHandler,
	mods::{download_version, engine_text, release_date_display},
};

impl CommandHandler {
	pub fn load_mods_tab(&self) {
		let Some(window) = self.ui() else {
			return;
		};

		load(&window, &self.database);
	}
}

fn load(window: &MainWindow, database: &Db) {
	let groups = match mod_groups::get_mod_groups(database) {
		Ok(groups) => groups,
		Err(err) => {
			notifications::error(window, notifications::data(MessageContext::LoadMods, &err));
			let state = window.global::<crate::ModsState>();
			state.set_items(Rc::new(slint::VecModel::from(Vec::<ModsTabItemData>::new())).into());
			state.set_total_count(0);
			return;
		}
	};

	let total_count = groups.iter().map(|group| group.mods.len()).sum::<usize>();

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
					mods: slint::ModelRc::default(),
					single_mod: mods
						.into_iter()
						.next()
						.map_or_else(ModsTabModData::default, mod_data),
				}
			}
		})
		.collect::<Vec<_>>();

	let state = window.global::<crate::ModsState>();
	state.set_items(Rc::new(slint::VecModel::from(items)).into());
	state.set_total_count(i32::try_from(total_count).unwrap_or_else(|err| {
		log::warn!("Mods count exceeds i32: {err}");
		i32::MAX
	}));
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
