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
	remote_game::RemoteGame,
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
}

pub type Map = HashMap<String, Provider>;

fn create_map_entry<TProvider: ProviderActions + ProviderStatic>() -> Result<(String, Provider)>
where
	Provider: From<TProvider>,
{
	let provider: Provider = TProvider::new()?.into();

	Ok((TProvider::ID.to_string(), provider))
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
