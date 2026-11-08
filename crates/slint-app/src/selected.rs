use std::{
	rc::Rc,
	sync::{Arc, Mutex},
};

use rai_pal_core::game::Game;
use slint::ModelRc;

use crate::{GameRowData, MainWindow, model::apply_game_to_row};

pub type SelectedGameState = Arc<Mutex<Option<(usize, Game)>>>;

pub fn show(window: &MainWindow, game: &Game, mut row: GameRowData) {
	apply_game_to_row(&mut row, game);

	let provider_actions: Vec<slint::SharedString> = game
		.provider_commands
		.iter()
		.map(|provider_command| provider_command.action.to_string().into())
		.collect();

	window.set_selected_game_row(row);
	window.set_selected_provider_id(game.provider_id.to_string().into());
	window.set_selected_has_exe_path(game.exe_path.is_some());
	window.set_selected_show_wine_folders(cfg!(target_os = "linux"));
	window.set_selected_provider_actions(Rc::new(slint::VecModel::from(provider_actions)).into());
	window.set_game_selected(true);
}

pub fn hide(window: &MainWindow) {
	window.set_game_selected(false);
	window.set_selected_game_row(GameRowData::default());
	window.set_selected_provider_id("".into());
	window.set_selected_has_exe_path(false);
	window.set_selected_show_wine_folders(false);
	window.set_selected_provider_actions(ModelRc::default());
	window.set_game_mods_installed(ModelRc::default());
	window.set_game_mods_available(ModelRc::default());
	window.set_game_mods_hidden(ModelRc::default());
	window.set_game_mods_incompatible(ModelRc::default());
	window.set_potential_mods(ModelRc::default());
	window.set_has_any_mods(false);
	window.set_has_installed_mods(false);
	window.set_has_available_mods(false);
	window.set_has_hidden_mods(false);
	window.set_has_incompatible_mods(false);
}
