use slint::ComponentHandle as _;
use std::{
	rc::Rc,
	sync::{Arc, Mutex},
};

use rai_pal_backend::game::Game;
use slint::ModelRc;

use crate::{GameRowData, MainWindow, ProviderActionData, model::apply_game_to_row};

pub type SelectedGameState = Arc<Mutex<Option<(usize, Game)>>>;

pub fn show(window: &MainWindow, game: &Game, mut row: GameRowData) {
	apply_game_to_row(&mut row, game);

	let provider_actions: Vec<ProviderActionData> = game
		.provider_commands
		.iter()
		.map(|provider_command| ProviderActionData {
			action: provider_command.action.into(),
			action_id: provider_command.action.to_string().into(),
		})
		.collect();

	window.set_selected_game_row(row);
	window
		.global::<crate::GameDetailsState>()
		.set_provider_name(game.provider_id.display_name().into());
	window
		.global::<crate::GameDetailsState>()
		.set_show_wine_folders(cfg!(target_os = "linux"));
	window
		.global::<crate::GameDetailsState>()
		.set_provider_actions(Rc::new(slint::VecModel::from(provider_actions)).into());
	window.set_game_selected(true);
}

pub fn hide(window: &MainWindow) {
	window.set_game_selected(false);
	window.set_selected_game_row(GameRowData::default());
	window
		.global::<crate::GameDetailsState>()
		.set_provider_name("".into());
	window
		.global::<crate::GameDetailsState>()
		.set_show_wine_folders(false);
	window
		.global::<crate::GameDetailsState>()
		.set_provider_actions(ModelRc::default());
	window
		.global::<crate::GameDetailsState>()
		.set_installed(ModelRc::default());
	window
		.global::<crate::GameDetailsState>()
		.set_available(ModelRc::default());
	window
		.global::<crate::GameDetailsState>()
		.set_hidden(ModelRc::default());
	window
		.global::<crate::GameDetailsState>()
		.set_incompatible(ModelRc::default());
	window
		.global::<crate::GameDetailsState>()
		.set_potential(ModelRc::default());
}
