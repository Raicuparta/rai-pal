use crate::{
	game_engines::{
		game_engine::{EngineBrand, EngineVersionNumbers},
		unity::UnityScriptingBackend,
	},
	serializable_struct,
};

#[serializable_struct]
pub struct EngineVersionRange {
	pub minimum: Option<EngineVersionNumbers>,
	pub maximum: Option<EngineVersionNumbers>,
}

#[serializable_struct]
pub struct CommonModData {
	pub id: String,
	pub engine: Option<EngineBrand>,
	pub unity_backend: Option<UnityScriptingBackend>,
	pub engine_version_range: Option<EngineVersionRange>,
	pub loader_id: String, // TODO make enum
}
