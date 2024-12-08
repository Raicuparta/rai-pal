use std::{
	borrow::Borrow,
	collections::HashMap,
	fmt::Display,
	hash::{BuildHasher, Hash},
	sync::Mutex,
};

use crate::result::Error;
use crate::result::Result;
use tauri::Manager;

use rai_pal_core::{
	installed_game::{DataQuery, InstalledGame},
	local_mod,
	maps::TryGettable,
	mod_loaders::mod_loader,
	owned_game::OwnedGame,
	providers::provider::ProviderId,
	remote_mod,
};

pub struct AppState {
	pub installed_games: HashMap<ProviderId, Mutex<Option<HashMap<String, InstalledGame>>>>,
	pub owned_games: HashMap<ProviderId, Mutex<Option<HashMap<String, OwnedGame>>>>,
	pub mod_loaders: Mutex<Option<mod_loader::Map>>,
	pub local_mods: Mutex<Option<local_mod::Map>>,
	pub remote_mods: Mutex<Option<remote_mod::Map>>,
	pub data_query: Mutex<Option<DataQuery>>,
}

type TauriState<'a> = tauri::State<'a, AppState>;

pub trait StateData<TData: Clone> {
	fn get_data(&self) -> Result<TData>;
}

impl<TData: Clone> StateData<TData> for Mutex<Option<TData>> {
	fn get_data(&self) -> Result<TData>
	where
		TData: Clone,
	{
		self.lock()
			.map_err(|err| Error::FailedToAccessStateData(err.to_string()))?
			.as_ref()
			.ok_or(Error::EmptyStateData())
			.cloned()
	}
}

pub trait DataValue<K, V> {
	fn try_get<Q>(&self, k: &Q) -> Result<V>
	where
		K: Borrow<Q> + Display,
		Q: Hash + Eq + Display + ?Sized;
}

impl<K, V, S: BuildHasher> DataValue<K, V> for Mutex<Option<HashMap<K, V, S>>>
where
	K: Hash + Eq + Display + Clone,
	V: Clone,
{
	fn try_get<Q>(&self, key: &Q) -> Result<V>
	where
		K: Borrow<Q>,
		Q: ?Sized + Hash + Display + Eq,
	{
		Ok(self
			.try_lock()
			.map_err(|err| Error::FailedToAccessStateData(err.to_string()))?
			.as_ref()
			.ok_or(Error::EmptyStateData())?
			.try_get(key)
			.cloned()?)
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
