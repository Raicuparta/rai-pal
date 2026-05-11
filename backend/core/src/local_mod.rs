use std::{
	collections::HashMap,
	fs,
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
		LogErrExt,
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

pub fn get_all() -> Result<HashMap<String, LocalMod>> {
	Ok(paths::glob_path(
		&paths::local_mods_path()?
			.join("*")
			.join(mod_manifest::Manifest::FILE_NAME),
	)
	.iter()
	.filter_map(|manifest_path| {
		LocalMod::new(manifest_path)
			.ok_or_log("Failed to create local mod")
			.map(|local_mod| (local_mod.common.id.clone(), local_mod))
	})
	.collect())
}

pub type Map = HashMap<String, LocalMod>;

pub fn delete(local_mod: &LocalMod) -> Result {
	if local_mod.data.path.exists() {
		fs::remove_dir_all(&local_mod.data.path)?;
	}

	Ok(())
}
