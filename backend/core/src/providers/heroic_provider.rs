#![cfg(target_os = "linux")]

use std::{
	collections::HashMap,
	fs,
	path::PathBuf,
};

use serde::{
	Deserialize,
	Serialize,
	de::DeserializeOwned,
};
use serde_json::Value;

use super::provider_command::ProviderCommand;
use crate::{
	paths,
	result::Result,
};

fn heroic_config_path(relative_path: &str) -> Result<std::path::PathBuf> {
	let dirs = paths::base_dirs()?;

	let native = dirs.config_dir().join("heroic");
	if native.exists() {
		return Ok(native.join(relative_path));
	}

	let flatpak = dirs
		.home_dir()
		.join(".var/app/com.heroicgameslauncher.hgl/config/heroic");
	Ok(flatpak.join(relative_path))
}

fn games_config_path(game_id: &str) -> Result<PathBuf> {
	heroic_config_path(&format!("GamesConfig/{game_id}.json"))
}

#[derive(Default, Deserialize, Serialize)]
#[serde(transparent)]
struct HeroicGamesConfig {
	games: HashMap<String, HeroicGameConfig>,
}

#[derive(Default, Deserialize, Serialize)]
struct HeroicGameConfig {
	#[serde(rename = "enviromentOptions", default)]
	environment_options: Vec<HeroicEnvironmentOption>,
	#[serde(flatten)]
	extra: HashMap<String, Value>,
}

#[derive(Deserialize, Serialize)]
struct HeroicEnvironmentOption {
	key: String,
	value: String,
	#[serde(flatten)]
	extra: HashMap<String, Value>,
}

pub fn read_heroic_json<T>(relative_path: &str) -> Result<Option<T>>
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

pub fn set_wine_dll_overrides(game_id: &str, dll_overrides: &[String]) -> Result {
	if dll_overrides.is_empty() {
		return Ok(());
	}

	let wine_dll_overrides_value = dll_overrides
		.iter()
		.map(|dll| format!("{dll}=n,b"))
		.collect::<Vec<_>>()
		.join(";");

	let environment = HashMap::from([("WINEDLLOVERRIDES".to_string(), wine_dll_overrides_value)]);

	let relative_path = format!("GamesConfig/{game_id}.json");
	let path = games_config_path(game_id)?;
	let mut config = read_heroic_json::<HeroicGamesConfig>(&relative_path)?.unwrap_or_default();

	let game_config = config.games.entry(game_id.to_string()).or_default();

	for (key, value) in environment {
		if let Some(existing_entry) = game_config
			.environment_options
			.iter_mut()
			.find(|entry| entry.key == *key)
		{
			existing_entry.value = value;
		} else {
			game_config
				.environment_options
				.push(HeroicEnvironmentOption {
					key,
					value,
					extra: HashMap::default(),
				});
		}
	}

	if let Some(parent) = path.parent() {
		fs::create_dir_all(parent)?;
	}
	fs::write(path, serde_json::to_string_pretty(&config)?)?;

	Ok(())
}

pub fn launch_command(app_name: &str, store_prefix: Option<&str>) -> ProviderCommand {
	let launch_target = store_prefix.map_or_else(
		|| app_name.to_string(),
		|prefix| format!("{prefix}/{app_name}"),
	);

	ProviderCommand::String(format!("heroic://launch/{launch_target}"))
}
