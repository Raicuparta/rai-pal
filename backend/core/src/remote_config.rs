use std::{fs, io::Cursor, path::Path};

use rai_pal_proc_macros::serializable_struct;
use reqwest::Response;
use zip::ZipArchive;

use crate::{game::DbGame, paths, result::Result};

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

fn get_config_url(game_name: &str) -> String {
	format!("{CONFIG_DB_BASE_URL}/{CONFIG_DB_VERSION}/{game_name}/configs.json")
}

fn get_game_config_name(exe_path: &Path) -> Result<String> {
	paths::file_name_without_extension(exe_path).map(|name| name.to_lowercase())
}

pub async fn get_remote_configs(exe_path: &Path) -> Result<Option<RemoteConfigs>> {
	let url = get_config_url(&get_game_config_name(exe_path)?);
	let response = reqwest::get(&url).await?;

	if !response.status().is_success() {
		return Ok(None);
	}

	let configs = response.json::<RemoteConfigs>().await?;
	Ok(Some(configs))
}

async fn download_config(config_file: &str, game: &DbGame) -> Result<Response> {
	let game_name = get_game_config_name(game.try_get_exe_path()?)?;

	let url = format!("{CONFIG_DB_BASE_URL}/{CONFIG_DB_VERSION}/{game_name}/configs/{config_file}");
	let response = reqwest::get(&url).await?;

	Ok(response.error_for_status()?)
}

pub async fn download_config_file(
	config_file: &str,
	game: &DbGame,
	destination_path: &Path,
) -> Result {
	let content = download_config(config_file, game).await?.bytes().await?;

	fs::write(destination_path, content)?;
	Ok(())
}

pub async fn download_config_folder(
	config_file: &str,
	game: &DbGame,
	destination_path: &Path,
) -> Result {
	let content = download_config(config_file, game).await?.bytes().await?;

	ZipArchive::new(Cursor::new(content))?.extract(destination_path)?;

	Ok(())
}
