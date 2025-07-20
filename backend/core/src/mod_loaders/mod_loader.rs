use std::{
	collections::HashMap,
	fs::{self, File},
	path::{Path, PathBuf},
};

use enum_dispatch::enum_dispatch;
use log::error;
use rai_pal_proc_macros::serializable_struct;
use zip::ZipArchive;

use super::{bepinex::BepInEx, mod_database, runnable_loader::RunnableLoader};
use crate::{
	files,
	game::DbGame,
	game_mod::CommonModData,
	http_client,
	local_mod::{self, LocalMod, ModKind},
	mod_loaders::mod_database::{ModConfigDestinationType, ModConfigs, ModDatabase},
	mod_manifest,
	paths::{self, open_folder_or_parent},
	remote_config,
	remote_mod::{RemoteMod, RemoteModData},
	result::{Error, Result},
};

#[serializable_struct]
pub struct ModLoaderData {
	pub id: String,
	pub path: PathBuf,
	pub kind: ModKind,
}

#[enum_dispatch]
#[derive(Clone)]
pub enum ModLoader {
	BepInEx,
	RunnableLoader,
}

#[enum_dispatch(ModLoader)]
pub trait ModLoaderActions {
	fn install(&self, game: &DbGame) -> Result;
	async fn install_mod_inner(&self, game: &DbGame, local_mod: &LocalMod) -> Result;
	async fn uninstall_mod(&self, game: &DbGame, local_mod: &LocalMod) -> Result;
	async fn run_without_game(&self, local_mod: &LocalMod) -> Result;
	fn get_config_path(&self, game: &DbGame, mod_configs: &ModConfigs) -> Result<PathBuf>;
	fn open_installed_mod_folder(&self, game: &DbGame, local_mod: &LocalMod) -> Result;
	fn get_data(&self) -> &ModLoaderData;
	fn get_mod_path(&self, mod_data: &CommonModData) -> Result<PathBuf>;
	fn get_local_mods(&self) -> Result<HashMap<String, LocalMod>>;

	fn open_folder(&self) -> Result {
		open_folder_or_parent(&self.get_data().path)
	}

	async fn install_mod(&self, game: &DbGame, local_mod: &LocalMod) -> Result {
		self.install_mod_inner(game, local_mod).await?;

		if self.get_data().kind != ModKind::Runnable {
			if let Some(manifest) = &local_mod.data.manifest {
				let manifest_path = game.get_installed_mod_manifest_path(&local_mod.common.id)?;
				fs::create_dir_all(paths::path_parent(&manifest_path)?)?;
				let manifest_contents = serde_json::to_string_pretty(manifest)?;
				fs::write(manifest_path, manifest_contents)?;
			}
		}

		Ok(())
	}

