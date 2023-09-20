use super::bepinex::BepInEx;
use super::melon_loader::MelonLoader;
use crate::game::Game;
use crate::game_mod::Mod;
use crate::{serializable_struct, Result};
use anyhow::anyhow;
use enum_dispatch::enum_dispatch;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

serializable_struct!(ModLoaderData {
    pub id: String,
    pub mod_count: u32,
    pub path: PathBuf,
    pub mods: Vec<Mod>,
});

#[enum_dispatch]
pub enum ModLoaderType {
    BepInEx,
    MelonLoader,
}

pub trait ModLoaderStatic {
    fn new(resources_path: &Path) -> Result<Self>
    where
        Self: Sized;
}

#[enum_dispatch(ModLoaderType)]
pub trait ModLoader {
    fn install(&self, game: &Game) -> Result;

    fn install_mod(&self, game: &Game, mod_id: String) -> Result;

    fn open_mod_folder(&self, mod_id: String) -> Result;

    fn get_data(&self) -> ModLoaderData;
}

pub trait ModLoaderId {
    const ID: &'static str;
}

type Map = HashMap<String, ModLoaderType>;
pub type DataMap = HashMap<String, ModLoaderData>;

fn create_map_entry<TModLoader: ModLoader + ModLoaderId + ModLoaderStatic + 'static>(
    path: &Path,
) -> Result<(String, ModLoaderType)>
where
    ModLoaderType: std::convert::From<TModLoader>,
{
    let mod_loader: ModLoaderType = TModLoader::new(path)?.into();

    Ok((TModLoader::ID.to_string(), mod_loader))
}

fn add_entry<TModLoader: ModLoader + ModLoaderId + ModLoaderStatic + 'static>(
    path: &Path,
    map: &mut Map,
) where
    ModLoaderType: std::convert::From<TModLoader>,
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

pub fn get(resources_path: &Path, id: &str) -> Result<ModLoaderType> {
    get_map(resources_path)
        .remove(id)
        .ok_or_else(|| anyhow!("Failed to find mod loader with id {id}"))
}

pub async fn get_data_map(app_handle: tauri::AppHandle) -> Result<DataMap> {
    let resources_path = app_handle
        .path_resolver()
        .resolve_resource("resources")
        .ok_or_else(|| anyhow!("Failed to find resources path"))?;

    get_map(&resources_path)
        .values()
        .map(|mod_loader| {
            let data = mod_loader.get_data();
            Ok((data.id.clone(), data))
        })
        .collect()
}
