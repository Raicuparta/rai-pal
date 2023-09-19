use crate::game::Game;
use crate::game_mod::Mod;
use crate::{serializable_struct, Result};
use anyhow::anyhow;
use std::path::{Path, PathBuf};

use super::bepinex;

serializable_struct!(ModLoaderData {
    pub id: String,
    pub mod_count: u32,
    pub path: PathBuf,
    pub mods: Vec<Mod>,
});

pub trait ModLoaderID {
    const ID: &'static str;
}

pub trait ModLoader {
    fn new(path: &Path) -> Result<Self>
    where
        Self: std::marker::Sized;

    fn install(&self, game: &Game) -> Result;

    fn install_mod(&self, game: &Game, mod_id: String) -> Result;

    fn open_mod_folder(&self, mod_id: String) -> Result;

    fn get_id(&self) -> String;
}

pub fn get<TModLoader: ModLoader + ModLoaderID>(resources_path: &Path) -> Result<TModLoader> {
    TModLoader::new(&resources_path.join(TModLoader::ID))
}

pub fn get_all(resources_path: &Path) -> Result<Vec<Box<dyn ModLoader>>> {
    Ok(vec![Box::new(get::<bepinex::BepInEx>(resources_path)?)])
}

pub fn find(resources_path: &Path, id: &str) -> Result<Box<dyn ModLoader>> {
    let mod_loaders = get_all(resources_path)?;

    mod_loaders
        .into_iter()
        .find(|mod_loader| mod_loader.get_id() == id)
        .ok_or_else(|| anyhow!("Failed to find mod loader with id {id}"))
}
