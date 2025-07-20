use std::path::Path;

use log::{debug, info, warn};
use rai_pal_proc_macros::serializable_struct;

use crate::{game::DbGame, result::Result};

// TODO remember to update this MORON
const CONFIG_DB_BASE_URL: &str = "https://config-db.rai-pal.pages.dev/config-db";
const CONFIG_DB_VERSION: u32 = 0;

#[serializable_struct]
pub struct RemoteConfig {
	pub version: u32,
	#[serde(rename = "mod")]
	pub mod_id: String,
	pub loader: String,
	pub file: String,
}

#[serializable_struct]
pub struct RemoteConfigs {
	pub configs: Vec<RemoteConfig>,
}

pub fn get_game_name_from_exe(exe_path: &Path) -> Option<String> {
	debug!("Extracting game name from exe path: {}", exe_path.display());

	let game_name = exe_path
		.file_stem()
		.and_then(|stem| stem.to_str())
		.map(|name| name.to_lowercase());

	match &game_name {
		Some(name) => {
			debug!("Extracted game name: '{}'", name);
		}
		None => {
			warn!(
				"Failed to extract game name from exe path: {}",
				exe_path.display()
			);
		}
	}

	game_name
}

pub fn get_config_url(game_name: &str) -> String {
	let url = format!("{CONFIG_DB_BASE_URL}/{CONFIG_DB_VERSION}/{game_name}/configs.json");
	debug!("Generated config URL for game '{}': {}", game_name, url);
	url
}

pub async fn get_remote_configs(exe_path: &Path) -> Result<Option<RemoteConfigs>> {
	info!(
		"Getting remote configs for exe path: {}",
		exe_path.display()
	);

	let game_name = match get_game_name_from_exe(exe_path) {
		Some(name) => name,
		None => {
			warn!(
				"Could not extract game name from exe path: {}",
				exe_path.display()
			);
			return Ok(None);
		}
	};

	let url = get_config_url(&game_name);
	info!("Requesting remote configs from URL: {}", url);

	let response = match reqwest::get(&url).await {
		Ok(response) => {
			debug!("Received response with status: {}", response.status());
			response
		}
		Err(err) => {
			warn!("Failed to request configs from URL '{}': {}", url, err);
			return Ok(None);
		}
	};

	if !response.status().is_success() {
		info!(
			"No configs available for game '{}' (status: {})",
			game_name,
			response.status()
		);
		return Ok(None);
	}

	let configs = response.json::<RemoteConfigs>().await?;
	info!(
		"Successfully downloaded {} remote configs for game '{}'",
		configs.configs.len(),
		game_name
	);
	Ok(Some(configs))
}

pub async fn download_config_file(
	config_file: &str,
	game: &DbGame,
	destination_path: &Path,
	overwrite: bool,
) -> Result {
	if destination_path.exists() {
		if overwrite {
			if destination_path.is_dir() {
				std::fs::remove_dir_all(destination_path)?;
			} else {
				std::fs::remove_file(destination_path)?;
			}
		} else {
			return Ok(());
		}
	}

	let game_name = match &game.exe_path {
		Some(exe_path) => {
			debug!("Game exe path: {}", exe_path.0.display());
			match get_game_name_from_exe(&exe_path.0) {
				Some(name) => name,
				None => {
					warn!(
						"Could not extract game name from exe path: {}",
						exe_path.0.display()
					);
					return Ok(());
				}
			}
		}
		None => {
			warn!("Game '{}' has no exe_path set", game.display_title);
			return Ok(());
		}
	};

	let url = format!("{CONFIG_DB_BASE_URL}/{CONFIG_DB_VERSION}/{game_name}/configs/{config_file}");
	info!("Requesting config file from URL: {}", url);

	let response = match reqwest::get(&url).await {
		Ok(response) => {
			debug!("Received response with status: {}", response.status());
			response
		}
		Err(err) => {
			warn!("Failed to request config file from URL '{}': {}", url, err);
			return Ok(());
		}
	};

	if !response.status().is_success() {
		info!(
			"Config file not available at URL '{}' (status: {})",
			url,
			response.status()
		);
		return Ok(());
	}

	let content = response.text().await?;

	if let Some(parent) = destination_path.parent() {
		std::fs::create_dir_all(parent)?;
	}

	std::fs::write(destination_path, content)?;

	info!(
		"Successfully downloaded and saved config file to: {}",
		destination_path.display()
	);

	Ok(())
}
