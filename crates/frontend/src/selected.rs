use slint::ComponentHandle as _;

use rai_pal_backend::game::Game;

use crate::{
	GameRowData, ProviderActionData,
	commands::CommandHandler,
	model::{self, apply_game_to_row},
};

impl CommandHandler {
	pub fn show_game_details(&self, game: &Game, mut row: GameRowData) {
		let Some(window) = self.ui() else {
			return;
		};

		apply_game_to_row(&mut row, game);

		let provider_actions = game
			.provider_commands
			.iter()
			.map(|provider_command| ProviderActionData {
				action: provider_command.action.into(),
				action_id: provider_command.action.to_string().into(),
			})
			.collect();

		window.set_selected_game_row(row);
		let state = window.global::<crate::GameDetailsState>();
		state.set_provider_name(game.provider_id.display_name().into());
		state.set_show_wine_folders(cfg!(target_os = "linux"));
		model::sync_vec_model(&self.models.game_details.provider_actions, provider_actions);
		window.set_game_selected(true);
	}

	pub fn hide_game_details(&self) {
		self.clear_selected_game();
		self.models.game_details.clear();

		let Some(window) = self.ui() else {
			return;
		};

		window.set_game_selected(false);
		window.set_selected_game_row(GameRowData::default());
		let state = window.global::<crate::GameDetailsState>();
		state.set_provider_name("".into());
		state.set_show_wine_folders(false);
	}
}
