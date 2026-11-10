use std::{cell::RefCell, rc::Rc, sync::Arc};

use rai_pal_backend::{
	app_settings::AppSettings, game::Game, games_query::GamesQuery,
	local_database::app_database::DbMutex,
};

use crate::{
	MainWindow, analytics, app::persist_settings, downloads::Tracker, mods,
	scheduler::FetchScheduler, selected,
};
use slint::ComponentHandle;

type ConfirmationFn = Box<dyn FnOnce(&MainWindow, Arc<DbMutex>)>;

struct PendingConfirmation {
	action: ConfirmationFn,
	skip_id: String,
}

#[derive(Default)]
pub struct PendingState(RefCell<Option<PendingConfirmation>>);

#[expect(clippy::too_many_arguments)]
pub fn request_install(
	window: &MainWindow,
	settings: &Rc<RefCell<AppSettings>>,
	pending: &Rc<PendingState>,
	database: Arc<DbMutex>,
	game: Game,
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
		Box::new(move |dialog_window, dialog_database| {
			analytics::send_mod(
				rai_pal_backend::analytics::Event::InstallMod,
				&mod_id,
				Some(&game),
			);
			mods::install_mod(
				dialog_database,
				dialog_window.as_weak(),
				game,
				mod_id,
				downloads,
			);
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
	game: Game,
	confirmation_text: String,
	confirm_label: String,
) {
	request(
		window,
		settings,
		pending,
		database,
		Box::new(move |dialog_window, dialog_database| {
			mods::uninstall_all_mods(dialog_database, dialog_window.as_weak(), game);
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
	game: Game,
	confirmation_text: String,
	confirm_label: String,
) {
	let action_database = Arc::clone(&database);

	request(
		window,
		settings,
		pending,
		database,
		Box::new(move |dialog_window, _| {
			crate::manual::remove_game(
				dialog_window,
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

#[expect(clippy::too_many_arguments)]
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

	window
		.global::<crate::ConfirmationState>()
		.set_text(confirmation_text.into());
	window
		.global::<crate::ConfirmationState>()
		.set_confirm_label(confirm_label.into());
	window
		.global::<crate::ConfirmationState>()
		.set_can_skip(can_skip);
	window
		.global::<crate::ConfirmationState>()
		.set_dont_ask_again(false);
	window.global::<crate::ConfirmationState>().set_open(true);
}

pub fn respond(
	window: &MainWindow,
	settings: &Rc<RefCell<AppSettings>>,
	pending: &Rc<PendingState>,
	database: Arc<DbMutex>,
	confirmed: bool,
) {
	window.global::<crate::ConfirmationState>().set_open(false);

	let Some(pending) = pending.0.borrow_mut().take() else {
		return;
	};

	if !confirmed {
		return;
	}

	if !pending.skip_id.is_empty()
		&& window
			.global::<crate::ConfirmationState>()
			.get_dont_ask_again()
	{
		let dialog_id = format!("skip-confirm-{}", pending.skip_id);
		settings.borrow_mut().skip_confirm_dialogs.insert(dialog_id);
		persist_settings(settings);
	}

	execute(window, database, pending.action);
}

fn execute(window: &MainWindow, database: Arc<DbMutex>, action: ConfirmationFn) {
	action(window, database);
}
