use std::path::{
	Path,
	PathBuf,
};

use crate::{
	game_engines::{
		game_engine::GameEngineBrand,
		unity::UnityScriptingBackend,
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

serializable_struct!(LocalMod {
	pub id: String,
	pub scripting_backend: Option<UnityScriptingBackend>,
	pub engine: Option<GameEngineBrand>,
	pub kind: ModKind,
	pub path: PathBuf,
});

impl LocalMod {
	pub fn new(
		path: &Path,
		engine: Option<GameEngineBrand>,
		scripting_backend: Option<UnityScriptingBackend>,
		kind: ModKind,
	) -> Result<Self> {
		Ok(Self {
			id: paths::file_name_without_extension(path)?.to_string(),
			path: path.to_path_buf(),
			engine,
			scripting_backend,
			kind,
		})
	}

	pub fn open_folder(&self) -> Result {
		Ok(open::that_detached(&self.path)?)
	}
}
