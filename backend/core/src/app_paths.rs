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

use crate::{
	paths,
	result::{
		Error,
		Result,
	},
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

fn app_data_subfolder(folder_name: &str) -> Result<PathBuf> {
	let path = app_data_path()?.join(folder_name);
	fs::create_dir_all(&path)?;
	Ok(path)
}

pub fn logs_path() -> Result<PathBuf> {
	app_data_subfolder("logs")
}

pub fn open_logs_folder() -> Result {
	paths::open_folder_or_parent(&logs_path()?)
}

pub fn local_mods_path() -> Result<PathBuf> {
	app_data_subfolder("mods")
}

pub fn installed_mods_path() -> Result<PathBuf> {
	app_data_subfolder("installed_mods")
}

pub fn downloads_path() -> Result<PathBuf> {
	app_data_subfolder("downloads")
}

fn databases_path() -> Result<PathBuf> {
	app_data_subfolder("databases")
}

pub fn database_path(database_name: &str) -> Result<PathBuf> {
	Ok(databases_path()?.join(format!("{database_name}.db")))
}
