use std::{rc::Rc, time::Duration};

use rai_pal_backend::{
	filter_views::{
		apply_filter_action, collapse_all_filters, expand_all_filters, reset_all_filters,
		reset_filter_group,
	},
	games_query::{FilterAction, FilterGroupKey},
};
use slint::ComponentHandle as _;

use crate::{
	FilterGroupId,
	filters::{build_groups, load_providers_with_games},
};

use super::query::ScrollBehavior;
use super::{CommandHandler, on_command};

impl CommandHandler {
	pub fn refresh_filters(&self) {
		let providers_with_games = load_providers_with_games(&self.database);
		let groups = {
			let settings = self.settings.borrow();
			build_groups(
				&settings.games_query.filter,
				&self.mod_families,
				&settings.collapsed_filters,
				&providers_with_games,
			)
		};

		let Some(window) = self.ui() else {
			return;
		};

		let query = window.global::<crate::GamesQuery>();
		query.set_filters_have_changes(groups.iter().any(|group| group.has_changes));
		query.set_filters_have_expanded(groups.iter().any(|group| group.expanded));
		query.set_filter_groups(Rc::new(slint::VecModel::from(groups)).into());
	}

	pub fn apply_filter(&self, group: FilterGroupId, value: &str, action: FilterAction) {
		let changed = {
			let mut settings = self.settings.borrow_mut();
			apply_filter_action(
				&mut settings.games_query.filter,
				&self.mod_families,
				group.into(),
				value,
				action,
			)
		};

		if !changed {
			return;
		}

		self.persist_settings();
		self.refresh_filters();
		self.start_query(Duration::ZERO, ScrollBehavior::Reset);
	}

	pub fn reset_filter(&self, group: FilterGroupId) {
		{
			let mut settings = self.settings.borrow_mut();
			reset_filter_group(&mut settings.games_query.filter, group.into());
		}

		self.persist_settings();
		self.refresh_filters();
		self.start_query(Duration::ZERO, ScrollBehavior::Reset);
	}

	pub fn reset_filters(&self) {
		reset_all_filters(&mut self.settings.borrow_mut().games_query.filter);

		self.persist_settings();
		self.refresh_filters();
		self.start_query(Duration::ZERO, ScrollBehavior::Reset);
	}

	pub fn collapse_filters(&self) {
		collapse_all_filters(&mut self.settings.borrow_mut().collapsed_filters);
		self.persist_settings();
		self.refresh_filters();
	}

	pub fn expand_filters(&self) {
		expand_all_filters(&mut self.settings.borrow_mut().collapsed_filters);
		self.persist_settings();
		self.refresh_filters();
	}

	pub fn set_filter_expanded(&self, group: FilterGroupId, expanded: bool) {
		let group = FilterGroupKey::from(group);
		{
			let mut settings = self.settings.borrow_mut();
			if expanded {
				settings.collapsed_filters.remove(&group);
			} else {
				settings.collapsed_filters.insert(group);
			}
		}

		self.persist_settings();
		self.refresh_filters();
	}
}

pub fn register(window: &crate::MainWindow) {
	on_command!(window, GamesActions, on_filter_toggled, |group, value, checked| handler => {
		handler.apply_filter(group, value.as_str(), FilterAction::Toggle(checked));
	});
	on_command!(window, GamesActions, on_filter_lock_toggled, |group, value| handler => {
		handler.apply_filter(group, value.as_str(), FilterAction::Lock);
	});
	on_command!(window, GamesActions, on_filter_only, |group, value| handler => {
		handler.apply_filter(group, value.as_str(), FilterAction::Only);
	});
	on_command!(window, GamesActions, on_filter_reset, |group| handler => {
		handler.reset_filter(group);
	});
	on_command!(window, GamesActions, on_filters_reset, || handler => {
		handler.reset_filters();
	});
	on_command!(window, GamesActions, on_filters_collapse_all, || handler => {
		handler.collapse_filters();
	});
	on_command!(window, GamesActions, on_filters_expand_all, || handler => {
		handler.expand_filters();
	});
	on_command!(window, GamesActions, on_filter_expanded_changed, |group, expanded| handler => {
		handler.set_filter_expanded(group, expanded);
	});
}
