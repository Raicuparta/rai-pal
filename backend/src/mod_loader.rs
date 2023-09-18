use crate::game::Game;
use crate::Result;
use crate::{game::UnityScriptingBackend, game_mod::Mod};
use std::path::Path;

pub trait ModLoader {
    const ID: &'static str;

    fn new(path: &Path) -> Result<Self>
    where
        Self: std::marker::Sized;

    fn get_mods(path: &Path, scripting_backend: UnityScriptingBackend) -> Result<Vec<Mod>>;

    fn install(&self, game: &Game) -> Result;

    fn install_mod(&self, game: &Game, mod_id: String) -> Result;

    fn open_mod_folder(&self, mod_id: String) -> Result;

    fn get_id(&self) -> String {
        Self::ID.to_string()
    }
}
