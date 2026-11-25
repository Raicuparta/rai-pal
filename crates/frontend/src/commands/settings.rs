use std::{
	collections::{BTreeMap, HashSet},
	rc::Rc,
	time::Duration,
};

use rai_pal_backend::{
	app_settings::{AppLocale, AppSettings},
	game::Game,
	game_providers::game_provider::GameProviderId,
	games_query::GamesSortBy,
	local_database::game_database::{GameDatabase, GameId},
	result::LogErrExt,
};
use slint::{ComponentHandle, Model as _};

use crate::{
	AppColorScheme, AppTab, GameRowData, MainWindow, filters::build_groups, i18n, languages, model,
	model::apply_game_to_row,
};

use super::query::ScrollBehavior;
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

pub fn apply_settings_to_ui(
	window: &MainWindow,
	settings: &AppSettings,
	detected_locale: Option<AppLocale>,
	mod_tags: &BTreeMap<String, String>,
	providers_with_games: &HashSet<GameProviderId>,
) {
	i18n::apply_translation(i18n::effective_locale(settings, detected_locale));

	let app_settings = window.global::<crate::AppSettings>();
	app_settings.set_selected_color_scheme(settings.color_scheme.into());
	app_settings.set_hide_game_thumbnails(settings.hide_game_thumbnails);
	languages::refresh(window, settings, detected_locale);

	let groups = build_groups(
		&settings.games_query.filter,
		mod_tags,
		&settings.collapsed_filters,
		providers_with_games,
	);

	let query = window.global::<crate::GamesQuery>();
	query.set_search_text(settings.games_query.search.clone().into());
	query.set_sort_index(sort_option_index(
		settings.games_query.sort_by,
		settings.games_query.sort_descending,
	));
	query.set_filters_have_expanded(groups.iter().any(|group| group.expanded));
	query.set_filters_have_changes(groups.iter().any(|group| group.has_changes));
	query.set_filter_groups(Rc::new(slint::VecModel::from(groups)).into());

	window.set_current_tab(settings.selected_tab.into());
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
		self.scheduler.set_thumbnails_enabled(true);

		let providers_with_games = crate::filters::load_providers_with_games(&self.database);
		let mod_tags = crate::filters::load_mod_tags(&self.database);
		apply_settings_to_ui(
			&window,
			&self.settings.borrow(),
			self.detected_locale,
			&mod_tags,
			&providers_with_games,
		);
		self.start_query(Duration::ZERO, ScrollBehavior::Reset);
		self.load_mods_tab();
	}

	pub fn set_language(&self, value: &str) {
		let override_language = if value.is_empty() {
			None
		} else if let Some(locale) = AppLocale::from_code(value) {
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
			languages::refresh(&window, &self.settings.borrow(), self.detected_locale);
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
			self.start_query(Duration::ZERO, ScrollBehavior::Reset);
		}
	}

	pub fn set_tab(&self, tab: AppTab) {
		let previous_tab: AppTab = self.settings.borrow().selected_tab.into();
		self.settings.borrow_mut().selected_tab = tab.into();
		self.persist_settings();

		let Some(window) = self.ui() else {
			return;
		};

		if tab == AppTab::Games && previous_tab == AppTab::Games && window.get_game_selected() {
			self.deselect_game();
		}

		if tab == AppTab::Mods {
			self.load_mods_tab();
		} else if tab == AppTab::Thanks
			&& window
				.global::<crate::ThanksState>()
				.get_patrons()
				.row_count()
				== 0
		{
			self.load_patrons();
		}
	}

	pub fn switch_tab(&self, tab: AppTab) {
		if let Some(window) = self.ui() {
			window.set_current_tab(tab);
		}

		self.set_tab(tab);
	}

	pub fn set_search_text(&self, text: &str) {
		self.settings.borrow_mut().games_query.search = text.to_string();
		self.start_query(Duration::from_millis(200), ScrollBehavior::Reset);
	}

	pub fn set_sort_index(&self, index: i32) {
		let (sort_by, sort_descending) = sort_option_from_index(index);
		{
			let mut settings = self.settings.borrow_mut();
			settings.games_query.sort_by = sort_by;
			settings.games_query.sort_descending = sort_descending;
		}
		self.persist_settings();
		self.start_query(Duration::ZERO, ScrollBehavior::Reset);
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

		let Some(provider_id) = Option::<GameProviderId>::from(row.provider) else {
			return;
		};

		let Some(game) = self
			.database
			.get_game(&provider_id, row.game_id.as_str())
			.ok_or_log("Failed to load game")
		else {
			return;
		};

		self.show_selected_game(game, row);
	}

	pub fn select_game_by_id(&self, provider_id: GameProviderId, game_id: &str) {
		let Some(game) = self
			.database
			.get_game(&provider_id, game_id)
			.ok_or_log("Failed to load game")
		else {
			return;
		};

		let Some(window) = self.ui() else {
			return;
		};

		let rows = window.global::<crate::GamesList>().get_rows();
		let provider = provider_id.into();
		let row = rows
			.iter()
			.position(|row| row.provider == provider && row.game_id == game_id)
			.and_then(|index| rows.row_data(index))
			.unwrap_or_else(|| {
				model::make_row(&GameId {
					provider_id,
					game_id: game_id.to_string(),
				})
			});

		self.show_selected_game(game, row);
	}

	fn show_selected_game(&self, game: Game, row: GameRowData) {
		self.set_selected_game(game.clone());
		self.show_game_details(&game, row);
		self.refresh_mods(&game);

		if game.exe_path.is_some() {
			self.refresh_game_with_state(false);
		}

		self.settings.borrow_mut().selected_game = Some(GameId {
			provider_id: game.provider_id,
			game_id: game.game_id,
		});
		self.persist_settings();
	}

	pub fn deselect_game(&self) {
		self.hide_game_details();
		self.settings.borrow_mut().selected_game = None;
		self.persist_settings();
	}

	pub fn refresh_selected_row(&self, game: &Game) {
		let Some(window) = self.ui() else {
			return;
		};

		let rows = window.global::<crate::GamesList>().get_rows();
		let provider = game.provider_id.into();
		let Some(index) = rows
			.iter()
			.position(|row| row.provider == provider && row.game_id == game.game_id)
		else {
			return;
		};

		let mut row = rows.row_data(index).unwrap_or_default();
		apply_game_to_row(&mut row, game);
		rows.set_row_data(index, row.clone());
		self.show_game_details(game, row);
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
