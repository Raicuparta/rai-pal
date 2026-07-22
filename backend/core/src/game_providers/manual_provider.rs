use std::{
	fs,
	path::{
		Path,
		PathBuf,
	},
	time::Instant,
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

const VALID_EXTENSIONS: [&str; 3] = ["exe", "x86_64", "x86"];

#[serializable_struct]
pub struct Manual {}

#[derive(serde::Serialize, serde::Deserialize)]
struct GamesConfig {
	pub paths: Vec<PathBuf>,
	pub directories: Vec<PathBuf>,
	pub ignored_paths: Vec<PathBuf>,
}

#[serializable_struct]
pub struct ScanProgress {
	pub scanned_dirs: u32,
	pub executables_found: u32,
	pub current_path: String,
}

#[serializable_struct]
pub struct DirectoryScanResult {
	pub games: Vec<DbGame>,
	pub duration_secs: f64,
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
						if config.ignored_paths.contains(&exe_path) {
							continue;
						}

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

		clean_up_stale_ignored_paths(&config)?;

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
				ignored_paths: Vec::default(),
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

	if games_config.directories.iter().any(|p| p.as_path() == path) {
		return Ok(games);
	}

	games_config.directories.push(path.to_path_buf());

	fs::write(config_path, serde_json::to_string_pretty(&games_config)?)?;

	Ok(games)
}

fn find_executables_in_directory(dir: &Path) -> Result<Vec<PathBuf>> {
	let mut executables = Vec::new();
	let mut scanned_dirs = 0u32;

	walk_directory(dir, &mut executables, &VALID_EXTENSIONS, None, &mut scanned_dirs)?;

	Ok(executables)
}

fn walk_directory(
	dir: &Path,
	executables: &mut Vec<PathBuf>,
	valid_extensions: &[&str],
	on_progress: Option<&dyn Fn(ScanProgress)>,
	scanned_dirs: &mut u32,
) -> Result {
	for entry in fs::read_dir(dir)? {
		let entry = entry?;
		let path = entry.path();

		if path.is_dir() {
			*scanned_dirs += 1;
			if let Some(cb) = on_progress {
				cb(ScanProgress {
					scanned_dirs: *scanned_dirs,
					executables_found: executables.len() as u32,
					current_path: path.display().to_string(),
				});
			}
			walk_directory(
				&path,
				executables,
				valid_extensions,
				on_progress,
				scanned_dirs,
			)?;
		} else if path.is_file()
			&& let Some(ext) = path.extension().and_then(|e| e.to_str())
			&& valid_extensions.contains(&ext.to_lowercase().as_str())
		{
			if let Some(cb) = on_progress {
				cb(ScanProgress {
					scanned_dirs: *scanned_dirs,
					executables_found: executables.len() as u32 + 1,
					current_path: path.display().to_string(),
				});
			}
			executables.push(path);
		}
	}

	Ok(())
}

pub fn scan_directory(
	path: &Path,
	on_progress: impl Fn(ScanProgress) + Send + Sync + 'static,
) -> Result<DirectoryScanResult> {
	if !path.is_dir() {
		return Err(Error::NoExecutableFound(path.to_owned()));
	}

	let start = Instant::now();
	let mut executables = Vec::new();
	let mut scanned_dirs = 0u32;

	walk_directory(path, &mut executables, &VALID_EXTENSIONS, Some(&on_progress), &mut scanned_dirs)?;

	let duration_secs = start.elapsed().as_secs_f64();

	let games: Result<Vec<DbGame>> = executables
		.iter()
		.map(|exe_path| get_game_from_path(exe_path))
		.collect();
	let games = games?;

	Ok(DirectoryScanResult {
		games,
		duration_secs,
	})
}

fn remove_directory_path(path: &Path) -> Result {
	let config_path = games_config_path()?;
	let mut games_config = read_games_config(&config_path);
	games_config.directories.retain(|p| p != path);
	games_config.ignored_paths.retain(|p| !p.starts_with(path));

	fs::write(config_path, serde_json::to_string_pretty(&games_config)?)?;

	Ok(())
}

pub fn remove_directory(path: &Path) -> Result {
	remove_directory_path(path)
}

pub fn get_directories() -> Result<Vec<PathBuf>> {
	let config_path = games_config_path()?;
	let config = read_games_config(&config_path);
	Ok(config.directories)
}

pub fn remove_game(game: &DbGame) -> Result {
	if game.provider_id != GameProviderId::Manual {
		return Err(Error::InvalidProviderId(game.provider_id.to_string()));
	}

	let path = &game
		.exe_path
		.as_ref()
		.ok_or_else(|| Error::GameNotInstalled(game.display_title.clone()))?;

	let config_path = games_config_path()?;
	let mut games_config = read_games_config(&config_path);

	if games_config.paths.contains(path) {
		games_config.paths.retain(|p| p != *path);
	} else if !games_config.ignored_paths.contains(path) {
		games_config.ignored_paths.push((*path).clone());
	}

	fs::write(config_path, serde_json::to_string_pretty(&games_config)?)?;

	Ok(())
}

fn remove_path(path: &Path) -> Result {
	let config_path = games_config_path()?;
	let mut games_config = read_games_config(&config_path);
	games_config.paths.retain(|p| p != path);

	fs::write(config_path, serde_json::to_string_pretty(&games_config)?)?;

	Ok(())
}

fn clean_up_stale_ignored_paths(config: &GamesConfig) -> Result {
	if config.ignored_paths.is_empty() {
		return Ok(());
	}

	let mut changed = false;
	for ignored_path in &config.ignored_paths {
		if !ignored_path.exists() {
			changed = true;
			break;
		}
	}

	if !changed {
		return Ok(());
	}

	let config_path = games_config_path()?;
	let mut games_config = read_games_config(&config_path);
	games_config.ignored_paths.retain(|p| p.exists());

	fs::write(config_path, serde_json::to_string_pretty(&games_config)?)?;

	Ok(())
}
