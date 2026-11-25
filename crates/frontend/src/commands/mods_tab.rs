use slint::ComponentHandle as _;
use std::{
	collections::BTreeMap,
	rc::Rc,
	sync::atomic::{AtomicU64, Ordering},
	time::Duration,
};

use rai_pal_backend::{
	filter_views::{
		apply_mods_filter_action, collapse_mods_filters, expand_all_filters,
		reset_all_mods_filters, reset_mods_filter_group, sync_mods_filter_availability,
	},
	games_query::{FilterAction, FilterGroupKey},
	local_database::mod_database::{ModDatabase, ModGame},
	mods::manifest::Mod,
};

use crate::{
	ActionKind, FilterGroupId, MessageContext, ModActionKind, ModsTabModData,
	filters::{build_mods_groups, load_all_mod_tags},
	mod_actions::{ActionGroup, ModActionList},
	model, notifications,
};

use super::{
	CommandHandler,
	mods::{download_version, engine_text, release_date_display},
	on_command,
};

static MODS_SEARCH_GENERATION: AtomicU64 = AtomicU64::new(0);

impl CommandHandler {
	pub fn load_mods_tab(&self) {
		let Some(window) = self.ui() else {
			return;
		};

		let mods = match self.database.get_catalog_mod_map() {
			Ok(mods) => mods,
			Err(err) => {
				notifications::error(&window, notifications::data(MessageContext::LoadMods, &err));
				let state = window.global::<crate::ModsState>();
				self.models.mods_tab.clear();
				state.set_total_count(0);
				return;
			}
		};

		let mod_tags = load_all_mod_tags(&self.database);
		let has_untagged_mods = mods
			.values()
			.any(|game_mod| game_mod.tags.as_ref().is_none_or(std::vec::Vec::is_empty));
		let (query, groups, availability_changed) = {
			let mut settings = self.settings.borrow_mut();
			let availability_changed =
				sync_mods_filter_availability(&mut settings.mods_query.filter, &mod_tags);
			let groups = build_mods_groups(
				&settings.mods_query.filter,
				&mod_tags,
				&settings.mods_collapsed_filters,
				has_untagged_mods,
			);
			(settings.mods_query.clone(), groups, availability_changed)
		};

		if availability_changed {
			self.persist_settings();
		}

		let query_ui = window.global::<crate::ModsQuery>();
		query_ui.set_search_text(query.search.clone().into());
		query_ui.set_filters_have_changes(groups.iter().any(|group| group.has_changes));
		query_ui.set_filters_have_expanded(groups.iter().any(|group| group.expanded));
		query_ui.set_filter_groups(Rc::new(slint::VecModel::from(groups)).into());

		let total_count = mods.len();
		let mod_games = match self.database.get_mod_games() {
			Ok(mod_games) => mod_games,
			Err(err) => {
				log::error!("Failed to load games for mods: {err}");
				BTreeMap::new()
			}
		};
		let mut mods = mods
			.into_values()
			.filter(|game_mod| query.matches(game_mod))
			.collect::<Vec<_>>();
		mods.sort_by(|a, b| {
			b.release_date()
				.unwrap_or_default()
				.cmp(a.release_date().unwrap_or_default())
		});

		let items = mods
			.into_iter()
			.map(|game_mod| {
				let games = mod_games.get(&game_mod.id).cloned().unwrap_or_default();
				mod_data(game_mod, &games)
			})
			.collect::<Vec<_>>();

		let state = window.global::<crate::ModsState>();
		model::sync_vec_model(&self.models.mods_tab, items);
		state.set_total_count(i32::try_from(total_count).unwrap_or_else(|err| {
			log::warn!("Mods count exceeds i32: {err}");
			i32::MAX
		}));
	}

	pub fn set_mods_search_text(&self, text: &str) {
		self.settings.borrow_mut().mods_query.search = text.to_string();

		let generation = MODS_SEARCH_GENERATION.fetch_add(1, Ordering::Relaxed) + 1;

		slint::Timer::single_shot(Duration::from_millis(200), move || {
			if MODS_SEARCH_GENERATION.load(Ordering::Relaxed) != generation {
				return;
			}

			let Some(handler) = Self::current() else {
				return;
			};

			handler.load_mods_tab();
		});
	}

	pub fn apply_mods_filter(&self, group: FilterGroupId, value: &str, action: FilterAction) {
		let mod_tags = load_all_mod_tags(&self.database);
		let changed = {
			let mut settings = self.settings.borrow_mut();
			apply_mods_filter_action(
				&mut settings.mods_query.filter,
				&mod_tags,
				group.into(),
				value,
				action,
			)
		};

		if !changed {
			return;
		}

		self.persist_settings();
		self.load_mods_tab();
	}

