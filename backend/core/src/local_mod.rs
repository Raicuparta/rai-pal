use std::{
	collections::HashMap,
	path::{
		Path,
		PathBuf,
	},
};

use rai_pal_proc_macros::{
	serializable_enum,
	serializable_struct,
};

use crate::{
	game_mod::CommonModData,
	mod_manifest::{
		self,
		Manifest,
	},
	paths::{
		self,
		open_folder_or_parent,
	},
	result::{
		Error,
		Result,
	},
};

#[serializable_enum]
pub enum ModKind {
	Installable,
	Runnable,
}

#[serializable_struct]
pub struct LocalModData {
	pub path: PathBuf,
	pub manifest: Manifest,
}

#[serializable_struct]
pub struct LocalMod {
	pub data: LocalModData,
	pub common: CommonModData,
}

pub fn get_manifest_path(mod_path: &Path) -> PathBuf {
	mod_path.join(mod_manifest::Manifest::FILE_NAME)
}

impl LocalMod {
	pub fn new(manifest_path: &Path) -> Result<Self> {
		let manifest = mod_manifest::get(manifest_path)
			.ok_or_else(|| Error::ManifestNotFound(manifest_path.display().to_string()))?;

		let mod_path = paths::path_parent(manifest_path)?;

		Ok(Self {
			data: LocalModData {
				path: mod_path.to_path_buf(),
				manifest: manifest.clone(),
			},
			common: CommonModData {
				id: paths::file_name_without_extension(mod_path)?.to_string(),
				engine: manifest.engine,
				engine_version_range: manifest.engine_version_range,
				is_loader: manifest.is_loader,
				architecture: manifest.architecture,
				unity_backend: manifest.unity_backend,
				loader_id: manifest.loader_id,
			},
		})
	}

	pub fn open_folder(&self) -> Result {
		open_folder_or_parent(&self.data.path)
	}
}

pub type Map = HashMap<String, LocalMod>;