	async fn get_remote_mods<F>(&self, error_handler: F) -> HashMap<String, RemoteMod>
	where
		F: Fn(Error) + Send,
	{
		let data = self.get_data();
		let loader_id = &data.id;

		let database = mod_database::get(loader_id).await.unwrap_or_else(|error| {
			error_handler(error);
			ModDatabase { mods: Vec::new() }
		});

		let mut mods_map = HashMap::new();

		for database_mod in database.mods {
			let remote_mod = RemoteMod {
				common: CommonModData {
					id: database_mod.id.clone(),
					engine: database_mod.engine,
					engine_version_range: database_mod.engine_version_range.clone(),
					unity_backend: database_mod.unity_backend,
					loader_id: loader_id.clone(),
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
			mods_map.insert(database_mod.id.clone(), remote_mod);
		}

		mods_map
	}

	async fn download_mod(&self, remote_mod: &RemoteMod) -> Result {
		if let Some(latest_version) = &remote_mod.data.latest_version {
			let target_path = self.get_mod_path(&remote_mod.common)?;
			let mod_loader_data = self.get_data();
			let mod_id = &remote_mod.common.id;
			let downloads_path = paths::installed_mods_path()?
				.join(&mod_loader_data.id)
				.join("downloads");

			let client = http_client::get_client();
			let response = client.get(&latest_version.url).send().await?;

			fs::create_dir_all(&downloads_path)?;

			let zip_path = downloads_path.join(format!("{mod_id}.zip"));

			// TODO Stream to disk instead of keeping it all in memory.
			fs::write(&zip_path, response.bytes().await?)?;
			let file = File::open(&zip_path)?;

			let mut zip_archive = ZipArchive::new(file)?;

			if let Some(root) = &latest_version.root {
				let unzip_path = downloads_path.join(mod_id);
				zip_archive.extract(downloads_path.join(mod_id))?;
				files::copy_dir_all(unzip_path.join(root), &target_path)?;
			} else {
				zip_archive.extract(&target_path)?;
			}

			// Saves the manifest so we know which version of the mod we installed.
			fs::write(
				local_mod::get_manifest_path(&target_path),
				serde_json::to_string_pretty(&mod_manifest::Manifest {
					title: Some(remote_mod.data.title.clone()),
					version: latest_version.id.clone(),
					runnable: latest_version.runnable.clone(),
					engine: remote_mod.common.engine,
					engine_version_range: remote_mod.common.engine_version_range.clone(),
					unity_backend: remote_mod.common.unity_backend,
				})?,
			)?;

			return Ok(());
		}
		Err(Error::ModDownloadNotAvailable(remote_mod.common.id.clone()))
	}

	fn delete_mod(&self, local_mod: &LocalMod) -> Result {
		let mod_path = self.get_mod_path(&local_mod.common)?;

		if mod_path.exists() {
			fs::remove_dir_all(&mod_path)?;
		}

		Ok(())
	}

	async fn download_config(
		&self,
		game: &DbGame,
		mod_configs: &ModConfigs,
		config_file: &str,
		overwrite: bool,
	) -> Result {
		let destination_path = self.get_config_path(game, mod_configs)?;

		if destination_path.try_exists()? {
			if overwrite {
				if destination_path.is_dir() {
					fs::remove_dir_all(&destination_path)?;
				} else {
					fs::remove_file(&destination_path)?;
				}
			} else {
				return Ok(());
			}
		}

		if let Some(parent) = destination_path.parent() {
			fs::create_dir_all(parent)?;
		}

		match mod_configs.destination_type {
			ModConfigDestinationType::File => {
				remote_config::download_config_file(config_file, game, &destination_path).await?;
			}
			ModConfigDestinationType::Folder => {
				todo!();
			}
		}

		Ok(())
	}

	fn configure_mod(&self, game: &DbGame, remote_mod: &RemoteMod) -> Result {
		// TODO configs should be saved in local mod manifest? otherwise bad!
		if let Some(configs) = remote_mod.data.configs.as_ref() {
			let config_path = self.get_config_path(game, configs)?;
			open::that_detached(config_path)?;
		}
		// TODO maybe error if no configs.

		Ok(())
	}
}

pub trait ModLoaderStatic {
	const ID: &'static str;

	fn new(resources_path: &Path) -> Result<Self>
	where
		Self: Sized;

	fn get_installed_mods_path() -> Result<PathBuf> {
		Ok(paths::installed_mods_path()?.join(Self::ID).join("mods"))
	}
}

pub type Map = HashMap<String, ModLoader>;
pub type DataMap = HashMap<String, ModLoaderData>;

fn create_map_entry<TModLoader: ModLoaderActions + ModLoaderStatic>(
	path: &Path,
) -> Result<(String, ModLoader)>
where
	ModLoader: std::convert::From<TModLoader>,
{
	let mod_loader: ModLoader = TModLoader::new(path)?.into();

	Ok((TModLoader::ID.to_string(), mod_loader))
}

fn add_entry<TModLoader: ModLoaderActions + ModLoaderStatic>(path: &Path, map: &mut Map)
where
	ModLoader: std::convert::From<TModLoader>,
{
	match create_map_entry::<TModLoader>(path) {
		Ok((key, value)) => {
			map.insert(key, value);
		}
		Err(err) => error!("Failed to create map entry: {err}"),
	}
}

pub fn get_map(resources_path: &Path) -> Map {
	let mut map = Map::new();

	add_entry::<BepInEx>(resources_path, &mut map);
	add_entry::<RunnableLoader>(resources_path, &mut map);

	map
}

pub fn get_data_map(map: &Map) -> Result<DataMap> {
	map.values()
		.map(|mod_loader| {
			let data = mod_loader.get_data();
			Ok((data.id.clone(), data.clone()))
		})
		.collect()
}
