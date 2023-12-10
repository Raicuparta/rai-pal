use std::collections::HashMap;

use crate::{
	game_engines::{
		game_engine::GameEngineBrand,
		unity::UnityScriptingBackend,
	},
	local_mod::LocalModData,
	mod_loaders::mod_database::RemoteModData,
	serializable_struct,
	Error,
	Result,
};

serializable_struct!(CommonModData {
	pub id: String,
	pub engine: Option<GameEngineBrand>,
	pub unity_backend: Option<UnityScriptingBackend>,
});

serializable_struct!(GameMod {
  pub local_mod: Option<LocalModData>,
  pub remote_mod: Option<RemoteModData>,
	pub common: CommonModData,
	pub loader_id: String,
});

impl GameMod {
	pub fn open_folder(&self) -> Result {
		if let Some(local_mod) = &self.local_mod {
			Ok(open::that_detached(&local_mod.path)?)
		} else {
			Err(Error::LocalModRequired(self.common.id.clone()))
		}
	}
}

pub type Map = HashMap<String, GameMod>;
