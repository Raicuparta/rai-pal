use anyhow::anyhow;
use glob::glob;
use serde::Serialize;
use specta::Type;
use std::fs;
use std::path::{Path, PathBuf};

use crate::files::copy_dir_all;
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
        let mods_folder_path = path.join(scripting_backend.to_string()).join("mods");

        let entries: Vec<_> = glob(
            mods_folder_path
                .join("*")
                .to_str()
                .ok_or(anyhow!("Failed to parse mods folder path"))?,
        )
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
        let scripting_backend_path = &self.path.join(game.scripting_backend.to_string());
        let architecture_path = scripting_backend_path
            .join(game.operating_system.to_string())
            .join(game.architecture.to_string());

        let mod_files_folder = architecture_path.join("mod-files");
        let copy_to_game_folder = architecture_path.join("copy-to-game");

        let game_data_folder = &game.get_data_folder()?;

        copy_dir_all(mod_files_folder, game_data_folder)
            .map_err(|err| anyhow!("Failed to copy mod loader files: {err}"))?;

        let game_folder = game
            .full_path
            .parent()
            .ok_or(anyhow!("Failed to get game parent folder"))?;

        copy_dir_all(copy_to_game_folder, game_folder)?;

        let config_origin_path = &self.path.join("config").join(if game.is_legacy {
            "BepInEx-legacy.cfg"
        } else {
            "BepInEx.cfg"
        });

        let config_target_folder = game_data_folder.join("BepInEx").join("config");

        fs::create_dir_all(&config_target_folder)?;

        fs::copy(config_origin_path, config_target_folder.join("BepInEx.cfg"))?;

        let doorstop_config =
            fs::read_to_string(scripting_backend_path.join("doorstop_config.ini"))?;

        fs::write(
            game_folder.join("doorstop_config.ini"),
            doorstop_config.replace(
                "{{MOD_FILES_PATH}}",
                game_data_folder
                    .to_str()
                    .ok_or(anyhow!("Failed to parse game data folder"))?,
            ),
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
