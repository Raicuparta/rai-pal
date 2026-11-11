use std::time::Duration;

use rai_pal_backend::{
	app_settings::AppSettings, game::Game, games_query::GamesSortBy,
	local_database::game_database::GameDatabase, result::LogErrExt,
};
use slint::{ComponentHandle, Model as _};

use crate::{AppColorScheme, AppTab, i18n, model::apply_game_to_row, selected};

use super::{CommandHandler, on_command};

const SORT_OPTIONS: [(GamesSortBy, bool); 6] = [
	(GamesSortBy::Title, false),
	(GamesSortBy::Title, true),
	(GamesSortBy::Engine, false),
	(GamesSortBy::Engine, true),
	(GamesSortBy::ReleaseDate, false),
	(GamesSortBy::ReleaseDate, true),
];

pub fn sort_option_index(sort_by: GamesSortBy, sort_descending: bool) -> i32 {
	SORT_OPTIONS
		.iter()
		.position(|option| *option == (sort_by, sort_descending))
		.and_then(|index| i32::try_from(index).ok_or_log("Sort option index does not fit in i32"))
		.unwrap_or_default()
}

fn sort_option_from_index(index: i32) -> (GamesSortBy, bool) {
	usize::try_from(index)
		.ok_or_log("Sort option index is negative")
		.and_then(|index| SORT_OPTIONS.get(index).copied())
		.unwrap_or((GamesSortBy::Title, false))
}

impl CommandHandler {
	pub fn reset_settings(&self) {
		let Some(window) = self.ui() else {
			return;
		};

		let window_state = self.settings.borrow().window_state;
		*self.settings.borrow_mut() = AppSettings {
			window_state,
			..AppSettings::default()
		};
		self.persist_settings();

		let locale = i18n::effective_locale(&self.settings.borrow(), self.detected_locale);
		i18n::apply_translation(locale);

		window
			.global::<crate::AppSettings>()
			.set_selected_language("".into());
		window
			.global::<crate::AppSettings>()
			.set_selected_color_scheme(AppColorScheme::Auto);
		window
			.global::<crate::AppSettings>()
			.set_hide_game_thumbnails(false);
		window
			.global::<crate::GamesQuery>()
			.set_search_text("".into());
		window.global::<crate::GamesQuery>().set_sort_index(0);
		window.set_current_tab(AppTab::Games);
		self.scheduler.set_thumbnails_enabled(true);

		self.refresh_filters();
		self.start_query(Duration::ZERO);
	}

	pub fn set_language(&self, value: &str) {
		let override_language = if value.is_empty() {
			None
		} else if let Some(locale) = i18n::locale_from_code(value) {
			Some(locale)
		} else {
			log::error!("Invalid language selection {value}");
			return;
		};

		self.settings.borrow_mut().override_language = override_language;
		self.persist_settings();

		let locale = i18n::effective_locale(&self.settings.borrow(), self.detected_locale);
		i18n::apply_translation(locale);

		if let Some(window) = self.ui() {
			window
				.global::<crate::AppSettings>()
				.set_selected_language(value.into());
		}
	}

	pub fn set_color_scheme(&self, value: AppColorScheme) {
		self.settings.borrow_mut().color_scheme = value.into();
		self.persist_settings();
	}

	pub fn set_hide_game_thumbnails(&self, hide: bool) {
		self.settings.borrow_mut().hide_game_thumbnails = hide;
		self.persist_settings();
		self.scheduler.set_thumbnails_enabled(!hide);

		if !hide {
			self.start_query(Duration::ZERO);
		}
	}

	pub fn set_tab(&self, tab: AppTab) {
		self.settings.borrow_mut().selected_tab = tab.into();
		self.persist_settings();

		let Some(window) = self.ui() else {
			return;
		};

		if tab == AppTab::Mods {
			self.load_mods_tab();
		} else if tab == AppTab::Thanks
			&& window
				.global::<crate::ThanksState>()
				.get_patrons()
				.row_count() == 0
		{
			self.load_patrons();
		}
	}

	pub fn set_search_text(&self, text: &str) {
		self.settings.borrow_mut().games_query.search = text.to_string();
		self.persist_settings();
		self.start_query(Duration::from_millis(200));
	}

	pub fn set_sort_index(&self, index: i32) {
		let (sort_by, sort_descending) = sort_option_from_index(index);
		{
			let mut settings = self.settings.borrow_mut();
			settings.games_query.sort_by = sort_by;
			settings.games_query.sort_descending = sort_descending;
		}
		self.persist_settings();
		self.start_query(Duration::ZERO);
	}

	pub fn select_game(&self, index: i32) {
		let index = usize::try_from(index.max(0)).unwrap_or_default();

		let Some(window) = self.ui() else {
			return;
		};

		let rows = window.global::<crate::GamesList>().get_rows();
		let Some(row) = rows.row_data(index) else {
			return;
		};

		let Some(provider_id) = crate::convert::core_provider_id(row.provider) else {
			return;
		};

		let Some(game) = self
			.database
			.get_game(&provider_id, row.game_id.as_str())
			.ok_or_log("Failed to load game")
		else {
			return;
		};

		self.set_selected_game(index, game.clone());
		selected::show(&window, &game, row);
		self.refresh_mods(&game);

		if game.exe_path.is_some() {
			self.refresh_game();
		}
	}

	pub fn deselect_game(&self) {
		self.clear_selected_game();

		if let Some(window) = self.ui() {
			selected::hide(&window);
		}
	}

	pub fn refresh_selected_row(&self, index: usize, game: &Game) {
		let Some(window) = self.ui() else {
			return;
		};

		let rows = window.global::<crate::GamesList>().get_rows();
		let mut row = rows.row_data(index).unwrap_or_default();
		apply_game_to_row(&mut row, game);
		rows.set_row_data(index, row.clone());
		selected::show(&window, game, row);
	}
}

pub fn register(window: &crate::MainWindow) {
	on_command!(window, AppActions, on_reset_settings, || handler => {
		handler.reset_settings();
	});
	on_command!(window, AppActions, on_language_changed, |value| handler => {
		handler.set_language(value.as_str());
	});
	on_command!(window, AppActions, on_color_scheme_changed, |value| handler => {
		handler.set_color_scheme(value);
	});
	on_command!(window, AppActions, on_hide_game_thumbnails_changed, |hide| handler => {
		handler.set_hide_game_thumbnails(hide);
	});
	on_command!(window, AppActions, on_tab_changed, |tab| handler => {
		handler.set_tab(tab);
	});
	on_command!(window, GamesActions, on_search_changed, |text| handler => {
		handler.set_search_text(text.as_str());
	});
	on_command!(window, GamesActions, on_sort_changed, |index| handler => {
		handler.set_sort_index(index);
	});
	on_command!(window, GamesActions, on_select_game, |index| handler => {
		handler.select_game(index);
	});
	on_command!(window, GamesActions, on_back, || handler => {
		handler.deselect_game();
	});
}
