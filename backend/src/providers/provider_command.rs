use std::{
	path::PathBuf,
	process::Command,
};

use crate::Result;

#[derive(serde::Serialize, serde::Deserialize, specta::Type, Clone, PartialEq, Eq, Hash, Debug)]
pub enum ProviderCommand {
	String(String),
	Path(PathBuf, Vec<String>),
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
		};
		Ok(())
	}
}
