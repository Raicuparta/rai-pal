use std::{
	cell::RefCell,
	rc::Rc,
	sync::{Arc, Mutex},
};

use rai_pal_backend::{
	app_settings::AppSettings, game::Game, games_query::GamesQuery,
	local_database::app_database::DbMutex,
};

use crate::{
	MainWindow, downloads::Tracker, scheduler::FetchScheduler, selected::SelectedGameState,
};
use rai_pal_backend::result::LogErrExt;

use self::{confirmation::PendingState, updater::Updater};

pub mod actions;
pub mod confirmation;
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

thread_local! {
	static CURRENT: RefCell<Option<std::rc::Weak<CommandHandler>>> = const { RefCell::new(None) };
}

pub struct CommandHandler {
	pub database: Arc<DbMutex>,
	pub settings: Rc<RefCell<AppSettings>>,
	pub scheduler: Arc<FetchScheduler>,
	pub downloads: Tracker,

	window: slint::Weak<MainWindow>,
	selected_game: SelectedGameState,
	pending_confirmation: Rc<PendingState>,
	mod_families: Vec<String>,
	updater: Updater,
}

impl CommandHandler {
	pub fn create(
		window: slint::Weak<MainWindow>,
		database: Arc<DbMutex>,
		settings: Rc<RefCell<AppSettings>>,
		scheduler: Arc<FetchScheduler>,
		mod_families: Vec<String>,
	) -> Rc<Self> {
		let updater = Updater::start(window.clone());
		let handler = Rc::new(Self {
			database,
			settings,
			scheduler,
			downloads: Tracker::new(window.clone()),
			window,
			selected_game: Arc::new(Mutex::new(None)),
			pending_confirmation: Rc::new(PendingState::default()),
			mod_families,
			updater,
		});

		CURRENT.with(|current| *current.borrow_mut() = Some(Rc::downgrade(&handler)));

		handler
	}

	/// The handler for the UI thread. Use this at thread boundaries
	/// (`invoke_from_event_loop`, deep link callbacks) where the `Rc` cannot be
	/// captured directly.
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
		self.selected_game
			.lock()
			.ok_or_log("Failed to lock selected game")
			.and_then(|selected| selected.clone())
	}

	pub fn set_selected_game(&self, index: usize, game: Game) {
		if let Some(mut selected) = self
			.selected_game
			.lock()
			.ok_or_log("Failed to lock selected game")
		{
			*selected = Some((index, game));
		}
	}

	pub fn clear_selected_game(&self) {
		if let Some(mut selected) = self
			.selected_game
			.lock()
			.ok_or_log("Failed to lock selected game")
		{
			*selected = None;
		}
	}

	pub fn query(&self) -> GamesQuery {
		self.settings.borrow().games_query.clone()
	}

	pub fn persist_settings(&self) {
		if let Err(err) = self.settings.borrow().try_write() {
			log::error!("Failed to save app settings: {err}");
		}
	}
}
