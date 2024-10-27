use rai_pal_proc_macros::{serializable_enum, serializable_struct};

#[serializable_enum]
pub enum EngineBrand {
	Unity,
	Unreal,
	Godot,
	GameMaker,
}

#[serializable_struct]
pub struct GameEngine {
	pub brand: EngineBrand,
	pub version: Option<EngineVersion>,
}

#[serializable_struct]
pub struct EngineVersionNumbers {
	pub major: u32,
	pub minor: Option<u32>,
	pub patch: Option<u32>,
}

#[serializable_struct]
pub struct EngineVersion {
	pub numbers: EngineVersionNumbers,
	pub suffix: Option<String>,
	pub display: String,
}
