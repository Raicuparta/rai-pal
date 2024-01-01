use std::{
	collections::hash_map::DefaultHasher,
	hash::{
		Hash,
		Hasher,
	},
	path::{
		Path,
		PathBuf,
	},
};

use directories::ProjectDirs;
use glob::{
	glob,
	Paths,
};
use log::error;

use crate::{
	Error,
	Result,
};

pub fn path_to_str(path: &Path) -> Result<&str> {
	path.to_str()
		.ok_or_else(|| Error::PathParseFailure(path.to_path_buf()))
}

pub fn glob_path(path: &Path) -> Result<Paths> {
	Ok(glob(path_to_str(path)?)?)
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

pub fn installed_mods_path() -> Result<PathBuf> {
	Ok(app_data_path()?.join("mod-loaders"))
}

pub fn resources_path(handle: &tauri::AppHandle) -> Result<PathBuf> {
	handle
		.path_resolver()
		.resolve_resource("resources")
		.ok_or_else(Error::ResourcesNotFound)
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
