use std::{
	fs,
	path::{
		Path,
		PathBuf,
	},
};

use log::error;
use rai_pal_proc_macros::serializable_struct;

use super::game_provider::{
	GameProviderId,
	ProviderActions,
};
use crate::{
	app_paths,
	game::DbGame,
	game_providers::game_provider::WineProviderActions,
	local_database::{
		app_database::DbMutex,
		game_database::GameDatabase,
	},
	path_extensions::PathExt,
	result::{
		Error,
		Result,
	},
};

#[serializable_struct]
pub struct Manual {}

#[derive(serde::Serialize, serde::Deserialize)]
struct GamesConfig {
	pub paths: Vec<PathBuf>,
	#[serde(default)]
	pub directories: Vec<PathBuf>,
}

impl ProviderActions for Manual {
	fn insert_games(&self, db: &DbMutex) -> Result {
		let config = read_games_config(&games_config_path()?);

		for path in &config.paths {
			match get_game_from_path(path) {
				Ok(game) => {
					db.insert_game(&game);
				}
				Err(error) => {
					error!(
						"Failed to get game from path '{}'. Will remove this path from the config. Error: {}",
						path.display(),
						error
					);
					remove_path(path)?;
				}
			}
		}

		for dir in &config.directories {
			if !dir.is_dir() {
				error!(
					"Directory '{}' not found. Will remove it from the config.",
					dir.display()
				);
				remove_directory_path(dir)?;
				continue;
			}

			match find_executables_in_directory(dir) {
				Ok(executables) => {
					for exe_path in executables {
						match get_game_from_path(&exe_path) {
							Ok(game) => {
								db.insert_game(&game);
							}
							Err(error) => {
								error!(
									"Failed to get game from path '{}': {}",
									exe_path.display(),
									error
								);
							}
						}
					}
				}
				Err(error) => {
					error!("Failed to walk directory '{}': {}", dir.display(), error);
				}
			}
		}

		Ok(())
	}
}

impl WineProviderActions for Manual {}

fn games_config_path() -> Result<PathBuf> {
	app_paths::app_data_file("games.json")
}

fn read_games_config(games_config_path: &Path) -> GamesConfig {
	match fs::read_to_string(games_config_path)
		.and_then(|games_config_file| Ok(serde_json::from_str::<GamesConfig>(&games_config_file)?))
	{
		Ok(games_config) => games_config,
		Err(error) => {
			error!("Error reading config: {error}");
			GamesConfig {
				paths: Vec::default(),
				directories: Vec::default(),
			}
		}
	}
}

fn get_game_from_path(exe_path: &Path) -> Result<DbGame> {
	let mut game = DbGame::new(
		GameProviderId::Manual,
		exe_path.hash_string(),
		exe_path.file_name_without_extension()?.to_string(),
	);
	game.set_executable(exe_path);
	Ok(game)
}

pub fn add_game(path: &Path) -> Result<DbGame> {
	let game = get_game_from_path(path)?;

	if game.exe_path.is_none() {
		return Err(Error::NoExecutableFound(path.to_owned()));
	}

	let config_path = games_config_path()?;

	let mut games_config = read_games_config(&config_path);
	games_config.paths.push(path.to_path_buf());

	fs::write(config_path, serde_json::to_string_pretty(&games_config)?)?;

	Ok(game)
}

pub fn add_directory(path: &Path) -> Result<Vec<DbGame>> {
	if !path.is_dir() {
		return Err(Error::NoExecutableFound(path.to_owned()));
	}

	let executables = find_executables_in_directory(path)?;

	let games: Result<Vec<DbGame>> = executables
		.iter()
		.map(|exe_path| get_game_from_path(exe_path))
		.collect();
	let games = games?;

	let config_path = games_config_path()?;
	let mut games_config = read_games_config(&config_path);
	games_config.directories.push(path.to_path_buf());

	fs::write(config_path, serde_json::to_string_pretty(&games_config)?)?;

	Ok(games)
}

fn find_executables_in_directory(dir: &Path) -> Result<Vec<PathBuf>> {
	const VALID_EXTENSIONS: [&str; 3] = ["exe", "x86_64", "x86"];
	let mut executables = Vec::new();

	walk_directory(dir, &mut executables, &VALID_EXTENSIONS)?;

	Ok(executables)
}

fn walk_directory(dir: &Path, executables: &mut Vec<PathBuf>, valid_extensions: &[&str]) -> Result {
	for entry in fs::read_dir(dir)? {
		let entry = entry?;
		let path = entry.path();

		if path.is_dir() {
			walk_directory(&path, executables, valid_extensions)?;
		} else if path.is_file()
			&& let Some(ext) = path.extension().and_then(|e| e.to_str())
			&& valid_extensions.contains(&ext.to_lowercase().as_str())
		{
			executables.push(path);
		}
	}

	Ok(())
}

fn remove_directory_path(path: &Path) -> Result {
	let config_path = games_config_path()?;
	let mut games_config = read_games_config(&config_path);
	games_config.directories.retain(|p| p != path);

	fs::write(config_path, serde_json::to_string_pretty(&games_config)?)?;

	Ok(())
}

pub fn remove_game(game: &DbGame) -> Result {
	if game.provider_id != GameProviderId::Manual {
		return Err(Error::InvalidProviderId(game.provider_id.to_string()));
	}

	let path = &game
		.exe_path
		.as_ref()
		.ok_or_else(|| Error::GameNotInstalled(game.display_title.clone()))?;

	remove_path(path)?;

	Ok(())
}

fn remove_path(path: &Path) -> Result {
	let config_path = games_config_path()?;
	let mut games_config = read_games_config(&config_path);
	games_config.paths.retain(|p| p != path);

	fs::write(config_path, serde_json::to_string_pretty(&games_config)?)?;

	Ok(())
}
