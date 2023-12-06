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
	pub name: String,
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
		let name = paths::file_name_without_extension(path)?;

		Ok(Self {
			id: format!(
				"{}_{}_{}",
				name,
				engine.map_or(String::new(), |e| e.to_string()),
				scripting_backend.map_or(String::new(), |s| s.to_string()),
			),
			path: path.to_path_buf(),
			name: name.to_string(),
			engine,
			scripting_backend,
			kind,
		})
	}

	pub fn open_folder(&self) -> Result {
		Ok(open::that_detached(&self.path)?)
	}
}
