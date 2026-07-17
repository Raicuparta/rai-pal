use std::{
	collections::BTreeMap,
	path::{
		Path,
		PathBuf,
	},
	process::Command,
	time::{
		SystemTime,
		UNIX_EPOCH,
	},
};

use rai_pal_proc_macros::serializable_enum;

#[cfg(target_os = "linux")]
use crate::game_providers::heroic_gog_provider::HeroicGog;
#[cfg(target_os = "linux")]
use crate::game_providers::{
	dummy_provider::Dummy,
	heroic_epic_provider::HeroicEpic,
};
#[cfg(target_os = "windows")]
use crate::game_providers::{
	epic_provider::Epic,
	gog_provider::Gog,
	xbox_provider::Xbox,
};
use crate::{
	game::DbGame,
	game_engines::pe_utils,
	game_providers::{
		itch_provider::Itch,
		manual_provider::Manual,
		steam::steam_provider::Steam,
	},
	local_database::{
		app_database::DbMutex,
		game_database::GameDatabase,
	},
	result::{
		Error,
		Result,
	},
};

// These IDs need to match the ones in rai-pal-db.
#[serializable_enum]
pub enum GameProviderId {
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

	fn get_run_with_wine_command(&self, game: &DbGame) -> Result<Command> {
		Err(Error::UnsupportedProviderOperation(
			game.provider_id,
			"get_run_with_wine_command".to_string(),
		))
	}

	fn run_with_wine(
		&self,
		game: &DbGame,
		exe_path: &Path,
		args: &[String],
		wine_env: &BTreeMap<String, String>,
	) -> Result {
		let mut cmd = self.get_run_with_wine_command(game)?;

		if pe_utils::is_pe_console_app(exe_path) {
			cmd.arg("wineconsole");
		}

		let child = cmd.arg(exe_path).args(args).envs(wine_env).spawn()?;

		log::info!(
			"Launched `{}` with Wine for game `{}` ({}) (pid {})",
			exe_path.display(),
			game.display_title,
			game.external_id,
			child.id(),
		);

		Ok(())
	}
}

pub trait GameProvider: ProviderActions + WineProviderActions {}
impl<T> GameProvider for T where T: ProviderActions + WineProviderActions {}

pub fn get_provider(provider_id: GameProviderId) -> Result<Box<dyn GameProvider>> {
	match provider_id {
		GameProviderId::Steam => Ok(Box::new(Steam {})),

		GameProviderId::Manual => Ok(Box::new(Manual {})),

		GameProviderId::Itch => Ok(Box::new(Itch {})),

		#[cfg(target_os = "linux")]
		GameProviderId::Epic => Ok(Box::new(HeroicEpic {})),
		#[cfg(target_os = "windows")]
		GameProviderId::Epic => Ok(Box::new(Epic {})),

		#[cfg(target_os = "linux")]
		GameProviderId::Gog => Ok(Box::new(HeroicGog {})),
		#[cfg(target_os = "windows")]
		GameProviderId::Gog => Ok(Box::new(Gog {})),

		#[cfg(target_os = "windows")]
		GameProviderId::Xbox => Ok(Box::new(Xbox {})),
		#[cfg(target_os = "linux")]
		GameProviderId::Xbox => Ok(Box::new(Dummy {})),
	}
}

impl GameProviderId {
	pub fn insert_games(&self, db: &DbMutex) -> Result {
		let start_time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

		get_provider(*self)?.insert_games(db)?;

		db.remove_stale_games(self, start_time)?;

		Ok(())
	}
}
