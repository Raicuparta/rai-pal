#![cfg(target_os = "linux")]

use serde::de::DeserializeOwned;

use super::provider_command::ProviderCommand;
use crate::{
	paths,
	result::Result,
};

fn heroic_config_path(relative_path: &str) -> Result<std::path::PathBuf> {
	let dirs = paths::base_dirs()?;
	Ok(dirs.config_dir().join("heroic").join(relative_path))
}

pub fn read_heroic_json<T>(relative_path: &str) -> Result<T>
where
	T: DeserializeOwned,
{
	let path = heroic_config_path(relative_path)?;
	let file_content = std::fs::read_to_string(path)?;
	Ok(serde_json::from_str::<T>(file_content.as_str())?)
}

pub fn read_heroic_json_if_exists<T>(relative_path: &str) -> Result<Option<T>>
where
	T: DeserializeOwned,
{
	let path = heroic_config_path(relative_path)?;
	if !path.try_exists()? {
		return Ok(None);
	}

	let file_content = std::fs::read_to_string(path)?;
	Ok(Some(serde_json::from_str::<T>(file_content.as_str())?))
}

pub fn launch_command(app_name: &str, store_prefix: Option<&str>) -> ProviderCommand {
	let launch_target = store_prefix.map_or_else(
		|| app_name.to_string(),
		|prefix| format!("{prefix}/{app_name}"),
	);

	ProviderCommand::String(format!("heroic://launch/{launch_target}"))
}
