use std::cell::RefCell;

use rai_pal_backend::{app_settings::SkipConfirmation, game::Game};
use slint::ComponentHandle;

use crate::ConfirmKind;

use super::{CommandHandler, on_command};

enum PendingAction {
	InstallMod { game: Game, mod_id: String },
	UninstallAllMods { game: Game },
	RemoveGame { game: Game },
}

struct PendingConfirmation {
	action: PendingAction,
	skip: Option<SkipConfirmation>,
}

#[derive(Default)]
pub struct PendingState(RefCell<Option<PendingConfirmation>>);

impl CommandHandler {
	pub fn prompt_install_mod(&self, mod_id: String, skip: crate::SkipConfirmation) {
		let Some((_, game)) = self.selected_game() else {
			return;
		};

		self.prompt(
			PendingAction::InstallMod { game, mod_id },
			ConfirmKind::InstallMod,
			skip.into(),
		);
	}

	pub fn prompt_uninstall_all_mods(&self) {
		let Some((_, game)) = self.selected_game() else {
			return;
		};

		self.prompt(
			PendingAction::UninstallAllMods { game },
			ConfirmKind::UninstallAllMods,
			None,
		);
	}

	pub fn prompt_remove_game(&self) {
		let Some((_, game)) = self.selected_game() else {
			return;
		};

		self.prompt(
			PendingAction::RemoveGame { game },
			ConfirmKind::RemoveGame,
			None,
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

		if let Some(skip) = pending.skip
			&& window
				.global::<crate::ConfirmationState>()
				.get_dont_ask_again()
		{
			self.settings.borrow_mut().skip_confirm_dialogs.insert(skip);
			self.persist_settings();
		}

		run_action(self, pending.action);
	}

	fn prompt(&self, action: PendingAction, kind: ConfirmKind, skip: Option<SkipConfirmation>) {
		let Some(window) = self.ui() else {
			return;
		};

		if skip.is_some_and(|skip| self.settings.borrow().skip_confirm_dialogs.contains(&skip)) {
			*self.pending_confirmation.0.borrow_mut() = None;
			run_action(self, action);
			return;
		}

		let can_skip = skip.is_some();

		*self.pending_confirmation.0.borrow_mut() = Some(PendingConfirmation { action, skip });

		let state = window.global::<crate::ConfirmationState>();
		state.set_kind(kind);
		state.set_can_skip(can_skip);
		state.set_dont_ask_again(false);
		state.set_open(true);
	}
}

fn run_action(handler: &CommandHandler, action: PendingAction) {
	match action {
		PendingAction::InstallMod { game, mod_id } => handler.install_mod(game, mod_id),
		PendingAction::UninstallAllMods { game } => handler.uninstall_all_mods(game),
		PendingAction::RemoveGame { game } => handler.remove_game(game),
	}
}

pub fn register(window: &crate::MainWindow) {
	on_command!(window, DialogActions, on_confirmation_response, |confirmed| handler => {
		handler.respond_to_confirmation(confirmed);
	});
	on_command!(window, GamesActions, on_remove_game, || handler => {
		handler.prompt_remove_game();
	});
	on_command!(window, ModActions, on_request_install, |mod_id, skip| handler => {
		handler.prompt_install_mod(mod_id.to_string(), skip);
	});
	on_command!(window, ModActions, on_request_uninstall_all, || handler => {
		handler.prompt_uninstall_all_mods();
	});
}
