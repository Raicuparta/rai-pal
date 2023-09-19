use super::bepinex::BepInEx;
use super::melon_loader::MelonLoader;
use crate::game::Game;
use crate::game_mod::Mod;
use crate::{serializable_struct, Result};
use anyhow::anyhow;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

serializable_struct!(ModLoaderData {
    pub id: String,
    pub mod_count: u32,
    pub path: PathBuf,
    pub mods: Vec<Mod>,
});

pub trait ModLoader {
    fn new(path: &Path) -> Result<Self>
    where
        Self: std::marker::Sized;

    fn install(&self, game: &Game) -> Result;

    fn install_mod(&self, game: &Game, mod_id: String) -> Result;

    fn open_mod_folder(&self, mod_id: String) -> Result;

    fn get_data(&self) -> ModLoaderData;
}

pub trait ModLoaderId {
    const ID: &'static str;
}

type Map = HashMap<String, Box<dyn ModLoader>>;
pub type DataMap = HashMap<String, ModLoaderData>;

fn create_map_entry<TModLoader: ModLoader + ModLoaderId + 'static>(
    path: &Path,
) -> Result<(String, Box<dyn ModLoader>)> {
    let mod_loader = TModLoader::new(path)?;
    let boxed: Box<dyn ModLoader> = Box::new(mod_loader);

    Ok((TModLoader::ID.to_string(), boxed))
}

fn add_entry<TModLoader: ModLoader + ModLoaderId + 'static>(path: &Path, map: &mut Map) {
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

pub fn find(resources_path: &Path, id: &str) -> Result<Box<dyn ModLoader>> {
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
