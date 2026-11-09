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

	window.set_selected_game_row(row);
	window
		.global::<crate::GameSidebarState>()
		.set_provider_id(game.provider_id.to_string().into());
	window
		.global::<crate::GameSidebarState>()
		.set_has_exe_path(game.exe_path.is_some());
	window
		.global::<crate::GameSidebarState>()
		.set_show_wine_folders(cfg!(target_os = "linux"));
	window
		.global::<crate::GameSidebarState>()
		.set_provider_actions(Rc::new(slint::VecModel::from(provider_actions)).into());
	window.set_game_selected(true);
}

pub fn hide(window: &MainWindow) {
	window.set_game_selected(false);
	window.set_selected_game_row(GameRowData::default());
	window
		.global::<crate::GameSidebarState>()
		.set_provider_id("".into());
	window
		.global::<crate::GameSidebarState>()
		.set_has_exe_path(false);
	window
		.global::<crate::GameSidebarState>()
		.set_show_wine_folders(false);
	window
		.global::<crate::GameSidebarState>()
		.set_provider_actions(ModelRc::default());
	window
		.global::<crate::GameMods>()
		.set_installed(ModelRc::default());
	window
		.global::<crate::GameMods>()
		.set_available(ModelRc::default());
	window
		.global::<crate::GameMods>()
		.set_hidden(ModelRc::default());
	window
		.global::<crate::GameMods>()
		.set_incompatible(ModelRc::default());
	window
		.global::<crate::GameMods>()
		.set_potential(ModelRc::default());
}
