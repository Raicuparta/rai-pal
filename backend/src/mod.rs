use std::path::PathBuf;

use serde::Serialize;
use specta::Type;

use crate::game_executable::{GameExecutable, UnityScriptingBackend};

#[derive(Serialize, Type, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Mod {
    pub path: PathBuf,
    pub name: String,
    pub scripting_backend: UnityScriptingBackend,
}

impl Mod {
    fn install(&self, game_executable: &GameExecutable) {
        todo!()
    }
}
