use std::{fs, path::Path};

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
	exe_path
		.file_stem()
		.and_then(|stem| stem.to_str())
		.map(|name| name.to_lowercase())
}

pub fn get_config_url(game_name: &str) -> String {
	format!("{CONFIG_DB_BASE_URL}/{CONFIG_DB_VERSION}/{game_name}/configs.json")
}

pub async fn get_remote_configs(exe_path: &Path) -> Result<Option<RemoteConfigs>> {
	let game_name = match get_game_name_from_exe(exe_path) {
		Some(name) => name,
		None => return Ok(None),
	};

	let url = get_config_url(&game_name);
	let response = reqwest::get(&url).await?;

	if !response.status().is_success() {
		return Ok(None);
	}

	let configs = response.json::<RemoteConfigs>().await?;
	Ok(Some(configs))
}

pub async fn download_config_file(
	config_file: &str,
	game: &DbGame,
	destination_path: &Path,
) -> Result {
	let game_name = match &game.exe_path {
		Some(exe_path) => match get_game_name_from_exe(&exe_path.0) {
			Some(name) => name,
			None => return Ok(()),
		},
		None => return Ok(()),
	};

	let url = format!("{CONFIG_DB_BASE_URL}/{CONFIG_DB_VERSION}/{game_name}/configs/{config_file}");
	let response = reqwest::get(&url).await?;

	if !response.status().is_success() {
		return Ok(());
	}

	let content = response.bytes().await?;

	fs::write(destination_path, content)?;
	Ok(())
}
