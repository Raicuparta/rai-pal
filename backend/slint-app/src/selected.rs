use std::rc::Rc;

use rai_pal_core::game::DbGame;
use slint::ModelRc;

use crate::{GameRowData, MainWindow, ProviderActionItem, actions, model::apply_game_to_row};

pub fn show(window: &MainWindow, game: &DbGame, mut row: GameRowData) {
	apply_game_to_row(&mut row, game);

	let provider_actions: Vec<ProviderActionItem> = actions::secondary_actions(game)
		.into_iter()
		.map(|action| ProviderActionItem {
			value: action.to_string().into(),
		})
		.collect();

	window.set_selected_game_row(row);
	window.set_selected_game_json(
		serde_json::to_string_pretty(game)
			.unwrap_or_default()
			.into(),
	);
	window.set_selected_provider_id(game.provider_id.to_string().into());
	window.set_selected_can_run_game(actions::primary_start(game).is_some());
	window.set_selected_has_exe_path(game.exe_path.is_some());
	window.set_selected_show_wine_folders(cfg!(target_os = "linux"));
	window.set_selected_provider_actions(Rc::new(slint::VecModel::from(provider_actions)).into());
	window.set_game_selected(true);
}

pub fn hide(window: &MainWindow) {
	window.set_game_selected(false);
	window.set_selected_game_row(GameRowData::default());
	window.set_selected_game_json("".into());
	window.set_selected_provider_id("".into());
	window.set_selected_can_run_game(false);
	window.set_selected_has_exe_path(false);
	window.set_selected_show_wine_folders(false);
	window.set_selected_provider_actions(ModelRc::default());
	window.set_game_mods(ModelRc::default());
	window.set_game_mods_hidden(ModelRc::default());
	window.set_game_mods_incompatible(ModelRc::default());
	window.set_has_installed_mods(false);
	window.set_has_compatible_mods(false);
	window.set_has_hidden_mods(false);
	window.set_has_incompatible_mods(false);
}
