use std::path::{
	Path,
	PathBuf,
};

use crate::{
	files::copy_dir_all,
	game_engines::{
		game_engine::GameEngineBrand,
		unity::UnityScriptingBackend,
	},
	serializable_struct,
	Error,
	Result,
};

serializable_struct!(Mod {
	pub id: String,
	pub name: String,
	pub scripting_backend: Option<UnityScriptingBackend>,
	pub engine: Option<GameEngineBrand>,
	path: PathBuf,
});

impl Mod {
	pub fn new(
		path: &Path,
		engine: Option<GameEngineBrand>,
		scripting_backend: Option<UnityScriptingBackend>,
	) -> Result<Self> {
		let name = String::from(
			path.file_name()
				.ok_or_else(|| Error::FailedToGetFileName(path.to_path_buf()))?
				.to_string_lossy(),
		);

		Ok(Self {
			id: name.clone(),
			path: path.to_path_buf(),
			name,
			engine,
			scripting_backend,
		})
	}

	pub fn install(&self, folder_path: &Path) -> Result {
		copy_dir_all(&self.path, folder_path.join(&self.id))
	}

	pub fn open_folder(&self) -> Result {
		Ok(open::that_detached(&self.path)?)
	}
}
