use std::{
	fs,
	marker::{Send, Sync},
	path::PathBuf,
};

use async_trait::async_trait;
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
	async fn get_games<TInstalledCallback, TOwnedCallback>(
		&self,
		installed_callback: TInstalledCallback,
		owned_callback: TOwnedCallback,
	) -> Result
	where
		TInstalledCallback: Fn(InstalledGame) + Send + Sync,
		TOwnedCallback: Fn(OwnedGame) + Send + Sync;
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
	match provider_id {
		ProviderId::Steam => Ok(Steam::new()?.into()),
		ProviderId::Manual => Ok(Manual::new()?.into()),
		ProviderId::Itch => Ok(Itch::new()?.into()),
		#[cfg(target_os = "windows")]
		ProviderId::Epic => Ok(Epic::new()?.into()),
		#[cfg(target_os = "windows")]
		ProviderId::Gog => Ok(Gog::new()?.into()),
		#[cfg(target_os = "windows")]
		ProviderId::Xbox => Ok(Xbox::new()?.into()),
		_ => Err(Error::InvalidProviderId(provider_id.to_string())),
	}
}
