use std::{
	collections::HashMap,
	fs::{
		self,
		File,
	},
};

use rai_pal_proc_macros::serializable_struct;
use zip::ZipArchive;

use crate::{
	files,
	game_mods::{
		game_mod::CommonModData,
		mod_config::ModConfig,
		mod_database::{
			self,
			ModDatabase,
			ModDependency,
			ModDownload,
		},
	},
	local_mod::{
		self,
	},
	mod_manifest,
	paths,
	result::{
		Error,
		LogErrExt,
		Result,
	},
};

#[serializable_struct]
pub struct RemoteModData {
	pub title: String,
	pub deprecated: bool,
	pub author: String,
	pub source_code: String,
	pub description: String,
	pub latest_version: Option<ModDownload>,
	pub config: Option<ModConfig>,
	pub dependencies: Option<Vec<ModDependency>>,
}

#[serializable_struct]
pub struct RemoteMod {
	pub common: CommonModData,
	pub data: RemoteModData,
}

pub type Map = HashMap<String, RemoteMod>;

pub async fn download(remote_mod: &RemoteMod) -> Result {
	if let Some(latest_version) = &remote_mod.data.latest_version {
		let target_path = paths::local_mods_path()?.join(&remote_mod.common.id);
		let downloads_path = paths::downloads_path()?;
		let mod_id = &remote_mod.common.id;

		let response = reqwest::get(&latest_version.url).await?;

		fs::create_dir_all(&downloads_path)?;

		let zip_path = downloads_path.join(format!("{mod_id}.zip"));

		// TODO Stream to disk instead of keeping it all in memory.
		fs::write(&zip_path, response.bytes().await?)?;
		let file = File::open(&zip_path)?;

		let mut zip_archive = ZipArchive::new(file)?;

		if let Some(root) = &latest_version.root {
			let unzip_path = downloads_path.join(mod_id);
			files::extract(&mut zip_archive, &unzip_path)?;
			files::copy_dir_all(unzip_path.join(root), &target_path)?;
		} else {
			files::extract(&mut zip_archive, &target_path)?;
		}

		fs::write(
			local_mod::get_manifest_path(&target_path),
			serde_json::to_string_pretty(&mod_manifest::Manifest {
				title: Some(remote_mod.data.title.clone()),
				version: latest_version.id.clone(),
				runnable: latest_version.runnable.clone(),
				engine: remote_mod.common.engine,
				architecture: remote_mod.common.architecture,
				engine_version_range: remote_mod.common.engine_version_range.clone(),
				unity_backend: remote_mod.common.unity_backend,
				config: remote_mod.data.config.clone(),
			})?,
		)?;

		return Ok(());
	}
	Err(Error::ModDownloadNotAvailable(remote_mod.common.id.clone()))
}

pub async fn get_all<F>(error_handler: F) -> HashMap<String, RemoteMod>
where
	F: Fn(Error) + Send,
{
	let database = mod_database::get().await.unwrap_or_else(|error| {
		error_handler(error);
		ModDatabase { mods: Vec::new() }
	});

	let mut mods_map = HashMap::new();
	let local_mods = local_mod::get_all().ok_or_log("Failed to get local mods");

	for database_mod in database.mods {
		let remote_mod = RemoteMod {
			common: CommonModData {
				id: database_mod.id.clone(),
				engine: database_mod.engine,
				architecture: database_mod.architecture,
				engine_version_range: database_mod.engine_version_range.clone(),
				unity_backend: database_mod.unity_backend,
			},
			data: RemoteModData {
				author: database_mod.author.clone(),
				description: database_mod.description.clone(),
				source_code: database_mod.source_code.clone(),
				title: database_mod.title.clone(),
				latest_version: database_mod.get_download(),
				deprecated: database_mod.deprecated.unwrap_or(false),
				config: database_mod.config.clone(),
				dependencies: database_mod.dependencies.clone(),
			},
		};

		// If there's a local mod with the same ID, update its manifest with remote info
		if let Some(local_mod) = local_mods
			.as_ref()
			.and_then(|local_mods| local_mods.get(&database_mod.id))
			&& let Some(latest_version) = &remote_mod.data.latest_version
		{
			let manifest_path = local_mod::get_manifest_path(&local_mod.data.path);

			// Only update if the manifest file exists (mod has been downloaded before)
			if manifest_path.exists() {
				let updated_manifest = mod_manifest::Manifest {
					title: Some(remote_mod.data.title.clone()),
					version: latest_version.id.clone(),
					runnable: latest_version.runnable.clone(),
					engine: remote_mod.common.engine,
					architecture: remote_mod.common.architecture,
					engine_version_range: remote_mod.common.engine_version_range.clone(),
					unity_backend: remote_mod.common.unity_backend,
					config: remote_mod.data.config.clone(),
				};

				if let Ok(manifest_contents) = serde_json::to_string_pretty(&updated_manifest) {
					let _ = fs::write(&manifest_path, manifest_contents);
				}
			}
		}

		mods_map.insert(database_mod.id.clone(), remote_mod);
	}

	mods_map
}
