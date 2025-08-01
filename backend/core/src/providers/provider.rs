use std::{fs, path::PathBuf};

use enum_dispatch::enum_dispatch;
use rai_pal_proc_macros::serializable_enum;

#[cfg(target_os = "linux")]
use crate::providers::heroic_epic_provider::HeroicEpic;
#[cfg(target_os = "linux")]
use crate::providers::heroic_gog_provider::HeroicGog;
#[cfg(target_os = "windows")]
use crate::providers::{epic_provider::Epic, gog_provider::Gog, xbox_provider::Xbox};
use crate::{
	local_database::DbMutex,
	paths,
	providers::{itch_provider::Itch, manual_provider::Manual, steam::steam_provider::Steam},
	result::{Error, Result},
};

// These IDs need to match the ones in rai-pal-db.
#[serializable_enum]
pub enum ProviderId {
	Epic,
	Gog,
	Itch,
	Manual,
	Steam,
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
	#[cfg(target_os = "windows")]
	Xbox,
	#[cfg(target_os = "linux")]
	HeroicEpic,
	#[cfg(target_os = "linux")]
	HeroicGog,
}

type Map = [(ProviderId, fn() -> Result<Provider>)];
const PROVIDERS: &Map = &[
	create_map_entry::<Steam>(),
	create_map_entry::<Manual>(),
	create_map_entry::<Itch>(),
	#[cfg(target_os = "linux")]
	create_map_entry::<HeroicEpic>(),
	#[cfg(target_os = "linux")]
	create_map_entry::<HeroicGog>(),
	#[cfg(target_os = "windows")]
	create_map_entry::<Epic>(),
	#[cfg(target_os = "windows")]
	create_map_entry::<Gog>(),
	#[cfg(target_os = "windows")]
	create_map_entry::<Xbox>(),
];

#[enum_dispatch(Provider)]
pub trait ProviderActions {
	async fn insert_games(&self, db: &DbMutex) -> Result;
}

const fn create_map_entry<TProvider: ProviderActions + ProviderStatic>()
-> (ProviderId, fn() -> Result<Provider>)
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
