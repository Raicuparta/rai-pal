use std::{
	collections::hash_map::DefaultHasher,
	ffi::OsStr,
	fs,
	hash::{
		Hash,
		Hasher,
	},
	path::{
		Component,
		Path,
		PathBuf,
	},
};

use globwalk::glob;
use log;

use crate::{
	open_better::open_detached_better,
	result::{
		Error,
		LogErrExt,
		Result,
	},
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

pub fn file_name_without_extension(file_path: &Path) -> Result<&str> {
	file_path
		.file_stem()
		.ok_or_else(|| Error::InvalidOsStr(file_path.display().to_string()))?
		.try_to_str()
}

pub fn normalize_path(path: &Path) -> PathBuf {
	// dunce::canonicalize resolves the path like std::fs::canonicalize, but on Windows
	// it avoids returning verbatim UNC paths (\\?\...) that some programs (e.g. explorer.exe)
	// don't accept.
	dunce::canonicalize(path).unwrap_or_else(|err| {
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

pub fn path_parent(path: &Path) -> Result<&Path> {
	path.parent()
		.ok_or_else(|| Error::PathParentNotFound(path.to_path_buf()))
}

pub fn hash_path(path: &Path) -> String {
	let mut hasher = DefaultHasher::new();
	normalize_path(path).hash(&mut hasher);
	hasher.finish().to_string()
}

pub fn open_folder_or_parent(path: &Path) -> Result {
	let folder_path = if path.is_dir() {
		path
	} else {
		path_parent(path)?
	};

	let normalized_path = normalize_path(folder_path);

	fs::create_dir_all(&normalized_path)?;

	// I've hard weird issues with non-normalized paths acting weird on Windows,
	// normalizing seems to fix it.
	open_detached_better(normalized_path)
}

pub fn remove_path_if_exists(path: &Path) -> Result {
	let Ok(metadata) = fs::symlink_metadata(path) else {
		return Ok(());
	};

	if metadata.is_dir() {
		fs::remove_dir_all(path)?;
	} else {
		fs::remove_file(path)?;
	}

	Ok(())
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
