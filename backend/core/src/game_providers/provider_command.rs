use std::path::PathBuf;

use rai_pal_proc_macros::serializable_enum;

use crate::{
	game::DbGame,
	open_better::open_detached_better,
	result::Result,
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
	pub fn run(&self, game: &DbGame) -> Result {
		match self {
			Self::String(command) => {
				open_detached_better(command)?;
			}
			Self::Path(path, args) => {
				#[cfg(target_os = "linux")]
				{
					use std::collections::BTreeMap;

					use crate::game_providers::game_provider;

					game_provider::get_provider(game.provider_id)?.run_with_wine(
						game,
						path,
						args,
						&BTreeMap::default(),
					)?;
				}

				#[cfg(target_os = "windows")]
				{
					use std::process::Command;

					let mut command = Command::new(path);
					command.args(args);
					if let Some(parent) = path.parent() {
						command.current_dir(parent);
					}
					command.spawn()?;
				}
			}
		}
		Ok(())
	}
}
