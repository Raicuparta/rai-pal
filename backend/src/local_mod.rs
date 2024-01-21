use std::{
	collections::HashMap,
	path::{
		Path,
		PathBuf,
	},
};

use crate::{
	game_engines::{
		game_engine::GameEngineBrand,
		unity::UnityScriptingBackend,
	},
	game_mod::CommonModData,
	mod_manifest::{
		self,
		Manifest,
	},
	paths,
	serializable_enum,
	serializable_struct,
	Result,
};

serializable_enum!(ModKind {
	Installable,
	Runnable,
});

serializable_struct!(LocalModData {
	pub path: PathBuf,
	pub manifest: Option<Manifest>,
});

serializable_struct!(LocalMod {
	pub data: LocalModData,
	pub common: CommonModData,
});

pub fn get_manifest_path(mod_path: &Path) -> PathBuf {
	mod_path.join(mod_manifest::Manifest::FILE_NAME)
}

impl LocalMod {
	pub fn new(
		loader_id: &str,
		path: &Path,
		engine: Option<GameEngineBrand>,
		unity_backend: Option<UnityScriptingBackend>,
	) -> Result<Self> {
		Ok(Self {
			data: LocalModData {
				path: path.to_path_buf(),
				manifest: mod_manifest::get(&get_manifest_path(path)),
			},
			common: CommonModData {
				id: paths::file_name_without_extension(path)?.to_string(),
				engine,
				unity_backend,
				loader_id: loader_id.to_string(),
			},
		})
	}

	pub fn open_folder(&self) -> Result {
		let path = if self.data.path.is_dir() {
			&self.data.path
		} else {
			paths::path_parent(&self.data.path)?
		};

		Ok(open::that_detached(path)?)
	}
}

pub type Map = HashMap<String, LocalMod>;
