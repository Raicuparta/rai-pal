use std::{
	borrow::Borrow,
	collections::HashMap,
	fmt::Display,
	hash::Hash,
	sync::Mutex,
};

use crate::{
	installed_game,
	local_mod,
	maps::TryGettable,
	mod_loaders::mod_loader,
	owned_game,
	remote_mod,
	Error,
	Result,
};

pub struct AppState {
	pub installed_games: Mutex<Option<installed_game::Map>>,
	pub owned_games: Mutex<Option<owned_game::Map>>,
	pub mod_loaders: Mutex<Option<mod_loader::Map>>,
	pub local_mods: Mutex<Option<local_mod::Map>>,
	pub remote_mods: Mutex<Option<remote_mod::Map>>,
}

pub type TauriState<'a> = tauri::State<'a, AppState>;

pub trait StateData<TData: Clone> {
	fn get_data(&self) -> Result<TData>;
}

impl<TData: Clone> StateData<TData> for Mutex<Option<TData>> {
	fn get_data(&self) -> Result<TData>
	where
		TData: Clone,
	{
		let guard = self
			.try_lock()
			.map_err(|err| Error::FailedToAccessStateData(err.to_string()))?;

		Ok(guard.as_ref().ok_or(Error::EmptyStateData())?.clone())
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
		self.get_data()?.try_get(key).cloned()
	}
}
