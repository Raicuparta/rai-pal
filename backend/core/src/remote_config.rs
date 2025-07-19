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

/// Extracts the game name from an executable path for use in config URLs.
/// Converts "PEAK.exe" to "peak", removing the extension and converting to lowercase.
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

/// Constructs the remote config URL for a given game name.
/// Example: game_name "peak" -> "https://config-db.rai-pal.pages.dev/config-db/0/peak/configs.json"
pub fn get_config_url(game_name: &str) -> String {
	let url = format!("{CONFIG_DB_BASE_URL}/{CONFIG_DB_VERSION}/{game_name}/configs.json");
	debug!("Generated config URL for game '{}': {}", game_name, url);
	url
}

/// Downloads the remote configuration list for a game based on its executable name.
///
/// # Arguments
/// * `exe_path` - Path to the game's executable file
///
/// # Returns
/// * `Ok(Some(RemoteConfigList))` if configs were found and downloaded successfully
/// * `Ok(None)` if no configs are available for this game
/// * `Err(Error)` if there was an error during the download or parsing
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

/// Downloads a specific config file for a game.
///
/// # Arguments
/// * `config_file` - The name of the config file to download
/// * `game` - The game database entry containing the executable path
/// * `destination_path` - The path where the config file should be saved
///
/// # Returns
/// * `Ok(Some(()))` if the file was downloaded and saved successfully
/// * `Ok(None)` if the config file is not available
/// * `Err(Error)` if there was an error during the download or file writing
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

	// Extract game name from the executable path
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

	// Construct the config file name from mod loader and mod IDs
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

	// Create parent directories if they don't exist
	if let Some(parent) = destination_path.parent() {
		std::fs::create_dir_all(parent)?;
	}

	// Write the content to the destination file
	std::fs::write(destination_path, content)?;

	info!(
		"Successfully downloaded and saved config file to: {}",
		destination_path.display()
	);

	Ok(())
}

/// Downloads a specific config file using a RemoteConfig entry and game.
///
/// # Arguments
/// * `remote_config` - The remote config entry containing file info
/// * `game` - The game database entry containing the executable path
///
/// # Returns
/// * `Ok(Some(String))` with the config file content if found
/// * `Ok(None)` if the config file is not available
/// * `Err(Error)` if there was an error during the download
pub async fn download_config_from_entry(
	remote_config: &RemoteConfig,
	game: &DbGame,
) -> Result<Option<String>> {
	info!(
		"Downloading config from entry - mod_id: '{}', loader: '{}', file: '{}', game: '{}'",
		remote_config.mod_id, remote_config.loader, remote_config.file, game.display_title
	);

	// Extract game name from the executable path
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
					return Ok(None);
				}
			}
		}
		None => {
			warn!("Game '{}' has no exe_path set", game.display_title);
			return Ok(None);
		}
	};

	let url = format!(
		"{CONFIG_DB_BASE_URL}/{CONFIG_DB_VERSION}/{game_name}/configs/{}",
		remote_config.file
	);
	info!("Requesting config file from URL: {}", url);

	let response = match reqwest::get(&url).await {
		Ok(response) => {
			debug!("Received response with status: {}", response.status());
			response
		}
		Err(err) => {
			warn!("Failed to request config file from URL '{}': {}", url, err);
			return Ok(None);
		}
	};

	if !response.status().is_success() {
		info!(
			"Config file not available at URL '{}' (status: {})",
			url,
			response.status()
		);
		return Ok(None);
	}

	let content = response.text().await?;
	info!(
		"Successfully downloaded config file ({} bytes) for mod '{}' in game '{}'",
		content.len(),
		remote_config.mod_id,
		game_name
	);
	Ok(Some(content))
}
