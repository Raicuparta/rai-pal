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

pub trait PathExt {
	fn glob(&self) -> Vec<PathBuf>;
	fn file_name_without_extension(&self) -> Result<&str>;
	fn normalize(&self) -> PathBuf;
	fn find_child_case_insensitive(&self, child_name: &OsStr) -> Option<PathBuf>;
	fn resolve_relative_path_case_insensitive(&self, relative_path: &Path) -> Option<PathBuf>;
	fn try_parent(&self) -> Result<&Path>;
	fn hash_string(&self) -> String;
	fn open_folder_or_parent(&self) -> Result;
	fn remove_if_exists(&self) -> Result;
}

impl PathExt for Path {
	fn glob(&self) -> Vec<PathBuf> {
		match self.try_to_str() {
			Ok(path_str) => match glob(path_str) {
				Ok(walker) => {
					walker
						.into_iter()
						.filter_map(|glob_result| match glob_result {
							Ok(glob_entry) => Some(glob_entry.into_path()),
							Err(err) => {
								// Ignore not found error since that's the point.
								err.path()?;

								log::error!(
									"Failed to resolve one of the globbed paths from glob '{}'. Error: {}",
									self.display(),
									err
								);
								None
							}
						})
						.collect()
				}
				Err(err) => {
					log::error!("Failed to glob path `{}`. Error: {}", self.display(), err);
					Vec::default()
				}
			},
			Err(err) => {
				log::error!(
					"Failed to convert path to str `{}`. Error: {}",
					self.display(),
					err
				);
				Vec::default()
			}
		}
	}

	fn file_name_without_extension(&self) -> Result<&str> {
		self.file_stem()
			.ok_or_else(|| Error::InvalidOsStr(self.display().to_string()))?
			.try_to_str()
	}

	fn normalize(&self) -> PathBuf {
		// dunce::canonicalize resolves the path like std::fs::canonicalize, but on Windows
		// it avoids returning verbatim UNC paths (\\?\...) that some programs (e.g. explorer.exe)
		// don't accept.
		dunce::canonicalize(self).unwrap_or_else(|err| {
			log::error!("Failed to normalize path `{}`: {}", self.display(), err);
			self.to_path_buf()
		})
	}

	fn find_child_case_insensitive(&self, child_name: &OsStr) -> Option<PathBuf> {
		let direct_path = self.join(Self::new(child_name));
		if direct_path.exists() {
			return Some(direct_path);
		}

		std::fs::read_dir(self)
			.ok_or_log(&format!("Failed to read dir {}", self.display()))?
			.flatten()
			.find(|entry| entry.file_name().eq_ignore_ascii_case(child_name))
			.map(|entry| entry.path())
	}

	fn resolve_relative_path_case_insensitive(&self, relative_path: &Path) -> Option<PathBuf> {
		let direct_path = self.join(relative_path);
		if direct_path.exists() {
			return Some(direct_path);
		}

		let mut current = self.to_path_buf();

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
						let found = current.find_child_case_insensitive(name)?;
						current.push(found.file_name()?);
					}
				}
				Component::RootDir | Component::Prefix(_) => return None,
			}
		}

		Some(current)
	}

	fn try_parent(&self) -> Result<&Path> {
		self.parent()
			.ok_or_else(|| Error::PathParentNotFound(self.to_path_buf()))
	}

	fn hash_string(&self) -> String {
		let mut hasher = DefaultHasher::new();
		self.normalize().hash(&mut hasher);
		hasher.finish().to_string()
	}

	fn open_folder_or_parent(&self) -> Result {
		let folder_path = if self.is_dir() {
			self
		} else {
			self.try_parent()?
		};

		let normalized_path = folder_path.normalize();

		fs::create_dir_all(&normalized_path)?;

		// I've had weird issues with non-normalized paths acting weird on Windows,
		// normalizing seems to fix it.
		open_detached_better(normalized_path)
	}

	fn remove_if_exists(&self) -> Result {
		let Ok(metadata) = fs::symlink_metadata(self) else {
			return Ok(());
		};

		if metadata.is_dir() {
			fs::remove_dir_all(self)?;
		} else {
			fs::remove_file(self)?;
		}

		Ok(())
	}
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
		#[allow(
			clippy::disallowed_methods,
			reason = "This is the replacement for to_string_lossy(), so it uses it internally"
		)]
		self.to_str()
			.ok_or_else(|| Error::InvalidOsStr(self.to_string_lossy().to_string()))
	}
}
