use std::path::PathBuf;

use rai_pal_proc_macros::serializable_struct;

use crate::{
	architecture::Architecture,
	game_engines::{
		game_engine::{
			EngineBrand,
			EngineVersionNumbers,
		},
		unity::UnityBackend,
	},
	mod_loaders::mod_loader::ModLoaderId,
	paths,
	result::Result,
};

#[serializable_struct]
pub struct EngineVersionRange {
	pub minimum: Option<EngineVersionNumbers>,
	pub maximum: Option<EngineVersionNumbers>,
}

#[serializable_struct]
pub struct CommonModData {
	pub id: String,
	pub is_loader: Option<bool>,
	pub engine: Option<EngineBrand>,
	pub unity_backend: Option<UnityBackend>,
	pub engine_version_range: Option<EngineVersionRange>,
	pub architecture: Option<Architecture>,
	pub loader_id: ModLoaderId,
}

impl CommonModData {
	pub fn get_path(&self) -> Result<PathBuf> {
		Ok(paths::installed_mods_path()?
			.join(self.loader_id.as_str())
			.join("mods")
			.join(&self.id))
	}
}
