use std::{
	collections::HashMap,
	sync::RwLock,
};

use rai_pal_core::{
	http::DownloadStatus,
	local_database::game_database::{
		self,
		DbMutex,
	},
	mods::game_mod::GameMod,
};
use tauri::{
	Manager,
	ipc::Channel,
};

use crate::result::{
	Error,
	Result,
};

pub struct AppState {
	pub mods: RwLock<Option<HashMap<String, GameMod>>>,
	pub database: DbMutex,
	pub download_status_channel: RwLock<Option<Channel<DownloadStatus>>>,
}

type TauriState<'a> = tauri::State<'a, AppState>;

pub trait StateData<TData> {
	fn read_state(&self) -> Result<TData>;
	fn write_state_value(&self, data: TData) -> Result;
}

impl<TData: Clone> StateData<TData> for RwLock<Option<TData>> {
	fn read_state(&self) -> Result<TData> {
		let guard = self
			.read()
			.map_err(|err| Error::FailedToAccessStateData(err.to_string()))?;

		(*guard).as_ref().map_or_else(
			|| Err(Error::FailedToAccessStateData("Empty data".into())),
			|data| Ok(data.clone()),
		)
	}

	fn write_state_value(&self, data: TData) -> Result<()> {
		*self
			.write()
			.map_err(|err| Error::FailedToAccessStateData(err.to_string()))? = Some(data);

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
			mods: RwLock::new(Some(HashMap::new())),
			database: game_database::try_create()?,
			download_status_channel: RwLock::new(None),
		})
	}
}
