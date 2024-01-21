use std::{
	fs,
	path::Path,
};

use log::error;

use crate::{
	game_engines::{
		game_engine::GameEngineBrand,
		unity::UnityScriptingBackend,
	},
	mod_loaders::mod_database::RunnableModData,
	serializable_struct,
};

serializable_struct!(Manifest {
	pub version: String,
	pub runnable: Option<RunnableModData>,
	pub engine: Option<GameEngineBrand>,
	pub unity_backend: Option<UnityScriptingBackend>,
});

impl Manifest {
	pub const FILE_NAME: &'static str = "rai-pal-manifest.json";
}

pub fn get(path: &Path) -> Option<Manifest> {
	match fs::read_to_string(path)
		.and_then(|manifest_bytes| Ok(serde_json::from_str::<Manifest>(&manifest_bytes)?))
	{
		Ok(manifest) => Some(manifest),
		Err(error) => {
			error!("Error getting manifest: {error}");
			None
		}
	}
}
