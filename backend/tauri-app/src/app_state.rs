use std::sync::RwLock;

use rai_pal_core::{
	game_providers::game_provider::GameProviderId,
	local_database::{
		app_database::DbMutex,
		game_database::{
			self,
		},
		mod_database::ModDatabase,
	},
	progress_status::ProgressStatus,
};
use tauri::{
	Manager,
	ipc::Channel,
};
use tokio::sync::Mutex as AsyncMutex;

use crate::result::{
	Error,
	Result,
};

pub struct AppState {
	pub database: DbMutex,
	pub download_status_channel: RwLock<Option<Channel<ProgressStatus>>>,
	pub selected_game: RwLock<Option<(GameProviderId, String)>>,
	pub install_lock: AsyncMutex<()>,
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
		})
	}
}
