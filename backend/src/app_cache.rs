use std::{collections::HashMap, path::PathBuf};

use crate::result::Result;
use rai_pal_core::{
	installed_game::InstalledGame,
	owned_game::OwnedGame,
	providers::provider::{self, ProviderId},
};
use rai_pal_proc_macros::serializable_struct;
use serde_json::json;
use tauri::{AppHandle, Manager, Wry};
use tauri_plugin_store::{with_store, StoreCollection};

#[serializable_struct]
pub struct ProviderCacheData {
	pub installed_games: HashMap<String, InstalledGame>,
	pub owned_games: HashMap<String, OwnedGame>,
}

pub struct ProviderCache {
	path: PathBuf,
	pub data: ProviderCacheData,
}

impl ProviderCache {
	const DATA_KEY: &'static str = "data";
	const CACHE_FOLDER: &'static str = "cache";

	pub fn new(id: ProviderId) -> Self {
		Self {
			path: PathBuf::from(Self::CACHE_FOLDER)
				.join("providers")
				.join(id.to_string()),
			data: ProviderCacheData {
				installed_games: HashMap::new(),
				owned_games: HashMap::new(),
			},
		}
	}

	pub fn load(&mut self, handle: &AppHandle) -> Result {
		let stores = handle.state::<StoreCollection<Wry>>();

		let mut cache_data = ProviderCacheData {
			installed_games: HashMap::new(),
			owned_games: HashMap::new(),
		};

		with_store(handle.clone(), stores, self.path.clone(), |store| {
			if let Some(data) = store.get(Self::DATA_KEY) {
				cache_data = serde_json::from_value(data.clone())?;
			}

			Ok(())
		})?;

		self.data = cache_data;

		Ok(())
	}

	pub fn save(&self, handle: &AppHandle) -> Result {
		let stores = handle.state::<StoreCollection<Wry>>();

		with_store(handle.clone(), stores, self.path.clone(), |store| {
			store.insert(Self::DATA_KEY.to_string(), json!(self.data))?;
			store.save()?;
			Ok(())
		})?;

		Ok(())
	}

	pub fn set_data(&mut self, data: ProviderCacheData) -> &mut Self {
		self.data = data;
		self
	}

	pub fn clear(&self, handle: &AppHandle) -> Result {
		let stores = handle.state::<StoreCollection<Wry>>();

		with_store(handle.clone(), stores, self.path.clone(), |store| {
			store.clear()?;
			store.save()?;

			Ok(())
		})?;
		Ok(())
	}

	pub fn clear_all(handle: &AppHandle) -> Result {
		for provider_id in provider::get_provider_ids() {
			Self::new(provider_id).clear(handle)?;
		}

		Ok(())
	}
}
