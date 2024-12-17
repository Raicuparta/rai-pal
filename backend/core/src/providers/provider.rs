use std::{
	fs,
	marker::{Send, Sync},
	path::PathBuf,
};

use enum_dispatch::enum_dispatch;
use rai_pal_proc_macros::serializable_enum;

#[cfg(target_os = "windows")]
use crate::providers::{epic_provider::Epic, gog_provider::Gog};
use crate::{
	game::Game,
	paths,
	providers::{
		itch_provider::Itch, manual_provider::Manual, steam_provider::Steam, xbox_provider::Xbox,
	},
	result::{Error, Result},
};

#[serializable_enum]
pub enum ProviderId {
	Steam,
	Manual,
	Itch,
	Epic,
	Gog,
	Xbox,
}

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
	Xbox,
}

type Map = [(ProviderId, fn() -> Result<Provider>)];
const PROVIDERS: &Map = &[
	create_map_entry::<Steam>(),
	create_map_entry::<Manual>(),
	create_map_entry::<Itch>(),
	create_map_entry::<Xbox>(),
	#[cfg(target_os = "windows")]
	create_map_entry::<Epic>(),
	#[cfg(target_os = "windows")]
	create_map_entry::<Gog>(),
];

#[enum_dispatch(Provider)]
pub trait ProviderActions {
	async fn get_games_new<TCallback>(&self, callback: TCallback) -> Result
	where
		TCallback: FnMut(Game) + Send + Sync;
}

const fn create_map_entry<TProvider: ProviderActions + ProviderStatic>(
) -> (ProviderId, fn() -> Result<Provider>)
where
	Provider: From<TProvider>,
{
	(*TProvider::ID, || Ok(TProvider::new()?.into()))
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

pub fn get_provider(provider_id: ProviderId) -> Result<Provider> {
	for &(id, create_provider) in PROVIDERS {
		if id == provider_id {
			return create_provider();
		}
	}
	Err(Error::InvalidProviderId(provider_id.to_string()))
}

pub fn get_provider_ids() -> Vec<ProviderId> {
	PROVIDERS.iter().map(|&(id, _)| id).collect()
}
