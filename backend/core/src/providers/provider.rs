use std::{
	collections::HashMap,
	path::{
		Path,
		PathBuf,
	},
};

use anyhow::bail;
use enum_dispatch::enum_dispatch;
use rai_pal_proc_macros::serializable_enum;

#[cfg(target_os = "linux")]
use crate::providers::heroic_epic_provider::HeroicEpic;
#[cfg(target_os = "linux")]
use crate::providers::heroic_gog_provider::HeroicGog;
#[cfg(target_os = "windows")]
use crate::providers::{
	epic_provider::Epic,
	gog_provider::Gog,
	xbox_provider::Xbox,
};
use crate::{
	game::DbGame,
	local_database::DbMutex,
	providers::{
		itch_provider::Itch,
		manual_provider::Manual,
		steam::steam_provider::Steam,
	},
	result::{
		CoreError,
		CoreResult,
	},
};

// These IDs need to match the ones in rai-pal-db.
#[serializable_enum]
pub enum ProviderId {
	Itch,
	Manual,
	Steam,
	Epic,
	Gog,
	#[cfg(target_os = "windows")]
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

type Map = [(ProviderId, fn() -> CoreResult<Provider>)];
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
	async fn insert_games(&self, db: &DbMutex) -> CoreResult;
	fn set_wine_dll_overrides(&self, _game: &DbGame, _dll_overrides: &[String]) -> CoreResult {
		Ok(())
	}
	fn get_wine_prefix_path(&self, _game: &DbGame) -> CoreResult<PathBuf> {
		bail!(CoreError::UnsupportedOperation(
			"get_wine_prefix_path".to_string(),
		))
	}
	fn get_wine_binary_path(&self, _game: &DbGame) -> CoreResult<PathBuf> {
		bail!(CoreError::UnsupportedOperation(
			"get_wine_binary_folder".to_string(),
		))
	}
	fn run_with_wine(
		&self,
		_game: &DbGame,
		_exe_path: &Path,
		_args: &[String],
		_wine_env: &HashMap<String, String>,
	) -> CoreResult {
		Ok(())
	}
}

const fn create_map_entry<TProvider: ProviderActions + ProviderStatic>()
-> (ProviderId, fn() -> CoreResult<Provider>)
where
	Provider: From<TProvider>,
{
	(*TProvider::ID, || Ok(TProvider::new()?.into()))
}

pub trait ProviderStatic: ProviderActions {
	const ID: &'static ProviderId;

	fn new() -> CoreResult<Self>
	where
		Self: Sized;
}

pub fn get_provider(provider_id: ProviderId) -> CoreResult<Provider> {
	for &(id, create_provider) in PROVIDERS {
		if id == provider_id {
			return create_provider();
		}
	}
	bail!(
		"Failed to find provider with ID `{provider_id}`. It's probably not supported in this platform."
	);
}
