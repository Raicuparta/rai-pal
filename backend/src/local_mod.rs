use std::path::{
	Path,
	PathBuf,
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
	pub kind: ModKind,
	pub path: PathBuf,
});

serializable_struct!(LocalMod {
	pub data: LocalModData,
	pub common: CommonModData,
});

impl LocalMod {
	pub fn new(
		path: &Path,
		engine: Option<GameEngineBrand>,
		unity_backend: Option<UnityScriptingBackend>,
		kind: ModKind,
	) -> Result<Self> {
		Ok(Self {
			data: LocalModData {
				kind,
				path: path.to_path_buf(),
			},
			common: CommonModData {
				id: paths::file_name_without_extension(path)?.to_string(),
				engine,
				unity_backend,
			},
		})
	}
}
