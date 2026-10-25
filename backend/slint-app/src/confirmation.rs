use std::{cell::RefCell, rc::Rc, sync::Arc};

use rai_pal_core::{
	app_settings::AppSettings, game::DbGame, games_query::GamesQuery,
	local_database::app_database::DbMutex,
};

use crate::{
	MainWindow, app::persist_settings, downloads::Tracker, mods, scheduler::FetchScheduler,
	selected,
};
use slint::ComponentHandle;

type ConfirmationFn = Box<dyn FnOnce(&MainWindow, Arc<DbMutex>)>;

struct PendingConfirmation {
	action: ConfirmationFn,
	skip_id: String,
}

#[derive(Default)]
pub struct PendingState(RefCell<Option<PendingConfirmation>>);

pub fn request_install(
	window: &MainWindow,
	settings: &Rc<RefCell<AppSettings>>,
	pending: &Rc<PendingState>,
	database: Arc<DbMutex>,
	game: DbGame,
	mod_id: String,
	downloads: Tracker,
	confirmation_text: String,
	confirm_label: String,
	skip_id: String,
) {
	request(
		window,
		settings,
		pending,
		database,
		Box::new(move |window, database| {
			mods::install_mod(database, window.as_weak(), game, mod_id, downloads);
		}),
		confirmation_text,
		confirm_label,
		skip_id,
	);
}

pub fn request_uninstall_all(
	window: &MainWindow,
	settings: &Rc<RefCell<AppSettings>>,
	pending: &Rc<PendingState>,
	database: Arc<DbMutex>,
	game: DbGame,
	confirmation_text: String,
	confirm_label: String,
) {
	request(
		window,
		settings,
		pending,
		database,
		Box::new(move |window, database| {
			mods::uninstall_all_mods(database, window.as_weak(), game);
		}),
		confirmation_text,
		confirm_label,
		String::new(),
	);
}

#[expect(clippy::too_many_arguments)]
pub fn request_remove_game(
	window: &MainWindow,
	settings: &Rc<RefCell<AppSettings>>,
	pending: &Rc<PendingState>,
	database: Arc<DbMutex>,
	scheduler: Arc<FetchScheduler>,
	query: GamesQuery,
	selected_game: selected::SelectedGameState,
	game: DbGame,
	confirmation_text: String,
	confirm_label: String,
) {
	let action_database = database.clone();

	request(
		window,
		settings,
		pending,
		database,
		Box::new(move |window, _| {
			crate::manual::remove_game(
				window,
				action_database,
				scheduler,
				query,
				selected_game,
				game,
			);
		}),
		confirmation_text,
		confirm_label,
		String::new(),
	);
}

fn request(
	window: &MainWindow,
	settings: &Rc<RefCell<AppSettings>>,
	pending: &Rc<PendingState>,
	database: Arc<DbMutex>,
	action: ConfirmationFn,
	confirmation_text: String,
	confirm_label: String,
	skip_id: String,
) {
	let dialog_id = if skip_id.is_empty() {
		String::new()
	} else {
		format!("skip-confirm-{skip_id}")
	};

	if !dialog_id.is_empty() && settings.borrow().skip_confirm_dialogs.contains(&dialog_id) {
		*pending.0.borrow_mut() = None;
		execute(window, database, action);
		return;
	}

	let can_skip = !skip_id.is_empty();

	*pending.0.borrow_mut() = Some(PendingConfirmation { action, skip_id });

	window.set_confirmation_text(confirmation_text.into());
	window.set_confirmation_confirm_label(confirm_label.into());
	window.set_confirmation_can_skip(can_skip);
	window.set_confirmation_dont_ask_again(false);
	window.set_confirmation_open(true);
}

pub fn respond(
	window: &MainWindow,
	settings: &Rc<RefCell<AppSettings>>,
	pending: &Rc<PendingState>,
	database: Arc<DbMutex>,
	confirmed: bool,
) {
	window.set_confirmation_open(false);

	let Some(pending) = pending.0.borrow_mut().take() else {
		return;
	};

	if !confirmed {
		return;
	}

	if !pending.skip_id.is_empty() && window.get_confirmation_dont_ask_again() {
		let dialog_id = format!("skip-confirm-{}", pending.skip_id);
		settings.borrow_mut().skip_confirm_dialogs.insert(dialog_id);
		persist_settings(settings);
	}

	execute(window, database, pending.action);
}

fn execute(window: &MainWindow, database: Arc<DbMutex>, action: ConfirmationFn) {
	action(window, database);
}
