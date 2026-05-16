use std::{
	collections::HashMap,
	fs,
};

use crate::{
	game_mods::mod_database::{
		self,
		GameMod,
		ModDatabase,
	},
	local_mod::{
		self,
	},
	result::{
		Error,
		LogErrExt,
	},
};

pub async fn get_all_remote<F>(error_handler: F) -> HashMap<String, GameMod>
where
	F: Fn(Error) + Send,
{
	let database = mod_database::get().await.unwrap_or_else(|error| {
		error_handler(error);
		ModDatabase { mods: Vec::new() }
	});

	let mut mods_map = HashMap::new();
	let local_mods = local_mod::get_all().ok_or_log("Failed to get local mods");

	for remote_mod in database.mods {
		// If there's a local mod with the same ID, update its manifest with remote info
		if let Some(local_mod) = local_mods
			.as_ref()
			.and_then(|local_mods| local_mods.get(&remote_mod.id))
			&& let Some(manifest_path) = local_mod
				.get_local_manifest_path()
				.ok_or_log("Failed to get manifest path for local mod.")
		{
			// Only update if the manifest file exists (mod has been downloaded before)
			if manifest_path.exists()
				&& let Ok(manifest_contents) = serde_json::to_string_pretty(&remote_mod)
			{
				// TODO what's going on with this result?
				let _ = fs::write(&manifest_path, manifest_contents);
			}
		}

		mods_map.insert(remote_mod.id.clone(), remote_mod);
	}

	mods_map
}
