use std::{rc::Rc, time::Duration};

use rai_pal_backend::games_query::FilterAction;
use slint::ComponentHandle as _;

use crate::filters::{apply_filter_action, build_groups, reset_all_filters, reset_filter_group};

use super::CommandHandler;

impl CommandHandler {
	pub fn refresh_filter_groups(&self) {
		let groups = {
			let settings = self.settings.borrow();
			build_groups(
				&settings.games_query.filter,
				&self.mod_families,
				&settings.expanded_filters,
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

	pub fn handle_filter_action(&self, group: &str, value: &str, action: FilterAction) {
		let changed = {
			let mut settings = self.settings.borrow_mut();
			apply_filter_action(
				&mut settings.games_query.filter,
				&self.mod_families,
				group,
				value,
				action,
			)
		};

		if !changed {
			return;
		}

		self.persist_settings();
		self.refresh_filter_groups();
		self.start_query(Duration::ZERO);
	}

	pub fn reset_filter(&self, group: &str) {
		let changed = {
			let mut settings = self.settings.borrow_mut();
			reset_filter_group(&mut settings.games_query.filter, group)
		};

		if !changed {
			return;
		}

		self.persist_settings();
		self.refresh_filter_groups();
		self.start_query(Duration::ZERO);
	}

	pub fn reset_filters(&self) {
		reset_all_filters(&mut self.settings.borrow_mut().games_query.filter);

		self.persist_settings();
		self.refresh_filter_groups();
		self.start_query(Duration::ZERO);
	}

	pub fn collapse_filters(&self) {
		self.settings.borrow_mut().expanded_filters.clear();
		self.persist_settings();
		self.refresh_filter_groups();
	}

	pub fn set_filter_expanded(&self, group: &str, expanded: bool) {
		{
			let mut settings = self.settings.borrow_mut();
			if expanded {
				settings.expanded_filters.insert(group.to_string());
			} else {
				settings.expanded_filters.remove(group);
			}
		}

		self.persist_settings();
		self.refresh_filter_groups();
	}
}
