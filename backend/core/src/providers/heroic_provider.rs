#![cfg(target_os = "linux")]

use std::{
	collections::HashMap,
	fs,
	path::PathBuf,
};

use serde::de::DeserializeOwned;
use serde_json::{
	Map,
	Value,
};

use super::provider_command::ProviderCommand;
use crate::{
	paths,
	result::Result,
};

fn heroic_config_path(relative_path: &str) -> Result<std::path::PathBuf> {
	let dirs = paths::base_dirs()?;
	Ok(dirs.config_dir().join("heroic").join(relative_path))
}

fn games_config_path(game_id: &str) -> Result<PathBuf> {
	heroic_config_path(&format!("GamesConfig/{game_id}.json"))
}

fn ensure_object(value: &mut Value) -> &mut Map<String, Value> {
	if !value.is_object() {
		*value = Value::Object(Map::default());
	}

	value
		.as_object_mut()
		.expect("value was converted to a JSON object")
}

fn ensure_array(value: &mut Value) -> &mut Vec<Value> {
	if !value.is_array() {
		*value = Value::Array(Vec::default());
	}

	value
		.as_array_mut()
		.expect("value was converted to a JSON array")
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

pub fn set_environment(game_id: &str, environment: &HashMap<String, String>) -> Result {
	let path = games_config_path(game_id)?;
	let mut config = if path.try_exists()? {
		serde_json::from_str::<Value>(&fs::read_to_string(&path)?)?
	} else {
		Value::Object(Map::default())
	};

	let config_object = ensure_object(&mut config);
	let game_config = config_object
		.entry(game_id.to_string())
		.or_insert_with(|| Value::Object(Map::default()));
	let environment_options = ensure_array(
		ensure_object(game_config)
			.entry("enviromentOptions".to_string())
			.or_insert_with(|| Value::Array(Vec::default())),
	);

	for (key, value) in environment {
		if let Some(existing_entry) = environment_options
			.iter_mut()
			.find(|entry| entry.get("key").and_then(Value::as_str) == Some(key.as_str()))
		{
			*existing_entry = serde_json::json!({ "key": key, "value": value });
		} else {
			environment_options.push(serde_json::json!({ "key": key, "value": value }));
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
