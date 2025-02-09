use std::{fs, path::PathBuf};

use crate::game;
use crate::result::Result;
use crate::{paths, providers::provider::ProviderId};

const CACHE_FOLDER: &str = "cache";

fn get_folder_path() -> Result<PathBuf> {
	Ok(paths::app_data_path()?.join(CACHE_FOLDER).join("providers"))
}

fn get_provider_path(provider_id: ProviderId) -> Result<PathBuf> {
	Ok(get_folder_path()?.join(provider_id.to_string()))
}

fn try_read(provider_id: ProviderId) -> Result<Option<game::Map>> {
	let path = get_provider_path(provider_id)?;
	if !path.is_file() {
		return Ok(None);
	}

	let data = serde_json::from_str(&fs::read_to_string(&path)?)?;

	Ok(Some(data))
}

fn try_write(provider_id: ProviderId, games: &game::Map) -> Result {
	let path = get_provider_path(provider_id)?;

	if path.is_dir() {
		fs::remove_dir_all(&path)?;
	}

	fs::create_dir_all(paths::path_parent(&path)?)?;

	fs::write(&path, serde_json::to_string(games)?)?;

	Ok(())
}

pub fn read(provider_id: ProviderId) -> Option<game::Map> {
	try_read(provider_id).unwrap_or_else(|err| {
		log::error!("Failed to read provider cache for provider {provider_id}: {err}");
		None
	})
}

pub fn write(provider_id: ProviderId, games: &game::Map) {
	if let Err(err) = try_write(provider_id, games) {
		log::error!("Failed to write provider cache for provider {provider_id}: {err}");
	}
}

pub fn clear() -> Result {
	fs::remove_dir_all(get_folder_path()?)?;

	Ok(())
}
