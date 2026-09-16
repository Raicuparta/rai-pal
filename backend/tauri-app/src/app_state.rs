use std::{
	collections::BTreeMap,
	sync::{Mutex, RwLock},
};

use rai_pal_core::{
	game_providers::game_provider::GameProviderId,
	local_database::{
		app_database::DbMutex,
		game_database::{self},
		mod_database::ModDatabase,
	},
	progress_status::ProgressStatus,
};
use rai_pal_proc_macros::serializable_struct;
use tauri::{Manager, ipc::Channel};
use tokio::sync::Mutex as AsyncMutex;

use crate::result::{Error, Result};

#[serializable_struct]
pub struct RunningModInfo {
	pub mod_id: String,
	pub pid: u32,
	pub started_at: u32,
}

pub struct AppState {
	pub database: DbMutex,
	pub download_status_channel: RwLock<Option<Channel<ProgressStatus>>>,
	pub selected_game: RwLock<Option<(GameProviderId, String)>>,
	pub install_lock: AsyncMutex<()>,
	pub running_mods: Mutex<BTreeMap<String, RunningModInfo>>,
}

type TauriState<'a> = tauri::State<'a, AppState>;

pub trait StateData<TData> {
	fn write_state_value(&self, data: TData) -> Result;
}

impl<TData: Clone> StateData<TData> for RwLock<Option<TData>> {
	fn write_state_value(&self, data: TData) -> Result {
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
		let games = game_database::try_create()?;
		games.setup_mod_tables()?;

		Ok(Self {
			database: games,
			download_status_channel: RwLock::new(None),
			selected_game: RwLock::new(None),
			install_lock: AsyncMutex::new(()),
			running_mods: Mutex::new(BTreeMap::new()),
		})
	}
}
