#![cfg(target_os = "linux")]

use fs_err as fs;
use fs_err::PathExt;
use std::path::PathBuf;

use chrono::DateTime;
use serde::de::DeserializeOwned;
use serde_json::Value;

use super::provider_command::ProviderCommand;
use crate::{app_paths, result::Result};

fn heroic_config_path(relative_path: &str) -> Result<std::path::PathBuf> {
	let dirs = app_paths::base_dirs()?;

	let native = dirs.config_dir().join("heroic");
	if native.exists() {
		return Ok(native.join(relative_path));
	}

	let flatpak = dirs
		.home_dir()
		.join(".var/app/com.heroicgameslauncher.hgl/config/heroic");
	Ok(flatpak.join(relative_path))
}

pub fn read_heroic_json<T>(relative_path: &str) -> Result<Option<T>>
where
	T: DeserializeOwned,
{
	let path = heroic_config_path(relative_path)?;
	if !path.fs_err_try_exists()? {
		return Ok(None);
	}

	let file_content = fs_err::read_to_string(path)?;
	Ok(Some(serde_json::from_str::<T>(file_content.as_str())?))
}

pub fn read_heroic_json_files<T>(relative_dir: &str) -> Result<Vec<T>>
where
	T: DeserializeOwned,
{
	let path = heroic_config_path(relative_dir)?;
	if !path.fs_err_try_exists()? {
		return Ok(Vec::new());
	}

	let mut values = Vec::new();
	for entry in fs::read_dir(path)? {
		let entry = entry?;
		if !entry.file_type()?.is_file() {
			continue;
		}

		let file_content = match fs::read_to_string(entry.path()) {
			Ok(file_content) => file_content,
			Err(error) => {
				log::warn!(
					"Failed to read Heroic cache file {}: {error}",
					entry.path().display()
				);
				continue;
			}
		};

		match serde_json::from_str::<T>(&file_content) {
			Ok(value) => values.push(value),
			Err(error) => {
				log::warn!(
					"Failed to parse Heroic cache file {}: {error}",
					entry.path().display()
				);
			}
		}
	}

	Ok(values)
}

pub fn parse_release_date(date: &str) -> Option<String> {
	DateTime::parse_from_rfc3339(date)
		.or_else(|_| DateTime::parse_from_str(date, "%Y-%m-%dT%H:%M:%S%z"))
		.ok()
		.map(|date_time| date_time.to_rfc3339())
}

pub fn get_wine_prefix_path(game_id: &str) -> Result<PathBuf> {
	let game_config_path = format!("GamesConfig/{game_id}.json");
	if let Some(prefix) = read_wine_prefix(&game_config_path, Some(game_id))? {
		return Ok(prefix);
	}

	if let Some(prefix) = read_wine_prefix("config.json", None)? {
		return Ok(prefix);
	}

	Ok(default_wine_prefix()?)
}

fn read_wine_prefix(relative_path: &str, game_id: Option<&str>) -> Result<Option<PathBuf>> {
	let Some(config) = read_heroic_json::<Value>(relative_path)? else {
		return Ok(None);
	};

	let prefix = match game_id {
		Some(game_id) => config
			.get(game_id)
			.and_then(|settings| settings.get("winePrefix")),
		None => config
			.get("defaultSettings")
			.and_then(|settings| settings.get("winePrefix")),
	};

	let Some(prefix) = prefix
		.and_then(Value::as_str)
		.filter(|prefix| !prefix.is_empty())
	else {
		return Ok(None);
	};

	Ok(Some(expand_home_dir(prefix)?))
}

fn expand_home_dir(path: &str) -> Result<PathBuf> {
	let path = PathBuf::from(path);
	if let Ok(relative_path) = path.strip_prefix("~") {
		return Ok(app_paths::base_dirs()?.home_dir().join(relative_path));
	}
	Ok(path)
}

fn default_wine_prefix() -> Result<PathBuf> {
	Ok(app_paths::base_dirs()?
		.home_dir()
		.join("Games")
		.join("Heroic")
		.join("Prefixes")
		.join("shared"))
}

pub fn launch_command(app_name: &str, store_prefix: Option<&str>) -> ProviderCommand {
	let launch_target = store_prefix.map_or_else(
		|| app_name.to_string(),
		|prefix| format!("{prefix}/{app_name}"),
	);

	ProviderCommand::String(format!("heroic://launch/{launch_target}"))
}
