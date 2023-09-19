use super::bepinex::BepInEx;
use super::melon_loader::MelonLoader;
use crate::game::Game;
use crate::game_mod::Mod;
use crate::{serializable_struct, Result};
use anyhow::anyhow;
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

fn get_all(resources_path: &Path) -> Result<Vec<Box<dyn ModLoader>>> {
    Ok(vec![
        Box::new(BepInEx::new(resources_path)?),
        Box::new(MelonLoader::new(resources_path)?),
    ])
}

pub fn find(resources_path: &Path, id: &str) -> Result<Box<dyn ModLoader>> {
    let mod_loaders = get_all(resources_path)?;

    mod_loaders
        .into_iter()
        .find(|mod_loader| mod_loader.get_data().id == id)
        .ok_or_else(|| anyhow!("Failed to find mod loader with id {id}"))
}

pub async fn get_all_data(app_handle: tauri::AppHandle) -> Result<Vec<ModLoaderData>> {
    let resources_path = app_handle
        .path_resolver()
        .resolve_resource("resources")
        .ok_or_else(|| anyhow!("Failed to find resources path"))?;

    get_all(&resources_path)?
        .into_iter()
        .map(|mod_loader| Ok(mod_loader.get_data()))
        .collect()
}