	pub fn reset_mods_filter(&self, group: FilterGroupId) {
		{
			let mut settings = self.settings.borrow_mut();
			reset_mods_filter_group(&mut settings.mods_query.filter, group.into());
		}

		self.persist_settings();
		self.load_mods_tab();
	}

	pub fn reset_mods_filters(&self) {
		reset_all_mods_filters(&mut self.settings.borrow_mut().mods_query.filter);

		self.persist_settings();
		self.load_mods_tab();
	}

	pub fn collapse_mods_filters(&self) {
		collapse_mods_filters(&mut self.settings.borrow_mut().mods_collapsed_filters);
		self.persist_settings();
		self.load_mods_tab();
	}

	pub fn expand_mods_filters(&self) {
		expand_all_filters(&mut self.settings.borrow_mut().mods_collapsed_filters);
		self.persist_settings();
		self.load_mods_tab();
	}

	pub fn set_mods_filter_expanded(&self, group: FilterGroupId, expanded: bool) {
		let group = FilterGroupKey::from(group);
		{
			let mut settings = self.settings.borrow_mut();
			if expanded {
				settings.mods_collapsed_filters.remove(&group);
			} else {
				settings.mods_collapsed_filters.insert(group);
			}
		}

		self.persist_settings();
		self.load_mods_tab();
	}
}

fn mod_data(game_mod: Mod, games: &[ModGame]) -> ModsTabModData {
	let version_text = download_version(&game_mod);
	let release_date = release_date_display(&game_mod);
	let engine_text = engine_text(&game_mod);
	let can_run = game_mod.run_standalone.is_some();
	let can_install = can_run && game_mod.install.is_some();
	let mut actions = ModActionList::new(&game_mod.id);

	if can_install {
		actions.push(
			ActionGroup::Primary,
			actions.action(ModActionKind::Download, ActionKind::StandaloneInstall, ""),
		);
	}

	if can_run {
		actions.push(
			ActionGroup::Primary,
			actions.action(ModActionKind::RunStandalone, ActionKind::StandaloneRun, ""),
		);
	}

	if let Some(website) = &game_mod.website {
		actions.push(
			ActionGroup::Link,
			actions.action(ModActionKind::Website, ActionKind::None, website),
		);
	}

	for game in games {
		actions.push(ActionGroup::Games, actions.game(game));
	}

	actions.push(
		ActionGroup::Debug,
		actions.action(ModActionKind::DebugData, ActionKind::None, ""),
	);

	ModsTabModData {
		title: game_mod.title.into(),
		description: game_mod.description.into(),
		version_text: version_text.into(),
		release_date: release_date.into(),
		author: game_mod.author.into(),
		engine_text: engine_text.into(),
		deprecated: game_mod.deprecated == Some(true),
		actions: Rc::new(slint::VecModel::from(actions.finish())).into(),
		special_actions: Default::default(),
	}
}

pub fn register(window: &crate::MainWindow) {
	on_command!(window, ModsTabActions, on_search_changed, |text| handler => {
		handler.set_mods_search_text(text.as_str());
	});
	on_command!(window, ModsTabActions, on_filter_toggled, |group, value, checked| handler => {
		handler.apply_mods_filter(group, value.as_str(), FilterAction::Toggle(checked));
	});
	on_command!(window, ModsTabActions, on_filter_lock_toggled, |group, value| handler => {
		handler.apply_mods_filter(group, value.as_str(), FilterAction::Lock);
	});
	on_command!(window, ModsTabActions, on_filter_only, |group, value| handler => {
		handler.apply_mods_filter(group, value.as_str(), FilterAction::Only);
	});
	on_command!(window, ModsTabActions, on_filter_reset, |group| handler => {
		handler.reset_mods_filter(group);
	});
	on_command!(window, ModsTabActions, on_filters_reset, || handler => {
		handler.reset_mods_filters();
	});
	on_command!(window, ModsTabActions, on_filters_collapse_all, || handler => {
		handler.collapse_mods_filters();
	});
	on_command!(window, ModsTabActions, on_filters_expand_all, || handler => {
		handler.expand_mods_filters();
	});
	on_command!(window, ModsTabActions, on_filter_expanded_changed, |group, expanded| handler => {
		handler.set_mods_filter_expanded(group, expanded);
	});
}
