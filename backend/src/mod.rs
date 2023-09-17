use crate::files::copy_dir_all;
use crate::game::{Game, UnityScriptingBackend};
use crate::Result;
use anyhow::anyhow;
use serde::Serialize;
use specta::Type;
use std::path::{Path, PathBuf};

#[derive(Serialize, Type, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Mod {
    pub id: String,
    pub name: String,
    pub scripting_backend: UnityScriptingBackend,
    path: PathBuf,
}

impl Mod {
    pub fn new(path: &Path, scripting_backend: &UnityScriptingBackend) -> Result<Self> {
        let name = String::from(
            path.file_name()
                .ok_or_else(|| anyhow!("Failed to get file name"))?
                .to_string_lossy(),
        );

        Ok(Self {
            id: format!("{scripting_backend}/{name}"),
            path: path.to_path_buf(),
            name,
            scripting_backend: scripting_backend.clone(),
        })
    }

    pub fn install(&self, game: &Game) -> Result {
        copy_dir_all(
            &self.path,
            game.get_data_folder()?
                .join("BepInEx")
                .join("plugins")
                .join(self.name.as_str()),
        )
        .map_err(|err| anyhow!("Failed to install mod: {err}"))
    }

    pub fn open_folder(&self) -> Result {
        open::that_detached(&self.path).map_err(|err| anyhow!("Failed to open mod folder: {err}"))
    }
}
