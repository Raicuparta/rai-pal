use anyhow::anyhow;
use glob::glob;
use serde::Serialize;
use specta::Type;
use std::path::Path;

use crate::game_executable::GameExecutable;
use crate::Result;
use crate::{game_executable::UnityScriptingBackend, r#mod::Mod};

#[derive(Serialize, Type, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BepInEx {
    pub id: String,
    pub mod_count: u32,
    mods: Vec<Mod>,
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

        let entries: Vec<_> = glob(mods_folder_path.join("*").to_str().unwrap())
            .expect("Failed to glob")
            .collect();

        Ok(entries
            .iter()
            .filter_map(|entry| match entry {
                Ok(path) => Some(Mod::new(path, &scripting_backend).ok()?),
                Err(_) => None,
            })
            .collect())
    }

    pub fn install_mod(&self, game_executable: &GameExecutable, mod_id: String) -> Result {
        let game_mod = self
            .mods
            .iter()
            .find(|game_mod| game_mod.id == mod_id)
            .ok_or(anyhow!("Failed to find mod with id {mod_id}"))?;

        println!(
            "Will install mod {} on game {}",
            game_mod.name, game_executable.name
        );

        Ok(())
    }
}
