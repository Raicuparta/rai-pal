use std::{collections::BTreeMap, path::PathBuf};

use rai_pal_proc_macros::serializable_enum;

use crate::{
	game::DbGame,
	game_providers::game_provider::GameProviderId,
	open_better::open_detached_better,
	operating_system::OperatingSystem,
	result::{Error, Result},
};

#[derive(serde::Serialize, serde::Deserialize, specta::Type, Clone, PartialEq, Eq, Hash, Debug)]
pub enum ProviderCommand {
	String(String),
	Path(PathBuf, Vec<String>),
}

#[serializable_enum]
pub enum ProviderCommandAction {
	Install,
	ShowInLibrary,
	ShowInStore,
	StartViaProvider,
	StartViaExe,
	OpenInBrowser,
}

impl ProviderCommand {
	pub fn run(&self, game: &DbGame, environment: &BTreeMap<String, String>) -> Result {
		if !environment.is_empty() && !self.supports_environment(game) {
			return Err(Error::UnsupportedGameEnvironment(
				game.display_title.clone(),
				game.provider_id,
			));
		}

		match self {
			Self::String(command) => {
				#[cfg(target_os = "linux")]
				if !environment.is_empty() {
					let exe_path = game.try_get_exe_path()?;
					let cwd = exe_path.parent().map(PathBuf::from);
					crate::game_providers::steam::steam_exe_swap::launch_via_steam_with_swapped_exe(
						game,
						exe_path,
						&[],
						environment,
						cwd.as_deref(),
					)?;

					return Ok(());
				}

				open_detached_better(command)?;
			}
			Self::Path(path, args) => {
				#[cfg(target_os = "linux")]
				{
					use crate::{
						game_launch::{GameLaunch, spawn_game},
						game_providers::game_provider,
					};

					if game.executable_os == Some(OperatingSystem::Linux) {
						let mut launch = GameLaunch::new(path);
						launch.args(args);
						launch.envs_expanded(environment);
						if let Some(parent) = path.parent() {
							launch.cwd = Some(parent.to_path_buf());
						}
						spawn_game(&launch)?;
					} else {
						game_provider::get_provider(game.provider_id)?.run_with_wine(
							game,
							path,
							args,
							&BTreeMap::default(),
						)?;
					}
				}

				#[cfg(target_os = "windows")]
				{
					use std::process::Command;

					use crate::open_better::spawn_detached;

					let mut command = Command::new(path);
					command.args(args);
					if let Some(parent) = path.parent() {
						command.current_dir(parent);
					}
					spawn_detached(&mut command)?;
				}
			}
		}
		Ok(())
	}

	fn supports_environment(&self, game: &DbGame) -> bool {
		#[cfg(target_os = "linux")]
		if game.executable_os == Some(OperatingSystem::Linux) {
			return match self {
				Self::Path(_, _) => true,
				Self::String(_) => game.provider_id == GameProviderId::Steam,
			};
		}

		#[cfg(target_os = "windows")]
		let _ = game;

		false
	}
}
