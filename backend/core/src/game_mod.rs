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
	architecture::Architecture,
	files,
	game_engines::{
		game_engine::{
			EngineBrand,
			EngineVersionNumbers,
		},
		unity::UnityBackend,
	},
	local_mod::{
		self,
		LocalMod,
	},
	mod_loaders::{
		mod_database::{
			self,
			ModDatabase,
		},
		mod_loader::ModLoaderId,
	},
	mod_manifest,
	paths,
	remote_mod::{
		RemoteMod,
		RemoteModData,
	},
	result::{
		Error,
		Result,
	},
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

// TODO move some of these to local_mod and remote_mod.

pub fn get_local() -> Result<HashMap<String, LocalMod>> {
	Ok(paths::glob_path(
		&paths::local_mods_path()?
			.join("*")
			.join(mod_manifest::Manifest::FILE_NAME),
	)
	.iter()
	.filter_map(|manifest_path| {
		let local_mod = LocalMod::new(manifest_path).ok(); // TODO don't swalloow error.
		local_mod.map(|m| (m.common.id.clone(), m))
	})
	.collect())
}

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

		// Saves the manifest so we know which version of the mod we installed.
		fs::write(
			local_mod::get_manifest_path(&target_path),
			serde_json::to_string_pretty(&mod_manifest::Manifest {
				title: Some(remote_mod.data.title.clone()),
				is_loader: remote_mod.common.is_loader,
				loader_id: remote_mod.common.loader_id,
				version: latest_version.id.clone(),
				runnable: latest_version.runnable.clone(),
				engine: remote_mod.common.engine,
				architecture: remote_mod.common.architecture,
				engine_version_range: remote_mod.common.engine_version_range.clone(),
				unity_backend: remote_mod.common.unity_backend,
				configs: remote_mod.data.configs.clone(),
			})?,
		)?;

		return Ok(());
	}
	Err(Error::ModDownloadNotAvailable(remote_mod.common.id.clone()))
}

pub fn delete(local_mod: &LocalMod) -> Result {
	if local_mod.data.path.exists() {
		fs::remove_dir_all(&local_mod.data.path)?;
	}

	Ok(())
}

pub async fn get_remote<F>(error_handler: F) -> HashMap<String, RemoteMod>
where
	F: Fn(Error) + Send,
{
	let database = mod_database::get().await.unwrap_or_else(|error| {
		error_handler(error);
		ModDatabase { mods: Vec::new() }
	});

	let mut mods_map = HashMap::new();

	for database_mod in database.mods {
		let remote_mod = RemoteMod {
			common: CommonModData {
				id: database_mod.id.clone(),
				is_loader: database_mod.is_loader,
				engine: database_mod.engine,
				architecture: database_mod.architecture,
				engine_version_range: database_mod.engine_version_range.clone(),
				unity_backend: database_mod.unity_backend,
				loader_id: database_mod.loader_id,
			},
			data: RemoteModData {
				author: database_mod.author.clone(),
				description: database_mod.description.clone(),
				source_code: database_mod.source_code.clone(),
				title: database_mod.title.clone(),
				latest_version: database_mod.get_download().await,
				deprecated: database_mod.deprecated.unwrap_or(false),
				configs: database_mod.configs.clone(),
			},
		};

		// If there's a local mod with the same ID, update its manifest with remote info
		if let Some(local_mod) = get_local()
			.ok()
			.as_ref()
			.and_then(|local_mods| local_mods.get(&database_mod.id))
			&& let Some(latest_version) = &remote_mod.data.latest_version
		{
			let manifest_path = local_mod::get_manifest_path(&local_mod.data.path);

			// Only update if the manifest file exists (mod has been downloaded before)
			if manifest_path.exists() {
				let updated_manifest = mod_manifest::Manifest {
					loader_id: remote_mod.common.loader_id,
					title: Some(remote_mod.data.title.clone()),
					is_loader: remote_mod.common.is_loader,
					version: latest_version.id.clone(),
					runnable: latest_version.runnable.clone(),
					engine: remote_mod.common.engine,
					architecture: remote_mod.common.architecture,
					engine_version_range: remote_mod.common.engine_version_range.clone(),
					unity_backend: remote_mod.common.unity_backend,
					configs: remote_mod.data.configs.clone(),
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
