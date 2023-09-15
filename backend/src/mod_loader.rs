use anyhow::anyhow;
use directories::ProjectDirs;
use glob::glob;
use serde::Serialize;
use specta::Type;
use std::path::{Path, PathBuf};
use std::{fs, io};

use crate::game::Game;
use crate::Result;
use crate::{game::UnityScriptingBackend, r#mod::Mod};

#[derive(Serialize, Type, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BepInEx {
    pub id: String,
    pub mod_count: u32,
    path: PathBuf,
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
            path: path.to_path_buf(),
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

    pub fn install(&self, game: &Game) -> Result {
        let project_dirs = ProjectDirs::from("com", "raicuparta", "pal")
            .ok_or_else(|| anyhow!("Failed to get user data folders"))?;

        let mod_loader_folder = &self
            .path
            .join(game.scripting_backend.to_string())
            .join(game.operating_system.to_string())
            .join(game.architecture.to_string());

        let mod_files_folder = mod_loader_folder.join("mod-files");
        let copy_to_game_folder = mod_loader_folder.join("copy-to-game");

        if let Some(thing) = mod_files_folder.to_str() {
            println!("folder?? {}", thing);
        }

        let game_mods_data_folder = project_dirs.data_dir().join("games").join(&game.id);

        copy_dir_all(mod_files_folder, game_mods_data_folder)
            .map_err(|err| anyhow!("Failed to copy mod loader files: {err}"))?;

        copy_dir_all(
            copy_to_game_folder,
            game.full_path
                .parent()
                .ok_or(anyhow!("Failed to get game parent folder"))?,
        )?;

        Ok(())
    }

    pub fn install_mod(&self, game: &Game, mod_id: String) -> Result {
        let game_mod = self
            .mods
            .iter()
            .find(|game_mod| game_mod.id == mod_id)
            .ok_or(anyhow!("Failed to find mod with id {mod_id}"))?;

        self.install(game)?;
        game_mod.install(game)
    }

    pub fn open_mod_folder(&self, mod_id: String) -> Result {
        let game_mod = self
            .mods
            .iter()
            .find(|game_mod| game_mod.id == mod_id)
            .ok_or(anyhow!("Failed to find mod with id {mod_id}"))?;

        game_mod.open_folder()
    }
}

fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> io::Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}
