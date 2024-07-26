use std::{
	collections::HashMap,
	fs,
	marker::{Send, Sync},
	path::PathBuf,
	time::Instant,
};

use async_trait::async_trait;
use enum_dispatch::enum_dispatch;
use log::error;

#[cfg(target_os = "windows")]
use crate::providers::{epic_provider::Epic, gog_provider::Gog, xbox_provider::Xbox};
use crate::{
	debug::LoggableInstant,
	installed_game::InstalledGame,
	owned_game::OwnedGame,
	paths,
	providers::{itch_provider::Itch, manual_provider::Manual, steam_provider::Steam},
	remote_game::{self, RemoteGame},
	result::Result,
	serializable_enum,
};

serializable_enum!(ProviderId {
	Steam,
	Manual,
	Itch,
	Epic,
	Gog,
	Xbox,
});

#[enum_dispatch]
#[derive(Clone)]
pub enum Provider {
	Steam,
	Manual,
	Itch,
	#[cfg(target_os = "windows")]
	Epic,
	#[cfg(target_os = "windows")]
	Gog,
	#[cfg(target_os = "windows")]
	Xbox,
}

#[async_trait]
#[enum_dispatch(Provider)]
pub trait ProviderActions {
	async fn get_games<TInstalledCallback, TOwnedCallback, TRemoteCallback>(
		&self,
		installed_callback: TInstalledCallback,
		owned_callback: TOwnedCallback,
		remote_callback: TRemoteCallback,
	) -> Result
	where
		TInstalledCallback: Fn(InstalledGame) + Send + Sync,
		TOwnedCallback: Fn(OwnedGame) + Send + Sync,
		TRemoteCallback: Fn(RemoteGame) + Send + Sync;
}

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

	fn get_remote_game_cache_path() -> Result<PathBuf> {
		Ok(Self::get_folder()?.join("remote-game-cache.json"))
	}

	fn save_remote_game_cache(cache: &remote_game::Map) -> Result {
		let json = serde_json::to_string_pretty(cache)?;
		fs::write(Self::get_remote_game_cache_path()?, json)?;
		Ok(())
	}

	fn try_save_remote_game_cache(remote_games: &[RemoteGame]) {
		if let Err(err) = Self::save_remote_game_cache(
			&remote_games
				.iter()
				.filter_map(|remote_game| {
					if remote_game.skip_cache {
						None
					} else {
						Some((remote_game.id.clone(), remote_game.clone()))
					}
				})
				.collect(),
		) {
			error!(
				"Failed to save engine cache for provider '{}'. Error: {}",
				Self::ID,
				err
			);
		}
	}

	fn get_remote_game_cache() -> Result<remote_game::Map> {
		let json = fs::read_to_string(Self::get_remote_game_cache_path()?)?;
		Ok(serde_json::from_str::<remote_game::Map>(&json)?)
	}

	fn try_get_remote_game_cache() -> remote_game::Map {
		match Self::get_remote_game_cache() {
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

pub type Map = HashMap<String, Provider>;

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

	add_entry::<Itch>(&mut map);
	now.log_next("set up provider (Itch)");

	add_entry::<Manual>(&mut map);
	now.log_next("set up provider (Manual)");

	#[cfg(target_os = "windows")]
	{
		add_entry::<Epic>(&mut map);
		now.log_next("set up provider (Epic)");

		add_entry::<Gog>(&mut map);
		now.log_next("set up provider (Gog)");

		add_entry::<Xbox>(&mut map);
		now.log_next("set up provider (Xbox)");
	}
	map
}
