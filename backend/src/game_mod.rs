use std::path::{
	Path,
	PathBuf,
};

use anyhow::anyhow;

use crate::{
	files::copy_dir_all,
	game::UnityScriptingBackend,
	serializable_struct,
	Result,
};

serializable_struct!(Mod {
	pub id: String,
	pub name: String,
	pub scripting_backend: UnityScriptingBackend,
	path: PathBuf,
});

impl Mod {
	pub fn new(path: &Path, scripting_backend: UnityScriptingBackend) -> Result<Self> {
		let name = String::from(
			path.file_name()
				.ok_or_else(|| anyhow!("Failed to get file name"))?
				.to_string_lossy(),
		);

		Ok(Self {
			id: format!("{scripting_backend}/{name}"),
			path: path.to_path_buf(),
			name,
			scripting_backend,
		})
	}

	pub fn install(&self, folder_path: &Path) -> Result {
		copy_dir_all(&self.path, folder_path.join(&self.name))
			.map_err(|err| anyhow!("Failed to install mod: {err}"))
	}

	pub fn open_folder(&self) -> Result {
		open::that_detached(&self.path).map_err(|err| anyhow!("Failed to open mod folder: {err}"))
	}
}
