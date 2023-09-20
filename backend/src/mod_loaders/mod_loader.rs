use std::{
	collections::HashMap,
	path::{
		Path,
		PathBuf,
	},
};

use enum_dispatch::enum_dispatch;

use super::{
	bepinex::BepInEx,
	melon_loader::MelonLoader,
};
use crate::{
	game::Game,
	game_mod::Mod,
	paths,
	serializable_struct,
	Error,
	Result,
};

serializable_struct!(ModLoaderData {
	pub id: String,
	pub path: PathBuf,
	pub mods: Vec<Mod>,
});

#[enum_dispatch]
pub enum ModLoader {
	BepInEx,
	MelonLoader,
}

#[enum_dispatch(ModLoader)]
pub trait ModLoaderActions {
	fn install(&self, game: &Game) -> Result;
	fn install_mod(&self, game: &Game, mod_id: String) -> Result;
	fn open_mod_folder(&self, mod_id: String) -> Result;
	fn get_data(&self) -> ModLoaderData;
}

pub trait ModLoaderStatic {
	const ID: &'static str;

	fn new(resources_path: &Path) -> Result<Self>
	where
		Self: Sized;
}

type Map = HashMap<String, ModLoader>;
pub type DataMap = HashMap<String, ModLoaderData>;

fn create_map_entry<TModLoader: ModLoaderActions + ModLoaderStatic + 'static>(
	path: &Path,
) -> Result<(String, ModLoader)>
where
	ModLoader: std::convert::From<TModLoader>,
{
	let mod_loader: ModLoader = TModLoader::new(path)?.into();

	Ok((TModLoader::ID.to_string(), mod_loader))
}

fn add_entry<TModLoader: ModLoaderActions + ModLoaderStatic + 'static>(path: &Path, map: &mut Map)
where
	ModLoader: std::convert::From<TModLoader>,
{
	match create_map_entry::<TModLoader>(path) {
		Ok((key, value)) => {
			map.insert(key, value);
		}
		Err(err) => println!("Failed to create map entry: {err}"),
	}
}

fn get_map(resources_path: &Path) -> Map {
	let mut map = Map::new();

	add_entry::<BepInEx>(resources_path, &mut map);
	add_entry::<MelonLoader>(resources_path, &mut map);

	map
}

pub fn get(resources_path: &Path, id: &str) -> Result<ModLoader> {
	get_map(resources_path)
		.remove(id)
		.ok_or_else(|| Error::ModLoaderNotFound(id.to_string()))
}

pub async fn get_data_map(app_handle: tauri::AppHandle) -> Result<DataMap> {
	get_map(&paths::resources_path(&app_handle)?)
		.values()
		.map(|mod_loader| {
			let data = mod_loader.get_data();
			Ok((data.id.clone(), data))
		})
		.collect()
}
