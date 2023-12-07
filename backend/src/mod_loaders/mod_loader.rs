use std::{
	collections::HashMap,
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
	unreal_vr::UnrealVr,
};
use crate::{
	game_mod::GameMod,
	installed_game::InstalledGame,
	serializable_struct,
	Error,
	Result,
};

serializable_struct!(ModLoaderData {
	pub id: String,
	pub path: PathBuf,
	pub mods: HashMap<String, GameMod>,
});

#[enum_dispatch]
pub enum ModLoader {
	BepInEx,
	MelonLoader,
	UnrealVr,
}

#[async_trait]
#[enum_dispatch(ModLoader)]
pub trait ModLoaderActions {
	fn install(&self, game: &InstalledGame) -> Result;
	fn install_mod(&self, game: &InstalledGame, mod_id: &str) -> Result;
	fn open_mod_folder(&self, mod_id: &str) -> Result;
	fn get_data(&self) -> &ModLoaderData;
	fn get_mod_path(&self, mod_id: &str) -> Result<PathBuf>;

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
						let bytes = response.bytes().await?;
						let cursor = Cursor::new(bytes);

						ZipArchive::new(cursor)?.extract(target_path)?;

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

type Map = HashMap<String, ModLoader>;
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

async fn get_map(resources_path: &Path) -> Map {
	let mut map = Map::new();

	add_entry::<BepInEx>(resources_path, &mut map).await;
	add_entry::<MelonLoader>(resources_path, &mut map).await;
	add_entry::<UnrealVr>(resources_path, &mut map).await;

	map
}

pub async fn get(resources_path: &Path, id: &str) -> Result<ModLoader> {
	get_map(resources_path)
		.await
		.remove(id)
		.ok_or_else(|| Error::ModLoaderNotFound(id.to_string()))
}

pub async fn get_data_map(resources_path: &Path) -> Result<DataMap> {
	get_map(resources_path)
		.await
		.values()
		.map(|mod_loader| {
			let data = mod_loader.get_data();
			Ok((data.id.clone(), data.clone()))
		})
		.collect()
}
