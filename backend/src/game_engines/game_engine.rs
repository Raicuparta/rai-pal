use crate::{
	serializable_enum,
	serializable_struct,
};

serializable_enum!(GameEngineBrand {
	Unity,
	Unreal,
	Godot,
	GameMaker,
});

serializable_struct!(GameEngine {
	pub brand: GameEngineBrand,
	pub version: Option<GameEngineVersion>,
});

serializable_struct!(GameEngineVersion {
	pub major: u32,
	pub minor: u32,
	pub patch: u32,
	pub suffix: Option<String>,
	pub display: String,
});
