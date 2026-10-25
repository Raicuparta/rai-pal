use std::{io::Cursor, path::Path};

use rai_pal_proc_macros::serializable_struct;
use reqwest::Response;
use zip::ZipArchive;

use crate::{
	game::DbGame,
	http,
	mods::game_mod::GameMod,
	path_extensions::PathExt,
	result::{ContextResult, Result},
};

const CONFIG_DB_BASE_URL: &str = "https://raicuparta.github.io/rai-pal-db/config-db";
const CONFIG_DB_VERSION: u32 = 0;

#[serializable_struct]
pub struct RemoteConfig {
	pub version: u32,
	pub mod_id: String,
	pub file: String,
}

#[serializable_struct]
pub struct RemoteConfigs {
	pub configs: Vec<RemoteConfig>,
}

impl RemoteConfigs {
	pub fn get_for_mod(&self, game_mod: &GameMod) -> Option<&RemoteConfig> {
		let mod_id = game_mod.get_config_db_id();

		self.configs.iter().find(|config| config.mod_id == mod_id)
	}
}

fn get_config_url(game_name: &str) -> String {
	format!("{CONFIG_DB_BASE_URL}/{CONFIG_DB_VERSION}/{game_name}/configs.json")
}

fn get_game_config_name(exe_path: &Path) -> Result<String> {
	exe_path
		.file_name_without_extension()
		.map(str::to_lowercase)
}

pub async fn get_remote_configs(exe_path: &Path) -> Result<Option<RemoteConfigs>> {
	let url = get_config_url(&get_game_config_name(exe_path)?);
	let response = http::CLIENT.get(&url).send().await?;

	if !response.status().is_success() {
		return Ok(None);
	}

	let configs = response
		.json::<RemoteConfigs>()
		.await
		.with_context(|| format!("failed to decode remote configs from `{url}`"))?;
	Ok(Some(configs))
}

async fn download_config(config_file: &str, game: &DbGame) -> Result<Response> {
	let game_name = get_game_config_name(game.try_get_exe_path()?)?;

	let url = format!("{CONFIG_DB_BASE_URL}/{CONFIG_DB_VERSION}/{game_name}/configs/{config_file}");
	let response = http::CLIENT.get(&url).send().await?;

	Ok(response.error_for_status()?)
}

pub async fn download_config_file(
	config_file: &str,
	game: &DbGame,
	destination_path: &Path,
) -> Result {
	let content = download_config(config_file, game)
		.await?
		.bytes()
		.await
		.with_context(|| format!("failed to download config file `{config_file}`"))?;

	fs_err::tokio::write(destination_path, content).await?;
	Ok(())
}

pub async fn download_config_folder(
	config_file: &str,
	game: &DbGame,
	destination_path: &Path,
) -> Result {
	let content = download_config(config_file, game)
		.await?
		.bytes()
		.await
		.with_context(|| format!("failed to download config file `{config_file}`"))?;

	ZipArchive::new(Cursor::new(content))?.extract(destination_path)?;

	Ok(())
}
