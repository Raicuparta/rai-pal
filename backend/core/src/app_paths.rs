use std::{
	self,
	env,
	fs,
	path::PathBuf,
};

use directories::{
	BaseDirs,
	ProjectDirs,
};
use log;

use crate::result::{
	Error,
	Result,
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
	let project_dirs =
		ProjectDirs::from("com", "raicuparta", "rai-pal").ok_or_else(Error::AppDataNotFound)?;
	let path = project_dirs.data_dir().to_path_buf();
	fs::create_dir_all(&path)?;

	Ok(path)
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

fn databases_path() -> Result<PathBuf> {
	app_data_subfolder("databases")
}

pub fn temp_dir(sub_dir: &str) -> Result<PathBuf> {
	let path = std::env::temp_dir().join("rai-pal").join(sub_dir);
	fs::create_dir_all(&path)?;
	Ok(path)
}

// We don't have migrations. If any database schema changes, update this so they get recreated.
const DATABASE_VERSION: u32 = 2u32;

pub fn database_path(database_name: &str) -> Result<PathBuf> {
	Ok(databases_path()?.join(format!("{database_name}-{DATABASE_VERSION}.db")))
}
