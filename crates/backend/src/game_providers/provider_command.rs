use std::{collections::BTreeMap, path::PathBuf};

use rai_pal_proc_macros::{serde_enum, serde_struct};
use serde::Deserialize;

use crate::{
	game::Game,
	game_providers::game_provider::GameProviderId,
	open_better::open_detached_better,
	operating_system::OperatingSystem,
	result::{Error, Result},
};

#[derive(serde::Serialize, serde::Deserialize, Clone, PartialEq, Eq, Hash, Debug)]
pub enum ProviderCommandKind {
	String(String),
	Path(PathBuf, Vec<String>),
	Delete,
}

#[serde_enum]
pub enum ProviderCommandAction {
	StartViaProvider,
	StartViaExe,
	Install,
	ShowInLibrary,
	ShowInStore,
	OpenInBrowser,
	Delete,
}

#[serde_struct]
#[derive(PartialEq, Eq, Hash)]
pub struct ProviderCommand {
	pub action: ProviderCommandAction,
	pub kind: ProviderCommandKind,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum ProviderCommandsRepr {
	List(Vec<ProviderCommand>),
	Map(BTreeMap<ProviderCommandAction, ProviderCommandKind>),
}

pub(crate) fn deserialize_provider_commands<'de, D>(
	deserializer: D,
) -> std::result::Result<Vec<ProviderCommand>, D::Error>
where
	D: serde::Deserializer<'de>,
{
	let mut commands = match ProviderCommandsRepr::deserialize(deserializer)? {
		ProviderCommandsRepr::List(commands) => commands,
		ProviderCommandsRepr::Map(commands) => commands
			.into_iter()
			.map(|(action, kind)| ProviderCommand { action, kind })
			.collect(),
	};

	commands.sort_by_key(|command| command.action);

	Ok(commands)
}

impl ProviderCommand {
	pub fn run(
		&self,
		game: &Game,
		environment: &BTreeMap<String, String>,
		environment_mod_ids: &[String],
	) -> Result {
		if !environment.is_empty() && !self.supports_environment(game) {
			return Err(Error::UnsupportedGameEnvironment(
				game.display_title.clone(),
				game.provider_id,
				environment_mod_ids.join("`, `"),
			));
		}

		match &self.kind {
			ProviderCommandKind::String(command) => {
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
			ProviderCommandKind::Path(path, args) => {
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
			ProviderCommandKind::Delete => {
				crate::game_providers::game_provider::get_provider(game.provider_id)?
					.delete_game(game)?;
			}
		}
		Ok(())
	}

	fn supports_environment(&self, game: &Game) -> bool {
		#[cfg(target_os = "linux")]
		if game.executable_os == Some(OperatingSystem::Linux) {
			return match &self.kind {
				ProviderCommandKind::Path(_, _) => true,
				ProviderCommandKind::String(_) => game.provider_id == GameProviderId::Steam,
				ProviderCommandKind::Delete => false,
			};
		}

		#[cfg(target_os = "windows")]
		let _ = game;

		false
	}
}
