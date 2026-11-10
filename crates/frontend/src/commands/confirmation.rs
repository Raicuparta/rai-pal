use std::cell::RefCell;

use slint::ComponentHandle;

use super::{CommandHandler, on_command};

type ConfirmationFn = Box<dyn FnOnce(&CommandHandler)>;

struct PendingConfirmation {
	action: ConfirmationFn,
	skip_id: String,
}

#[derive(Default)]
pub struct PendingState(RefCell<Option<PendingConfirmation>>);

impl CommandHandler {
	pub fn prompt_install_mod(
		&self,
		mod_id: String,
		confirmation_text: String,
		confirm_label: String,
		skip_id: String,
	) {
		let Some((_, game)) = self.selected_game() else {
			return;
		};

		self.prompt(
			Box::new(move |handler| handler.install_mod(game, mod_id)),
			confirmation_text,
			confirm_label,
			skip_id,
		);
	}

	pub fn prompt_uninstall_all_mods(&self, confirmation_text: String, confirm_label: String) {
		let Some((_, game)) = self.selected_game() else {
			return;
		};

		self.prompt(
			Box::new(move |handler| handler.uninstall_all_mods(game)),
			confirmation_text,
			confirm_label,
			String::new(),
		);
	}

	pub fn prompt_remove_game(&self, confirmation_text: String, confirm_label: String) {
		let Some((_, game)) = self.selected_game() else {
			return;
		};

		self.prompt(
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

	fn prompt(
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

pub fn register(window: &crate::MainWindow) {
	on_command!(window, DialogActions, on_confirmation_response, |confirmed| handler => {
		handler.respond_to_confirmation(confirmed);
	});
	on_command!(window, GamesActions, on_remove_game, |text, label| handler => {
		handler.prompt_remove_game(text.to_string(), label.to_string());
	});
	on_command!(window, ModActions, on_request_install, |mod_id, text, label, skip_id| handler => {
		handler.prompt_install_mod(
			mod_id.to_string(),
			text.to_string(),
			label.to_string(),
			skip_id.to_string(),
		);
	});
	on_command!(window, ModActions, on_request_uninstall_all, |text, label| handler => {
		handler.prompt_uninstall_all_mods(text.to_string(), label.to_string());
	});
}
