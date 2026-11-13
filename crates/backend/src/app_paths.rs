use fs_err as fs;
use std::{self, env, path::PathBuf};

use directories::{BaseDirs, ProjectDirs};
use log;

use crate::{
	app_variant::AppVariant,
	result::{Error, Result},
};

pub fn base_dirs() -> Result<BaseDirs> {
	directories::BaseDirs::new().ok_or_else(Error::AppDataNotFound)
}

fn get_program_data_path() -> Result<PathBuf> {
	let path_from_env = env::var("ProgramData")?;
	Ok(PathBuf::from(path_from_env))
}

pub fn try_get_program_data_path() -> PathBuf {
	get_program_data_path().unwrap_or_else(|err| {
		log::error!("Failed to get ProgramData path from environment: {err}");
		PathBuf::from("C:/ProgramData")
	})
}

fn app_data_path() -> Result<PathBuf> {
	let path = match AppVariant::get_current() {
		AppVariant::WindowsPortable => portable_data_path()?,
		_ => project_data_path()?,
	};
	fs::create_dir_all(&path)?;

	Ok(path)
}

fn project_data_path() -> Result<PathBuf> {
	let project_dirs =
		ProjectDirs::from("com", "raicuparta", "rai-pal").ok_or_else(Error::AppDataNotFound)?;
	Ok(project_dirs.data_dir().to_path_buf())
}

fn portable_data_path() -> Result<PathBuf> {
	let exe_path = std::env::current_exe()?;
	let app_folder = exe_path
		.parent()
		.ok_or_else(|| Error::PathParentNotFound(exe_path.clone()))?;
	Ok(app_folder.join("Data"))
}

pub fn app_data_file(file_name: &str) -> Result<PathBuf> {
	Ok(app_data_path()?.join(file_name))
}

pub fn app_data_subfolder(folder_name: &str) -> Result<PathBuf> {
	let path = app_data_path()?.join(folder_name);
	fs::create_dir_all(&path)?;
	Ok(path)
}

pub fn logs_path() -> Result<PathBuf> {
	app_data_subfolder("logs")
}

pub fn shared_mods_path() -> Result<PathBuf> {
	app_data_subfolder("shared-mods")
}

pub fn local_mods_path() -> Result<PathBuf> {
	app_data_subfolder("local-mods")
}

pub fn installed_mods_path() -> Result<PathBuf> {
	app_data_subfolder("installed-mods")
}

pub fn thumbnail_cache_path() -> Result<PathBuf> {
	app_data_subfolder("thumbnail-cache")
}

fn databases_path() -> Result<PathBuf> {
	app_data_subfolder("databases")
}

pub fn temp_dir(sub_dir: &str) -> Result<PathBuf> {
	let path = std::env::temp_dir().join("rai-pal").join(sub_dir);
	fs::create_dir_all(&path)?;
	Ok(path)
}

// We don't have migrations. If any database schema changes, update this so they get recreated.
const DATABASE_VERSION: u32 = 10u32;

pub fn database_path(database_name: &str) -> Result<PathBuf> {
	Ok(databases_path()?.join(format!("{database_name}-{DATABASE_VERSION}.db")))
}
