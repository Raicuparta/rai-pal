use crate::game::Game;
use crate::Result;
use crate::{game::UnityScriptingBackend, r#mod::Mod};
use std::path::Path;

pub trait ModLoader {
    fn get_mods(path: &Path, scripting_backend: UnityScriptingBackend) -> Result<Vec<Mod>>;

    fn install(&self, game: &Game) -> Result;

    fn install_mod(&self, game: &Game, mod_id: String) -> Result;

    fn open_mod_folder(&self, mod_id: String) -> Result;
}
