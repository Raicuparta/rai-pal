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

fn create_map_entry<TModLoader: ModLoader + ModLoaderId + 'static>(
    path: &Path,
) -> Result<(String, Box<dyn ModLoader>)> {
    let mod_loader = TModLoader::new(path)?;
    let boxed: Box<dyn ModLoader> = Box::new(mod_loader);

    Ok((TModLoader::ID.to_string(), boxed))
}

fn get_all(resources_path: &Path) -> Result<HashMap<String, Box<dyn ModLoader>>> {
    Ok(HashMap::from([
        create_map_entry::<BepInEx>(resources_path)?,
        create_map_entry::<MelonLoader>(resources_path)?,
    ]))
}

pub fn find(resources_path: &Path, id: &str) -> Result<Box<dyn ModLoader>> {
    let mut mod_loaders = get_all(resources_path)?;

    mod_loaders
        .remove(id)
        .ok_or_else(|| anyhow!("Failed to find mod loader with id {id}"))
}

pub async fn get_all_data(app_handle: tauri::AppHandle) -> Result<HashMap<String, ModLoaderData>> {
    let resources_path = app_handle
        .path_resolver()
        .resolve_resource("resources")
        .ok_or_else(|| anyhow!("Failed to find resources path"))?;

    get_all(&resources_path)?
        .values()
        .map(|mod_loader| {
            let data = mod_loader.get_data();
            Ok((data.id.clone(), data))
        })
        .collect()
}
