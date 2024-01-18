use crate::{
	game_engines::{
		game_engine::GameEngineBrand,
		unity::UnityScriptingBackend,
	},
	serializable_struct,
};

serializable_struct!(CommonModData {
	pub id: String,
	pub engine: Option<GameEngineBrand>,
	pub unity_backend: Option<UnityScriptingBackend>,
	pub loader_id: String, // TODO make enum
});
