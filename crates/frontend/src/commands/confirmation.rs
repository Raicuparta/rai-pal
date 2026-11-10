use std::cell::RefCell;

use rai_pal_backend::game::Game;
use slint::ComponentHandle;

use crate::analytics;

use super::CommandHandler;

type ConfirmationFn = Box<dyn FnOnce(&CommandHandler)>;

struct PendingConfirmation {
	action: ConfirmationFn,
	skip_id: String,
}

#[derive(Default)]
pub struct PendingState(RefCell<Option<PendingConfirmation>>);

impl CommandHandler {
	pub fn request_install(
		&self,
		game: Game,
		mod_id: String,
		confirmation_text: String,
		confirm_label: String,
		skip_id: String,
	) {
		let analytics_mod_id = mod_id.clone();
		let analytics_game = game.clone();

		self.request(
			Box::new(move |handler| {
				analytics::send_mod(
					rai_pal_backend::analytics::Event::InstallMod,
					&analytics_mod_id,
					Some(&analytics_game),
				);
				handler.install_mod(game, mod_id);
			}),
			confirmation_text,
			confirm_label,
			skip_id,
		);
	}

	pub fn request_uninstall_all(
		&self,
		game: Game,
		confirmation_text: String,
		confirm_label: String,
	) {
		self.request(
			Box::new(move |handler| handler.uninstall_all_mods(game)),
			confirmation_text,
			confirm_label,
			String::new(),
		);
	}

	pub fn request_remove_game(
		&self,
		game: Game,
		confirmation_text: String,
		confirm_label: String,
	) {
		self.request(
			Box::new(move |handler| handler.remove_game(game)),
			confirmation_text,
			confirm_label,
			String::new(),
		);
	}

	pub fn respond_to_confirmation(&self, confirmed: bool) {
		let Some(window) = self.ui() else {
			return;
		};

		window.global::<crate::ConfirmationState>().set_open(false);

		let Some(pending) = self.pending_confirmation.0.borrow_mut().take() else {
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
			self.settings
				.borrow_mut()
				.skip_confirm_dialogs
				.insert(dialog_id);
			self.persist_settings();
		}

		(pending.action)(self);
	}

	fn request(
		&self,
		action: ConfirmationFn,
		confirmation_text: String,
		confirm_label: String,
		skip_id: String,
	) {
		let Some(window) = self.ui() else {
			return;
		};

		let dialog_id = if skip_id.is_empty() {
			String::new()
		} else {
			format!("skip-confirm-{skip_id}")
		};

		if !dialog_id.is_empty()
			&& self
				.settings
				.borrow()
				.skip_confirm_dialogs
				.contains(&dialog_id)
		{
			*self.pending_confirmation.0.borrow_mut() = None;
			action(self);
			return;
		}

		let can_skip = !skip_id.is_empty();

		*self.pending_confirmation.0.borrow_mut() = Some(PendingConfirmation { action, skip_id });

		let state = window.global::<crate::ConfirmationState>();
		state.set_text(confirmation_text.into());
		state.set_confirm_label(confirm_label.into());
		state.set_can_skip(can_skip);
		state.set_dont_ask_again(false);
		state.set_open(true);
	}
}
