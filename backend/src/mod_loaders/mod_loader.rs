use std::{
	collections::{
		HashMap,
		HashSet,
	},
	fs,
	io::Cursor,
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
	melon_loader::MelonLoader,
	mod_database,
	unreal_vr::UnrealVr,
};
use crate::{
	game_mod::GameMod,
	installed_game::InstalledGame,
	local_mod::{
		self,
		ModKind,
	},
	mod_loaders::mod_database::ModDatabase,
	serializable_struct,
	Error,
	Result,
};

serializable_struct!(ModLoaderData {
	pub id: String,
	pub path: PathBuf,
	pub mods: HashMap<String, GameMod>,
	pub kind: ModKind,
});

#[enum_dispatch]
#[derive(Clone)]
pub enum ModLoader {
	BepInEx,
	MelonLoader,
	UnrealVr,
}

#[async_trait]
#[enum_dispatch(ModLoader)]
pub trait ModLoaderActions {
	fn install(&self, game: &InstalledGame) -> Result;
	async fn install_mod(&self, game: &InstalledGame, mod_id: &str) -> Result;
	fn open_mod_folder(&self, mod_id: &str) -> Result;
	fn get_data(&self) -> &ModLoaderData;
	fn get_data_mut(&mut self) -> &mut ModLoaderData;
	fn get_mod_path(&self, mod_id: &str) -> Result<PathBuf>;

	async fn update_remote_mods<F>(&mut self, error_handler: F)
	where
		F: Fn(Error) + Send,
	{
		let id = "bepinex"; // TODO get actual ID.

		let database = mod_database::get(id).await.unwrap_or_else(|error| {
			error_handler(error);
			ModDatabase {
				mods: HashMap::new(),
			}
		});

		let game_mods = &self.get_data().mods;

		let keys: HashSet<_> = game_mods
			.keys()
			.chain(database.mods.keys())
			.cloned()
			.collect();

		self.get_data_mut().mods = keys
			.iter()
			.filter_map(|key| {
				let remote_mod = database.mods.get(key);
				let game_mod = game_mods.get(key);

				let common = remote_mod.map_or_else(
					|| game_mod.map(|local| local.common.clone()),
					|remote| Some(remote.common.clone()),
				)?;

				Some((
					key.clone(),
					GameMod {
						remote_mod: remote_mod.map(|m| m.data.clone()),
						local_mod: game_mod.and_then(|m| m.local_mod.clone()),
						common,
					},
				))
			})
			.collect();
	}

	async fn download_mod(&self, mod_id: &str) -> Result {
		let target_path = self.get_mod_path(mod_id)?;
		let data = self.get_data();
		let downloads_folder = data.path.join("downloads");
		fs::create_dir_all(&downloads_folder)?;

		if let Some(game_mod) = data.mods.get(mod_id) {
			if let Some(remote_mod) = &game_mod.remote_mod {
				if let Some(first_download) = remote_mod.downloads.first() {
					let response = reqwest::get(&first_download.url).await?;

					if response.status().is_success() {
						// This keeps the whole zip in memory and only copies the extracted part to disk.
						// If we ever need to support very big mods, we should stream the zip to disk first,
						// and extract it after it's written to disk.
						ZipArchive::new(Cursor::new(response.bytes().await?))?
							.extract(&target_path)?;

						// Saves the manifest so we know which version of the mod we installed.
						fs::write(
							local_mod::get_manifest_path(&target_path),
							serde_json::to_string_pretty(&local_mod::Manifest {
								version: first_download.version.clone(),
							})?,
						)?;

						Ok(())
					} else {
						Err(Error::ModNotFound(mod_id.to_string())) // TODO error
					}
				} else {
					Err(Error::ModNotFound(mod_id.to_string())) // TODO error
				}
			} else {
				Err(Error::ModNotFound(mod_id.to_string())) // TODO error
			}
		} else {
			Err(Error::ModNotFound(mod_id.to_string()))
		}
	}
}

#[async_trait]
pub trait ModLoaderStatic {
	const ID: &'static str;

	async fn new(resources_path: &Path) -> Result<Self>
	where
		Self: Sized;
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
	add_entry::<MelonLoader>(resources_path, &mut map).await;
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
