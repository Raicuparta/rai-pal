use std::ops::Deref;
use std::ops::DerefMut;
use std::sync::RwLock;

use crate::result::Error;
use crate::result::Result;
use std::sync::Mutex;
use tauri::Manager;

use rai_pal_core::{local_mod, mod_loaders::mod_loader, remote_mod};

pub struct AppState {
	pub mod_loaders: RwLock<mod_loader::Map>,
	pub local_mods: RwLock<local_mod::Map>,
	pub remote_mods: RwLock<remote_mod::Map>,
	pub database_connection: Mutex<rusqlite::Connection>,
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
