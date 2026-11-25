use std::rc::Rc;

use slint::{ComponentHandle as _, VecModel};

use crate::{GameModData, MainWindow, ModsTabModData, ProviderActionData};

pub struct UiModels {
	pub game_details: GameDetailsModels,
	pub mods_tab: Rc<VecModel<ModsTabModData>>,
}

impl UiModels {
	pub fn new(window: &MainWindow) -> Self {
		let game_details = GameDetailsModels::new(window);
		let mods_tab = Rc::new(VecModel::default());
		window
			.global::<crate::ModsState>()
			.set_items(mods_tab.clone().into());

		Self {
			game_details,
			mods_tab,
		}
	}
}

pub struct GameDetailsModels {
	pub provider_actions: Rc<VecModel<ProviderActionData>>,
	pub installed: Rc<VecModel<GameModData>>,
	pub game_specific: Rc<VecModel<GameModData>>,
	pub available: Rc<VecModel<GameModData>>,
	pub dependency_only: Rc<VecModel<GameModData>>,
	pub incompatible: Rc<VecModel<GameModData>>,
}

impl GameDetailsModels {
	fn new(window: &MainWindow) -> Self {
		let provider_actions = Rc::new(VecModel::default());
		let installed = Rc::new(VecModel::default());
		let game_specific = Rc::new(VecModel::default());
		let available = Rc::new(VecModel::default());
		let dependency_only = Rc::new(VecModel::default());
		let incompatible = Rc::new(VecModel::default());

		let state = window.global::<crate::GameDetailsState>();
		state.set_provider_actions(provider_actions.clone().into());
		state.set_installed(installed.clone().into());
		state.set_game_specific(game_specific.clone().into());
		state.set_available(available.clone().into());
		state.set_dependency_only(dependency_only.clone().into());
		state.set_incompatible(incompatible.clone().into());

		Self {
			provider_actions,
			installed,
			game_specific,
			available,
			dependency_only,
			incompatible,
		}
	}

	pub fn clear(&self) {
		self.provider_actions.clear();
		self.installed.clear();
		self.game_specific.clear();
		self.available.clear();
		self.dependency_only.clear();
		self.incompatible.clear();
	}
}
