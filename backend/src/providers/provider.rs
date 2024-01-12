use std::{
	collections::HashMap,
	fs,
	path::PathBuf,
	time::Instant,
};

use async_trait::async_trait;
use enum_dispatch::enum_dispatch;
use log::error;

use super::{
	epic_provider::Epic,
	gog_provider::Gog,
	xbox_provider::Xbox,
};
use crate::{
	debug::LoggableInstant,
	game_engines::game_engine::GameEngine,
	installed_game::InstalledGame,
	owned_game::OwnedGame,
	paths,
	providers::{
		manual_provider::Manual,
		steam_provider::Steam,
	},
	serializable_enum,
	Result,
};

serializable_enum!(ProviderId {
	Steam,
	Manual,
	Epic,
	Gog,
	Xbox,
});

#[enum_dispatch]
pub enum Provider {
	Steam,
	Manual,
	Epic,
	Gog,
	Xbox,
}

#[async_trait]
#[enum_dispatch(Provider)]
pub trait ProviderActions {
	fn get_installed_games(&self) -> Result<Vec<InstalledGame>>;

	async fn get_owned_games(&self) -> Result<Vec<OwnedGame>>;
}

pub type EngineCache = HashMap<String, Option<GameEngine>>;

pub trait ProviderStatic: ProviderActions {
	const ID: &'static ProviderId;

	fn new() -> Result<Self>
	where
		Self: Sized;

	fn get_folder() -> Result<PathBuf> {
		let path = paths::app_data_path()?
			.join("providers")
			.join(Self::ID.to_string());

		fs::create_dir_all(&path)?;

		Ok(path)
	}

	fn get_engine_cache_path() -> Result<PathBuf> {
		Ok(Self::get_folder()?.join("engine-cache.json"))
	}

	fn save_engine_cache(cache: &EngineCache) -> Result {
		let json = serde_json::to_string_pretty(cache)?;
		fs::write(Self::get_engine_cache_path()?, json)?;
		Ok(())
	}

	fn try_save_engine_cache(cache: &EngineCache) {
		if let Err(err) = Self::save_engine_cache(cache) {
			error!(
				"Failed to save engine cache for provider '{}'. Error: {}",
				Self::ID,
				err
			);
		}
	}

	fn get_engine_cache() -> Result<EngineCache> {
		let json = fs::read_to_string(Self::get_engine_cache_path()?)?;
		Ok(serde_json::from_str::<EngineCache>(&json)?)
	}

	fn try_get_engine_cache() -> EngineCache {
		match Self::get_engine_cache() {
			Ok(pc_gaming_wiki_cache) => pc_gaming_wiki_cache,
			Err(err) => {
				error!(
					"Failed to get engine cache for provider '{}'. Error: {}",
					Self::ID,
					err
				);
				HashMap::default()
			}
		}
	}
}

type Map = HashMap<String, Provider>;

fn create_map_entry<TProvider: ProviderActions + ProviderStatic>() -> Result<(String, Provider)>
where
	Provider: From<TProvider>,
{
	let mod_loader: Provider = TProvider::new()?.into();

	Ok((TProvider::ID.to_string(), mod_loader))
}

fn add_entry<TProvider: ProviderActions + ProviderStatic>(map: &mut Map)
where
	Provider: From<TProvider>,
{
	match create_map_entry::<TProvider>() {
		Ok((key, value)) => {
			map.insert(key, value);
		}
		Err(error) => error!("Failed to set up provider: {error}"),
	}
}

pub fn get_map() -> Map {
	let mut map = Map::new();
	let now = &mut Instant::now();

	add_entry::<Steam>(&mut map);
	now.log_next("set up provider (Steam)");

	add_entry::<Epic>(&mut map);
	now.log_next("set up provider (Epic)");

	add_entry::<Gog>(&mut map);
	now.log_next("set up provider (Gog)");

	add_entry::<Xbox>(&mut map);
	now.log_next("set up provider (Xbox)");

	add_entry::<Manual>(&mut map);
	now.log_next("set up provider (Manual)");

	map
}
