use std::{
	fs,
	marker::{Send, Sync},
	path::PathBuf,
};

use enum_dispatch::enum_dispatch;

#[cfg(target_os = "windows")]
use crate::providers::{epic_provider::Epic, gog_provider::Gog, xbox_provider::Xbox};
use crate::{
	installed_game::InstalledGame,
	owned_game::OwnedGame,
	paths,
	providers::{itch_provider::Itch, manual_provider::Manual, steam_provider::Steam},
	result::{Error, Result},
	serializable_enum,
};

use super::ubisoft_provider::Ubisoft;

serializable_enum!(ProviderId {
	Steam,
	Manual,
	Itch,
	Epic,
	Gog,
	Xbox,
	Ubisoft,
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
	Ubisoft,
}

type Map = [(ProviderId, fn() -> Result<Provider>)];
const PROVIDERS: &Map = &[
	create_map_entry::<Steam>(),
	create_map_entry::<Manual>(),
	create_map_entry::<Itch>(),
	#[cfg(target_os = "windows")]
	create_map_entry::<Epic>(),
	#[cfg(target_os = "windows")]
	create_map_entry::<Gog>(),
	#[cfg(target_os = "windows")]
	create_map_entry::<Xbox>(),
	create_map_entry::<Ubisoft>(),
];

#[enum_dispatch(Provider)]
pub trait ProviderActions {
	async fn get_games<TInstalledCallback, TOwnedCallback>(
		&self,
		installed_callback: TInstalledCallback,
		owned_callback: TOwnedCallback,
	) -> Result
	where
		TInstalledCallback: FnMut(InstalledGame) + Send + Sync,
		TOwnedCallback: FnMut(OwnedGame) + Send + Sync;
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
