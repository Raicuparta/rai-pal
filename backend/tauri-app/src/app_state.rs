use std::{
	collections::HashMap,
	sync::RwLock,
};

use rai_pal_core::{
	game_mods::game_mod::GameMod,
	http::DownloadStatus,
	local_database::{
		self,
		DbMutex,
	},
};
use tauri::{
	Manager,
	ipc::Channel,
};

use crate::result::{
	AppError,
	AppResult,
};

pub struct AppState {
	pub local_mods: RwLock<Option<HashMap<String, GameMod>>>,
	pub remote_mods: RwLock<Option<HashMap<String, GameMod>>>,
	pub database: DbMutex,
	pub download_status_channel: RwLock<Option<Channel<DownloadStatus>>>,
}

type TauriState<'a> = tauri::State<'a, AppState>;

pub trait StateData<TData> {
	fn read_state(&self) -> AppResult<TData>;
	fn write_state_value(&self, data: TData) -> AppResult;
}

impl<TData: Clone> StateData<TData> for RwLock<Option<TData>> {
	fn read_state(&self) -> AppResult<TData> {
		let guard = self
			.read()
			.map_err(|err| AppError::FailedToAccessStateData(err.to_string()))?;

		(*guard).as_ref().map_or_else(
			|| Err(AppError::FailedToAccessStateData("Empty data".to_string())),
			|data| Ok(data.clone()),
		)
	}

	fn write_state_value(&self, data: TData) -> AppResult {
		*self
			.write()
			.map_err(|err| AppError::FailedToAccessStateData(err.to_string()))? = Some(data);

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
	pub fn new() -> AppResult<Self> {
		Ok(Self {
			local_mods: RwLock::new(Some(HashMap::new())),
			remote_mods: RwLock::new(Some(HashMap::new())),
			database: local_database::try_create()?,
			download_status_channel: RwLock::new(None),
		})
	}
}
