use appinfo::SteamLaunchOption;
use serde::Serialize;
use specta::Type;
use std::{error::Error, fs, path::PathBuf};

use crate::appinfo;

#[derive(Serialize, Type)]
pub enum UnityScriptingBackend {
    Il2Cpp,
    Mono,
}

#[derive(Serialize, Type)]
pub enum Architecture {
    X64,
    X32,
}

#[derive(Serialize, Type)]
pub enum OperatingSystem {
    Linux,
    Windows,
}

#[derive(Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct GameExecutable {
    pub id: String,
    pub name: String,
    pub is_legacy: bool,
    pub mod_files_path: String,
    pub full_path: PathBuf,
    pub architecture: Architecture,
    pub scripting_backend: UnityScriptingBackend,
    pub unity_version: String,
    pub operating_system: OperatingSystem,
    pub steam_launch: Option<SteamLaunchOption>,
}

pub fn is_unity_exe(game_exe_path: &PathBuf) -> bool {
    if let Ok(data_path) = get_data_path(game_exe_path) {
        game_exe_path.is_file() && data_path.is_dir()
    } else {
        false
    }
}

pub fn get_unity_scripting_backend(
    game_exe_path: &PathBuf,
) -> Result<UnityScriptingBackend, String> {
    match game_exe_path.parent() {
        Some(game_folder) => {
            if game_folder.join("GameAssembly.dll").is_file()
                || game_folder.join("GameAssembly.so").is_file()
            {
                Ok(UnityScriptingBackend::Il2Cpp)
            } else {
                Ok(UnityScriptingBackend::Mono)
            }
        }
        None => Err("Noooo".to_owned()),
    }
}

fn file_name_without_extension(file_path: &PathBuf) -> Option<&str> {
    file_path.file_stem()?.to_str()
}

fn get_data_path(game_exe_path: &PathBuf) -> Result<PathBuf, &'static str> {
    if let Some(parent) = game_exe_path.parent() {
        if let Some(exe_name) = file_name_without_extension(game_exe_path) {
            let data_folder_name = format!("{}_Data", exe_name);
            println!("exe_name: {}", exe_name);
            println!("data_folder_name: {}", data_folder_name);
            Ok(parent.join(data_folder_name))
        } else {
            Err("Failed to get file name without extension")
        }
    } else {
        Err("Failed to get parent directory")
    }
}
