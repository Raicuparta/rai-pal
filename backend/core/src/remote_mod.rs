use std::{
	collections::HashMap,
	fs::{
		self,
		File,
	},
};

use zip::ZipArchive;

use crate::{
	files,
	game_mods::mod_database::{
		self,
		DatabaseEntry,
		ModDatabase,
	},
	local_mod::{
		self,
	},
	paths,
	result::{
		Error,
		LogErrExt,
		Result,
	},
};

pub type Map = HashMap<String, DatabaseEntry>;

pub async fn download(remote_mod: &DatabaseEntry) -> Result {
	let target_path = paths::local_mods_path()?.join(&remote_mod.id);
	let downloads_path = paths::downloads_path()?;
	let mod_id = &remote_mod.id;

	let response = reqwest::get(&remote_mod.latest_version.url).await?;

	fs::create_dir_all(&downloads_path)?;

	let zip_path = downloads_path.join(format!("{mod_id}.zip"));

	// TODO Stream to disk instead of keeping it all in memory.
	fs::write(&zip_path, response.bytes().await?)?;
	let file = File::open(&zip_path)?;

	let mut zip_archive = ZipArchive::new(file)?;

	files::extract(&mut zip_archive, &target_path)?;

	fs::write(
		local_mod::get_manifest_path(&target_path),
		serde_json::to_string_pretty(&remote_mod)?,
	)?;

	Ok(())
}

pub async fn get_all<F>(error_handler: F) -> HashMap<String, DatabaseEntry>
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
				.get_manifest_path()
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
