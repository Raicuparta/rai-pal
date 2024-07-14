use std::{borrow::Borrow, collections::HashMap, fmt::Display, hash::Hash, sync::Mutex};

use tauri::Manager;

use crate::{local_mod, maps::TryGettable, mod_loaders::mod_loader, remote_mod, Error, Result};

pub struct AppState {
	pub mod_loaders: Mutex<Option<mod_loader::Map>>,
	pub local_mods: Mutex<Option<local_mod::Map>>,
	pub remote_mods: Mutex<Option<remote_mod::Map>>,
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
		self.try_lock()
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

impl<K, V> DataValue<K, V> for Mutex<Option<HashMap<K, V>>>
where
	K: Hash + Eq + Display + Clone,
	V: Clone,
{
	fn try_get<Q>(&self, key: &Q) -> Result<V>
	where
		K: Borrow<Q>,
		Q: ?Sized + Hash + Display + Eq,
	{
		self.try_lock()
			.map_err(|err| Error::FailedToAccessStateData(err.to_string()))?
			.as_ref()
			.ok_or(Error::EmptyStateData())?
			.try_get(key)
			.cloned()
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
