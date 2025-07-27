use std::{path::PathBuf, process::Command};

use rai_pal_proc_macros::serializable_enum;

use crate::result::Result;

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
	pub fn run(&self) -> Result {
		match self {
			Self::String(command) => {
				open::that_detached(command)?;
			}
			Self::Path(path, args) => {
				let mut command = Command::new(path);
				command.args(args);
				if let Some(parent) = path.parent() {
					command.current_dir(parent);
				}
				command.spawn()?;
			}
		}
		Ok(())
	}
}
