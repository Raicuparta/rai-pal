use std::{
	collections::{
		BTreeMap,
		HashMap,
	},
	path::{
		Path,
		PathBuf,
	},
};

use rai_pal_proc_macros::serializable_enum;

#[cfg(target_os = "linux")]
use crate::providers::heroic_gog_provider::HeroicGog;
#[cfg(target_os = "linux")]
use crate::providers::{
	dummy_provider::Dummy,
	heroic_epic_provider::HeroicEpic,
};
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
		Error,
		Result,
	},
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

pub trait ProviderActions {
	fn insert_games(&self, db: &DbMutex) -> Result;
}

pub trait WineProviderActions {
	fn set_wine_dll_overrides(&self, _game: &DbGame, _dll_overrides: &[String]) -> Result {
		Ok(())
	}

	fn get_wine_prefix_path(&self, game: &DbGame) -> Result<PathBuf> {
		Err(Error::UnsupportedProviderOperation(
			game.provider_id,
			"get_wine_prefix_path".to_string(),
		))
	}

	fn get_wine_binary_path(&self, game: &DbGame) -> Result<PathBuf> {
		Err(Error::UnsupportedProviderOperation(
			game.provider_id,
			"get_wine_binary_path".to_string(),
		))
	}

	fn run_with_wine(
		&self,
		game: &DbGame,
		_exe_path: &Path,
		_args: &[String],
		_wine_env: &BTreeMap<String, String>,
	) -> Result {
		Err(Error::UnsupportedProviderOperation(
			game.provider_id,
			"run_with_wine".to_string(),
		))
	}
}

pub trait Provider: ProviderActions + WineProviderActions {}
impl<T> Provider for T where T: ProviderActions + WineProviderActions {}

pub fn get_provider(provider_id: ProviderId) -> Result<Box<dyn Provider>> {
	match provider_id {
		ProviderId::Steam => Ok(Box::new(Steam {})),

		ProviderId::Manual => Ok(Box::new(Manual {})),

		ProviderId::Itch => Ok(Box::new(Itch {})),

		#[cfg(target_os = "linux")]
		ProviderId::Epic => Ok(Box::new(HeroicEpic {})),
		#[cfg(target_os = "windows")]
		ProviderId::Epic => Ok(Box::new(Epic {})),

		#[cfg(target_os = "linux")]
		ProviderId::Gog => Ok(Box::new(HeroicGog {})),
		#[cfg(target_os = "windows")]
		ProviderId::Gog => Ok(Box::new(Gog {})),

		#[cfg(target_os = "windows")]
		ProviderId::Xbox => Ok(Box::new(Xbox {})),
		#[cfg(target_os = "linux")]
		ProviderId::Xbox => Ok(Box::new(Dummy {})),
	}
}
