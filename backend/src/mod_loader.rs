use std::{
    collections::HashMap,
    fs::{canonicalize, File},
    path::{Path, PathBuf},
};

use anyhow::anyhow;
use directories::BaseDirs;
use glob::{glob, MatchOptions};
use serde::Serialize;
use specta::Type;

use crate::game_executable::{GameExecutable, UnityScriptingBackend};
use crate::Result;

#[derive(Serialize, Type, Clone)]
#[serde(rename_all = "camelCase")]
struct Mod {
    path: PathBuf,
    name: String,
    scripting_backend: UnityScriptingBackend,
}

#[derive(Serialize, Type, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BepInEx {
    id: String,
    mods: Vec<Mod>,
    pub mod_count: u32,
}

impl BepInEx {
    pub fn new(path: &Path) -> Result<Self> {
        let mut mods = Self::get_mods(path, UnityScriptingBackend::Il2Cpp)?;
        mods.append(&mut Self::get_mods(path, UnityScriptingBackend::Mono)?);
        let mod_count = mods.len();

        Ok(BepInEx {
            id: "BepInEx".to_owned(),
            mods,
            mod_count: u32::try_from(mod_count)?,
        })
    }

    fn get_mods(path: &Path, scripting_backend: UnityScriptingBackend) -> Result<Vec<Mod>> {
        let mods_folder_path = path
            .join(if scripting_backend == UnityScriptingBackend::Il2Cpp {
                "il2cpp"
            } else {
                "mono"
            })
            .join("mods");

        let entries: Vec<_> = glob(format!("{}/*", mods_folder_path.to_str().unwrap()).as_str())
            .expect("Failed to glob")
            .collect();

        Ok(entries
            .iter()
            .filter_map(|entry| match entry {
                Ok(path) => Some(Mod {
                    path: path.clone(),
                    name: String::from(path.file_name().unwrap().to_str().unwrap()),
                    scripting_backend: scripting_backend.clone(),
                }),
                Err(_) => None,
            })
            .collect())
    }
}
