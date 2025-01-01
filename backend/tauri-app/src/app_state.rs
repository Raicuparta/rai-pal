use std::ops::DerefMut;
use std::sync::RwLock;
use std::{collections::HashMap, ops::Deref};

use crate::result::Error;
use crate::result::Result;
use tauri::Manager;

use rai_pal_core::{
	game::Game, local_mod, mod_loaders::mod_loader, providers::provider::ProviderId, remote_game,
	remote_mod,
};

pub struct AppState {
	pub games: HashMap<ProviderId, RwLock<HashMap<String, Game>>>,
	pub remote_games: RwLock<remote_game::Map>,
	pub mod_loaders: RwLock<mod_loader::Map>,
	pub local_mods: RwLock<local_mod::Map>,
	pub remote_mods: RwLock<remote_mod::Map>,
}

type TauriState<'a> = tauri::State<'a, AppState>;

pub trait StateData<TData> {
	fn read_state(&self) -> Result<impl Deref<Target = TData>>;
	fn write_state(&self) -> Result<impl DerefMut<Target = TData>>;
	fn write_state_value(&self, data: TData) -> Result;
}

impl<TData: Clone> StateData<TData> for RwLock<TData> {
	fn read_state(&self) -> Result<impl Deref<Target = TData>> {
		self.read()
			.map_err(|err| Error::FailedToAccessStateData(err.to_string()))
	}

	fn write_state(&self) -> Result<impl DerefMut<Target = TData>> {
		self.write()
			.map_err(|err| Error::FailedToAccessStateData(err.to_string()))
	}

	fn write_state_value(&self, data: TData) -> Result {
		*self
			.write()
			.map_err(|err| Error::FailedToAccessStateData(err.to_string()))? = data;
		Ok(())
	}
}

pub trait StatefulHandle {
	fn app_state(&self) -> TauriState<'_>;
}

impl StatefulHandle for tauri::AppHandle {
	fn app_state(&self) -> TauriState<'_> {
		self.state::<AppState>()
	}
}
