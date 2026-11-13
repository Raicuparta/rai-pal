use std::{cell::RefCell, fmt::Display, rc::Rc, sync::Arc};

use rai_pal_backend::{
	app_settings::{AppLocale, AppSettings},
	game::Game,
	games_query::GamesQuery,
	local_database::app_database::DbMutex,
};

use crate::{
	ActionKey, MainWindow, MessageContext, action_state, downloads::Tracker,
	scheduler::FetchScheduler,
};

use self::{confirmation::PendingState, updater::Updater};

pub mod actions;
pub mod confirmation;
pub mod debug;
pub mod filters;
pub mod manual;
pub mod mod_sources;
pub mod mods;
pub mod mods_tab;
pub mod os_integration;
pub mod query;
pub mod refresh;
pub mod settings;
pub mod thanks;
pub mod tools;
pub mod updater;
pub mod user;

macro_rules! on_command {
	($window:expr, $global:ident, $callback:ident, || $handler:ident => $body:block) => {{
		slint::ComponentHandle::global::<crate::$global>($window).$callback(move || {
			let Some($handler) = crate::commands::CommandHandler::current() else {
				return;
			};
			$body
		});
	}};
	($window:expr, $global:ident, $callback:ident, |$($arg:ident),*| $handler:ident => $body:block) => {{
		slint::ComponentHandle::global::<crate::$global>($window).$callback(move |$($arg),*| {
			let Some($handler) = crate::commands::CommandHandler::current() else {
				return;
			};
			$body
		});
	}};
}

pub(crate) use on_command;

pub fn register(window: &MainWindow) {
	actions::register(window);
	confirmation::register(window);
	debug::register(window);
	filters::register(window);
	manual::register(window);
	mod_sources::register(window);
	mods::register(window);
	query::register(window);
	refresh::register(window);
	settings::register(window);
	tools::register(window);
	updater::register(window);
	user::register(window);
	crate::downloads::register(window);
	crate::notifications::register(window);
}

thread_local! {
	static CURRENT: RefCell<Option<std::rc::Weak<CommandHandler>>> = const { RefCell::new(None) };
}

pub struct CommandHandler {
	pub database: Arc<DbMutex>,
	pub settings: Rc<RefCell<AppSettings>>,
	pub scheduler: Arc<FetchScheduler>,
	pub downloads: Tracker,

	window: slint::Weak<MainWindow>,
	selected_game: RefCell<Option<(usize, Game)>>,
	pending_confirmation: PendingState,
	mod_families: Vec<String>,
	detected_locale: Option<AppLocale>,
	updater: Updater,
}

impl CommandHandler {
	pub fn create(
		window: slint::Weak<MainWindow>,
		database: Arc<DbMutex>,
		settings: Rc<RefCell<AppSettings>>,
		scheduler: Arc<FetchScheduler>,
		mod_families: Vec<String>,
		detected_locale: Option<AppLocale>,
	) -> Rc<Self> {
		let updater = Updater::start(window.clone());
		let handler = Rc::new(Self {
			database,
			settings,
			scheduler,
			downloads: Tracker::new(window.clone()),
			window,
			selected_game: RefCell::default(),
			pending_confirmation: PendingState::default(),
			mod_families,
			detected_locale,
			updater,
		});

		CURRENT.with(|current| *current.borrow_mut() = Some(Rc::downgrade(&handler)));

		handler
	}

	pub fn current() -> Option<Rc<Self>> {
		CURRENT.with(|current| current.borrow().as_ref().and_then(std::rc::Weak::upgrade))
	}

	pub fn window(&self) -> slint::Weak<MainWindow> {
		self.window.clone()
	}

	pub fn ui(&self) -> Option<MainWindow> {
		self.window.upgrade()
	}

	pub fn selected_game(&self) -> Option<(usize, Game)> {
		self.selected_game.borrow().clone()
	}

	pub fn set_selected_game(&self, index: usize, game: Game) {
		*self.selected_game.borrow_mut() = Some((index, game));
	}

	pub fn clear_selected_game(&self) {
		*self.selected_game.borrow_mut() = None;
	}

	pub fn query(&self) -> GamesQuery {
		self.settings.borrow().games_query.clone()
	}

	pub fn spawn_action<E: Display>(
		&self,
		action_id: ActionKey,
		context: MessageContext,
		action: impl FnOnce() -> std::result::Result<(), E> + Send + 'static,
	) {
		let window = self.window();
		action_state::start(&window, action_id.clone());

		std::thread::spawn(move || {
			let result = action();

			if let Err(err) = &result {
				log::error!("{context:?} failed: {err}");
			}

			action_state::finish(&window, action_id, result, context);
		});
	}

	pub fn persist_settings(&self) {
		if let Err(err) = self.settings.borrow().try_write() {
			log::error!("Failed to save app settings: {err}");
		}
	}
}
