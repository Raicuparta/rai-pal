use std::path::{Path, PathBuf};

use anyhow::anyhow;
use directories::ProjectDirs;
use serde::Serialize;
use specta::Type;

use crate::game::{Game, UnityScriptingBackend};
use crate::Result;

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
                .ok_or(anyhow!("Failed to get file name"))?
                .to_string_lossy(),
        );

        Ok(Mod {
            id: format!("{scripting_backend}/{name}"),
            path: path.to_path_buf(),
            name,
            scripting_backend: scripting_backend.clone(),
        })
    }

    pub fn install(&self, game: &Game) -> Result {
        let project_dirs = ProjectDirs::from("com", "raicuparta", "pal")
            .ok_or_else(|| anyhow!("Failed to get user data folders"))?;

        let game_mods_folder = project_dirs.data_dir().join("mods");

        // let installed_mod_folder = game_mods_folder.join(path);

        todo!()
    }

    pub fn open_folder(&self) -> Result {
        open::that_detached(&self.path).map_err(|err| anyhow!("Failed to open mod folder: {err}"))
    }
}
