use std::{
	collections::HashMap,
	ops::Deref,
	sync::RwLock,
};

use rai_pal_core::{
	game_mods::game_mod::GameMod,
	local_database::{
		self,
		DbMutex,
	},
};
use tauri::Manager;

use crate::result::{
	Error,
	Result,
};

pub struct AppState {
	pub local_mods: RwLock<HashMap<String, GameMod>>,
	pub remote_mods: RwLock<HashMap<String, GameMod>>,
	pub database: DbMutex,
}

type TauriState<'a> = tauri::State<'a, AppState>;

pub trait StateData<TData> {
	fn read_state(&self) -> Result<impl Deref<Target = TData>>;
	fn write_state_value(&self, data: TData) -> Result;
}

impl<TData: Clone> StateData<TData> for RwLock<TData> {
	fn read_state(&self) -> Result<impl Deref<Target = TData>> {
		self.read()
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

impl AppState {
	pub fn new() -> Result<Self> {
		Ok(Self {
			local_mods: RwLock::new(HashMap::new()),
			remote_mods: RwLock::new(HashMap::new()),
			database: local_database::try_create()?,
		})
	}
}
