use std::{
	collections::hash_map::DefaultHasher,
	env,
	ffi::OsStr,
	fs,
	hash::{
		Hash,
		Hasher,
	},
	io,
	path::{
		Component,
		Path,
		PathBuf,
	},
	process::Stdio,
};

use directories::{
	BaseDirs,
	ProjectDirs,
};
use globwalk::glob;
use log;

use crate::result::{
	Error,
	LogErrExt,
	Result,
};

pub fn glob_path(path: &Path) -> Vec<PathBuf> {
	match path.try_to_str() {
		Ok(path_str) => match glob(path_str) {
			Ok(walker) => walker
				.into_iter()
				.filter_map(|glob_result| match glob_result {
					Ok(glob_entry) => Some(glob_entry.into_path()),
					Err(err) => {
						log::error!(
							"Failed to resolve one of the globbed paths from glob '{}'. Error: {}",
							path.display(),
							err
						);
						None
					}
				})
				.collect(),
			Err(err) => {
				log::error!("Failed to glob path `{}`. Error: {}", path.display(), err);
				Vec::default()
			}
		},
		Err(err) => {
			log::error!(
				"Failed to convert path to str `{}`. Error: {}",
				path.display(),
				err
			);
			Vec::default()
		}
	}
}

pub fn path_parent(path: &Path) -> Result<&Path> {
	path.parent()
		.ok_or_else(|| Error::PathParentNotFound(path.to_path_buf()))
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
	open_folder_or_parent(&logs_path()?)
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

pub fn file_name_without_extension(file_path: &Path) -> Result<&str> {
	file_path
		.file_stem()
		.ok_or_else(|| Error::InvalidOsStr(file_path.display().to_string()))?
		.try_to_str()
}

pub fn normalize_path(path: &Path) -> PathBuf {
	path.canonicalize().unwrap_or_else(|err| {
		log::error!("Failed to normalize path `{}`: {}", path.display(), err);
		path.to_path_buf()
	})
}

pub fn find_child_case_insensitive(parent: &Path, child_name: &OsStr) -> Option<PathBuf> {
	let direct_path = parent.join(Path::new(child_name));
	if direct_path.exists() {
		return Some(direct_path);
	}

	std::fs::read_dir(parent)
		.ok_or_log(&format!("Failed to read dir {}", parent.display()))?
		.flatten()
		.find(|entry| entry.file_name().eq_ignore_ascii_case(child_name))
		.map(|entry| entry.path())
}

pub fn resolve_relative_path_case_insensitive(
	base_path: &Path,
	relative_path: &Path,
) -> Option<PathBuf> {
	let direct_path = base_path.join(relative_path);
	if direct_path.exists() {
		return Some(direct_path);
	}

	let mut current = base_path.to_path_buf();

	for component in relative_path.components() {
		match component {
			Component::CurDir => {}
			Component::ParentDir => {
				current.pop();
			}
			Component::Normal(name) => {
				current.push(name);
				if current.exists() {
					continue;
				}

				current.pop();

				{
					let found = find_child_case_insensitive(&current, name)?;
					current.push(found.file_name()?);
				}
			}
			Component::RootDir | Component::Prefix(_) => return None,
		}
	}

	Some(current)
}

pub fn hash_path(path: &Path) -> String {
	let mut hasher = DefaultHasher::new();
	normalize_path(path).hash(&mut hasher);
	hasher.finish().to_string()
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

// Weird workaround for AppImage builds.
fn open_detached_clean_env(path: impl AsRef<OsStr>) -> Result {
	let mut last_err = io::Error::new(io::ErrorKind::NotFound, "No command to open the path");

	for mut cmd in open::commands(path) {
		cmd.env_remove("LD_LIBRARY_PATH");
		cmd.env_remove("QT_PLUGIN_PATH");
		cmd.env_remove("APPDIR");
		cmd.env_remove("APPIMAGE");

		cmd.stdin(Stdio::null())
			.stdout(Stdio::null())
			.stderr(Stdio::null());

		match cmd.spawn() {
			Ok(_) => return Ok(()),
			Err(e) => last_err = e,
		}
	}

	Err(last_err.into())
}

pub fn open_folder_or_parent(path: &Path) -> Result {
	let folder_path = if path.is_dir() {
		path
	} else {
		path_parent(path)?
	};

	// I've hard weird issues with non-normalized paths acting weird on Windows,
	// normalizing seems to fix it.
	open_detached_clean_env(normalize_path(folder_path))
}

pub fn base_dirs() -> Result<BaseDirs> {
	directories::BaseDirs::new().ok_or_else(Error::AppDataNotFound)
}

pub trait AsValidStr {
	fn try_to_str(&self) -> Result<&str>;
}

impl<T> AsValidStr for T
where
	T: AsRef<OsStr>,
{
	fn try_to_str(&self) -> Result<&str> {
		self.as_ref()
			.to_str()
			.ok_or_else(|| Error::InvalidOsStr(self.as_ref().to_string_lossy().to_string()))
	}
}

impl AsValidStr for OsStr {
	fn try_to_str(&self) -> Result<&str> {
		self.to_str()
			.ok_or_else(|| Error::InvalidOsStr(self.to_string_lossy().to_string()))
	}
}

impl AsValidStr for Path {
	fn try_to_str(&self) -> Result<&str> {
		self.to_str()
			.ok_or_else(|| Error::InvalidOsStr(self.to_string_lossy().to_string()))
	}
}
