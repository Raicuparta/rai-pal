use std::{fs, path::Path};

use log::error;
use rai_pal_proc_macros::serializable_struct;

use crate::{
	game_engines::{game_engine::EngineBrand, unity::UnityScriptingBackend},
	game_mod::EngineVersionRange,
	mod_loaders::mod_database::RunnableModData,
};

#[serializable_struct]
pub struct Manifest {
	pub title: Option<String>,
	pub version: String,
	pub runnable: Option<RunnableModData>,
	pub engine: Option<EngineBrand>,
	pub engine_version_range: Option<EngineVersionRange>,
	pub unity_backend: Option<UnityScriptingBackend>,
}

impl Manifest {
	pub const FILE_NAME: &'static str = "rai-pal-manifest.json";
}

pub fn get(path: &Path) -> Option<Manifest> {
	match fs::read_to_string(path)
		.and_then(|manifest_bytes| Ok(serde_json::from_str::<Manifest>(&manifest_bytes)?))
	{
		Ok(manifest) => Some(manifest),
		Err(error) => {
			error!(
				"Error getting manifest in path '{}': {}",
				path.display(),
				error
			);
			None
		}
	}
}
