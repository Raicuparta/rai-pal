use slint::ComponentHandle as _;
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

	window.global::<crate::SelectedGameState>().set_row(row);
	window
		.global::<crate::SelectedGameState>()
		.set_provider_id(game.provider_id.to_string().into());
	window
		.global::<crate::SelectedGameState>()
		.set_has_exe_path(game.exe_path.is_some());
	window
		.global::<crate::SelectedGameState>()
		.set_show_wine_folders(cfg!(target_os = "linux"));
	window
		.global::<crate::SelectedGameState>()
		.set_provider_actions(Rc::new(slint::VecModel::from(provider_actions)).into());
	window.global::<crate::SelectedGameState>().set_active(true);
}

pub fn hide(window: &MainWindow) {
	window
		.global::<crate::SelectedGameState>()
		.set_active(false);
	window
		.global::<crate::SelectedGameState>()
		.set_row(GameRowData::default());
	window
		.global::<crate::SelectedGameState>()
		.set_provider_id("".into());
	window
		.global::<crate::SelectedGameState>()
		.set_has_exe_path(false);
	window
		.global::<crate::SelectedGameState>()
		.set_show_wine_folders(false);
	window
		.global::<crate::SelectedGameState>()
		.set_provider_actions(ModelRc::default());
	window
		.global::<crate::SelectedGameState>()
		.set_mods_installed(ModelRc::default());
	window
		.global::<crate::SelectedGameState>()
		.set_mods_available(ModelRc::default());
	window
		.global::<crate::SelectedGameState>()
		.set_mods_hidden(ModelRc::default());
	window
		.global::<crate::SelectedGameState>()
		.set_mods_incompatible(ModelRc::default());
	window
		.global::<crate::SelectedGameState>()
		.set_potential_mods(ModelRc::default());
}
