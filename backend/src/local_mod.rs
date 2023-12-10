use std::{
	collections::HashMap,
	fs,
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

serializable_struct!(Manifest {
	pub version: String,
});

pub fn get_manifest_path(mod_path: &Path) -> PathBuf {
	mod_path.join("rai-pal-manifest.json")
}

fn get_manifest(mod_path: &Path) -> Option<Manifest> {
	match fs::read_to_string(get_manifest_path(mod_path))
		.and_then(|manifest_bytes| Ok(serde_json::from_str::<Manifest>(&manifest_bytes)?))
	{
		Ok(manifest) => Some(manifest),
		Err(error) => {
			eprintln!("Error getting manifest: {error}");
			None
		}
	}
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
				manifest: get_manifest(path),
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
		Ok(open::that_detached(&self.data.path)?)
	}
}

pub type Map = HashMap<String, LocalMod>;
