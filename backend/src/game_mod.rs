use std::collections::{
	HashMap,
	HashSet,
};

use crate::{
	game_engines::{
		game_engine::GameEngineBrand,
		unity::UnityScriptingBackend,
	},
	local_mod::{self,},
	remote_mod,
	serializable_struct,
};

serializable_struct!(CommonModData {
	pub id: String,
	pub engine: Option<GameEngineBrand>,
	pub unity_backend: Option<UnityScriptingBackend>,
	pub loader_id: String, // TODO make enum
});

pub type CommonDataMap = HashMap<String, CommonModData>;

pub fn get_common_data_map(
	local_mods: &local_mod::Map,
	remote_mods: &remote_mod::Map,
) -> CommonDataMap {
	let keys: HashSet<_> = remote_mods
		.keys()
		.chain(local_mods.keys())
		.cloned()
		.collect();

	keys.iter()
		.filter_map(|key| {
			Some((
				key.clone(),
				local_mods.get(key).map_or_else(
					|| remote_mods.get(key).map(|local| local.common.clone()),
					|remote| Some(remote.common.clone()),
				)?,
			))
		})
		.collect()
}
