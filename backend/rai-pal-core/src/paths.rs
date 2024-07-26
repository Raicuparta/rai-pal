use std::{
	collections::hash_map::DefaultHasher,
	env,
	hash::{Hash, Hasher},
	path::{Path, PathBuf},
};

use directories::ProjectDirs;
use glob::glob;
use log::error;
use tauri::{path::BaseDirectory, Manager};

use crate::result::{Error, Result};

pub fn glob_path(path: &Path) -> Vec<PathBuf> {
	match glob(path.to_string_lossy().as_ref()) {
		Ok(paths) => paths
			.filter_map(|glob_result| match glob_result {
				Ok(globbed_path) => Some(globbed_path),
				Err(err) => {
					error!(
						"Failed to resolve one of the globbed paths from glob '{}'. Error: {}",
						path.display(),
						err
					);
					None
				}
			})
			.collect(),
		Err(err) => {
			error!("Failed to glob path `{}`. Error: {}", path.display(), err);
			Vec::default()
		}
	}
}

pub fn path_parent(path: &Path) -> Result<&Path> {
	path.parent()
		.ok_or_else(|| Error::PathParentNotFound(path.to_path_buf()))
}

pub fn app_data_path() -> Result<PathBuf> {
	let project_dirs =
		ProjectDirs::from("com", "raicuparta", "rai-pal").ok_or_else(Error::AppDataNotFound)?;

	Ok(project_dirs.data_dir().to_path_buf())
}

pub fn logs_path() -> Result<PathBuf> {
	Ok(app_data_path()?.join("logs"))
}

pub fn open_logs_folder() -> Result {
	Ok(open::that_detached(logs_path()?)?)
}

pub fn installed_mods_path() -> Result<PathBuf> {
	Ok(app_data_path()?.join("mod-loaders"))
}

pub fn resources_path(handle: &tauri::AppHandle) -> Result<PathBuf> {
	Ok(handle
		.path()
		.resolve("resources", BaseDirectory::Resource)?)
}

pub fn file_name_without_extension(file_path: &Path) -> Result<&str> {
	file_path
		.file_stem()
		.ok_or_else(|| Error::PathParseFailure(file_path.to_path_buf()))?
		.to_str()
		.ok_or_else(|| Error::PathParseFailure(file_path.to_path_buf()))
}

pub fn normalize_path(path: &Path) -> PathBuf {
	path.canonicalize().unwrap_or_else(|err| {
		error!(
			"Failed to normalize path `{}`: {}",
			path.to_string_lossy(),
			err
		);
		path.to_path_buf()
	})
}

pub fn hash_path(path: &Path) -> String {
	let mut hasher = DefaultHasher::new();
	path.hash(&mut hasher);
	hasher.finish().to_string()
}

fn get_program_data_path() -> Result<PathBuf> {
	let path_from_env = env::var("ProgramData")?;
	Ok(PathBuf::from(path_from_env))
}

pub fn try_get_program_data_path() -> PathBuf {
	get_program_data_path().unwrap_or_else(|err| {
		error!("Failed to get ProgramData path from environment: {err}");
		PathBuf::from("C:/ProgramData")
	})
}
