use std::{sync::Arc, thread, time::Duration};

use rai_pal_backend::{
	app_settings::{AppLocale, AppSettings, ColorScheme, TabId},
	game::Game,
	game_providers::game_provider::GameProviderId,
	games_query::GamesSortBy,
	local_database::{game_database::GameDatabase, mod_database::ModDatabase},
	result::LogErrExt,
};
use slint::{ComponentHandle, Model as _};

use crate::{
	ActionKind, AppColorScheme, AppTab, action_state, i18n, model::apply_game_to_row, selected,
};

use super::CommandHandler;

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

const fn core_tab(tab: AppTab) -> TabId {
	match tab {
		AppTab::Games => TabId::Games,
		AppTab::Mods => TabId::Mods,
		AppTab::Thanks => TabId::Thanks,
	}
}

impl CommandHandler {
	pub fn reset_settings(&self, detected_locale: Option<AppLocale>) {
		let Some(window) = self.ui() else {
			return;
		};

		let window_state = self.settings.borrow().window_state;
		*self.settings.borrow_mut() = AppSettings {
			window_state,
			..AppSettings::default()
		};
		self.persist_settings();

		let locale = i18n::effective_locale(&self.settings.borrow(), detected_locale);
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

		self.refresh_filter_groups();
		self.start_query(Duration::ZERO);
	}

	pub fn set_language(&self, value: &str, detected_locale: Option<AppLocale>) {
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

		let locale = i18n::effective_locale(&self.settings.borrow(), detected_locale);
		i18n::apply_translation(locale);

		if let Some(window) = self.ui() {
			window
				.global::<crate::AppSettings>()
				.set_selected_language(value.into());
		}
	}

	pub fn set_color_scheme(&self, value: AppColorScheme) {
		self.settings.borrow_mut().color_scheme = match value {
			AppColorScheme::Dark => ColorScheme::Dark,
			AppColorScheme::Light => ColorScheme::Light,
			AppColorScheme::Auto => ColorScheme::Auto,
		};
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
		self.settings.borrow_mut().selected_tab = core_tab(tab);
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

		let Some(provider_id) = row
			.provider_id
			.parse::<GameProviderId>()
			.ok_or_log("Failed to parse game provider ID")
		else {
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
	}

	pub fn deselect_game(&self) {
		self.clear_selected_game();

		if let Some(window) = self.ui() {
			selected::hide(&window);
		}
	}

	pub fn update_selected_row(&self, index: usize, game: &Game) {
		let Some(window) = self.ui() else {
			return;
		};

		let rows = window.global::<crate::GamesList>().get_rows();
		let mut row = rows.row_data(index).unwrap_or_default();
		apply_game_to_row(&mut row, game);
		rows.set_row_data(index, row.clone());
		selected::show(&window, game, row);
	}

	pub fn refresh_game(&self) {
		let Some((index, game)) = self.selected_game() else {
			return;
		};

		let action_id = action_state::key(ActionKind::RefreshGame, "");
		let window = self.window();
		action_state::start(&window, action_id.clone());

		let database = Arc::clone(&self.database);

		thread::spawn(move || {
			let mut current_game = game;

			let result = (|| {
				current_game.refresh_executable()?;
				database.insert_game(&current_game);
				database.refresh_installed_mods()?;
				database.get_game(&current_game.provider_id, &current_game.game_id)
			})();

			let updated_game = match result {
				Ok(updated_game) => updated_game,
				Err(err) => {
					log::error!("Failed to refresh game: {err}");
					action_state::finish(
						&window,
						action_id.clone(),
						Err(err),
						"Failed to refresh game",
					);
					return;
				}
			};

			let finish_window = window.clone();
			crate::runtime::invoke_on_ui(move || {
				let Some(handler) = Self::current() else {
					return;
				};

				handler.set_selected_game(index, updated_game.clone());
				handler.update_selected_row(index, &updated_game);
				handler.refresh_mods(&updated_game);
			});

			action_state::finish(
				&finish_window,
				action_id,
				Ok::<(), rai_pal_backend::result::Error>(()),
				"",
			);
		});
	}
}
