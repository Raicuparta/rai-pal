use appinfo::SteamLaunchOption;
use serde::Serialize;
use specta::Type;
use std::{fs, path::PathBuf};

use crate::appinfo;

#[derive(Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct GameExecutable {
    pub id: String,
    pub name: String,
    pub is_legacy: bool,
    pub mod_files_path: String,
    pub full_path: PathBuf,
    pub architecture: String,
    pub scripting_backend: String,
    pub unity_version: String,
    pub is_linux: bool,
    pub steam_launch: Option<SteamLaunchOption>,
}

pub fn is_unity_exe(game_exe_path: &PathBuf) -> bool {
    if let Ok(data_path) = get_data_path(game_exe_path) {
        fs::metadata(game_exe_path).is_ok() && fs::metadata(data_path).is_ok()
    } else {
        false
    }
}

fn file_name_without_extension(file_path: &PathBuf) -> Option<&str> {
    file_path.file_stem().and_then(|stem| stem.to_str())
}

fn get_data_path(game_exe_path: &PathBuf) -> Result<PathBuf, &'static str> {
    if let Some(parent) = game_exe_path.parent() {
        if let Some(exe_name) = file_name_without_extension(game_exe_path) {
            let data_folder_name = format!("{}_Data", exe_name);
            Ok(parent.join(data_folder_name))
        } else {
            Err("Failed to get file name without extension")
        }
    } else {
        Err("Failed to get parent directory")
    }
}
