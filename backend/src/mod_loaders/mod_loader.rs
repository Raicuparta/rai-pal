use std::{
	collections::HashMap,
	fs::{
		self,
		File,
	},
	path::{
		Path,
		PathBuf,
	},
};

use async_trait::async_trait;
use enum_dispatch::enum_dispatch;
use zip::ZipArchive;

use super::{
	bepinex::BepInEx,
	mod_database::{self,},
	unreal_vr::UnrealVr,
};
use crate::{
	files,
	game_mod::CommonModData,
	installed_game::InstalledGame,
	local_mod::{
		self,
		LocalMod,
		ModKind,
	},
	mod_loaders::mod_database::ModDatabase,
	paths,
	remote_mod::{
		RemoteMod,
		RemoteModData,
	},
	serializable_struct,
	Error,
	Result,
};

serializable_struct!(ModLoaderData {
	pub id: String,
	pub path: PathBuf,
	pub kind: ModKind,
});

#[enum_dispatch]
#[derive(Clone)]
pub enum ModLoader {
	BepInEx,
	UnrealVr,
}

#[async_trait]
#[enum_dispatch(ModLoader)]
pub trait ModLoaderActions {
	fn install(&self, game: &InstalledGame) -> Result;
	async fn install_mod(&self, game: &InstalledGame, game_mod: &LocalMod) -> Result;
	fn get_data(&self) -> &ModLoaderData;
	fn get_mod_path(&self, mod_data: &CommonModData) -> Result<PathBuf>;
	fn get_local_mods(&self) -> Result<HashMap<String, LocalMod>>;

	async fn get_remote_mods<F>(&self, error_handler: F) -> HashMap<String, RemoteMod>
	where
		F: Fn(Error) + Send,
	{
		let data = self.get_data();
		let id = &data.id;

		let database = mod_database::get(id).await.unwrap_or_else(|error| {
			error_handler(error);
			ModDatabase {
				mods: HashMap::new(),
			}
		});

		database
			.mods
			.into_iter()
			.map(|(mod_id, database_mod)| {
				(
					mod_id.clone(),
					RemoteMod {
						common: CommonModData {
							id: mod_id,
							engine: database_mod.engine,
							unity_backend: database_mod.unity_backend,
							loader_id: id.clone(),
						},
						data: RemoteModData {
							author: database_mod.author,
							description: database_mod.description,
							source_code: database_mod.source_code,
							title: database_mod.title,
							downloads: database_mod.downloads,
						},
					},
				)
			})
			.collect()
	}

	async fn download_mod(&self, remote_mod: &RemoteMod) -> Result {
		let target_path = self.get_mod_path(&remote_mod.common)?;
		let mod_loader_data = self.get_data();
		let mod_id = &remote_mod.common.id;
		let downloads_path = paths::installed_mods_path()?
			.join(&mod_loader_data.id)
			.join("downloads");

		if let Some(first_download) = remote_mod.data.downloads.first() {
			let response = reqwest::get(&first_download.url).await?;

			if response.status().is_success() {
				fs::create_dir_all(&downloads_path)?;

				let zip_path = downloads_path.join(format!("{}.zip", mod_id));

				// TODO Stream to disk instead of keeping it all in memory.
				fs::write(&zip_path, response.bytes().await?)?;
				let file = File::open(&zip_path)?;

				let mut zip_archive = ZipArchive::new(file)?;

				if let Some(root) = &first_download.root {
					let unzip_path = downloads_path.join(mod_id);
					zip_archive.extract(downloads_path.join(mod_id))?;
					files::copy_dir_all(unzip_path.join(root), &target_path)?;
				} else {
					zip_archive.extract(&target_path)?;
				}

				// Saves the manifest so we know which version of the mod we installed.
				fs::write(
					local_mod::get_manifest_path(&target_path),
					serde_json::to_string_pretty(&local_mod::Manifest {
						version: first_download.version.clone(),
					})?,
				)?;

				Ok(())
			} else {
				Err(Error::ModNotFound(remote_mod.common.id.to_string())) // TODO error
			}
		} else {
			Err(Error::ModNotFound(remote_mod.common.id.to_string())) // TODO error
		}
	}
}

#[async_trait]
pub trait ModLoaderStatic {
	const ID: &'static str;

	async fn new(resources_path: &Path) -> Result<Self>
	where
		Self: Sized;

	fn get_installed_mods_path() -> Result<PathBuf> {
		Ok(paths::installed_mods_path()?.join(Self::ID).join("mods"))
	}
}

pub type Map = HashMap<String, ModLoader>;
pub type DataMap = HashMap<String, ModLoaderData>;

async fn create_map_entry<TModLoader: ModLoaderActions + ModLoaderStatic>(
	path: &Path,
) -> Result<(String, ModLoader)>
where
	ModLoader: std::convert::From<TModLoader>,
{
	let mod_loader: ModLoader = TModLoader::new(path).await?.into();

	Ok((TModLoader::ID.to_string(), mod_loader))
}

async fn add_entry<TModLoader: ModLoaderActions + ModLoaderStatic>(path: &Path, map: &mut Map)
where
	ModLoader: std::convert::From<TModLoader>,
{
	match create_map_entry::<TModLoader>(path).await {
		Ok((key, value)) => {
			map.insert(key, value);
		}
		Err(err) => eprintln!("Failed to create map entry: {err}"),
	}
}

pub async fn get_map(resources_path: &Path) -> Map {
	let mut map = Map::new();

	add_entry::<BepInEx>(resources_path, &mut map).await;
	add_entry::<UnrealVr>(resources_path, &mut map).await;

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
