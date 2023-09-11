use appinfo::SteamLaunchOption;
use goblin::elf::Elf;
use goblin::pe::PE;
use regex::Regex;
use serde::Serialize;
use specta::Type;
use std::{
    error::Error,
    fs::{metadata, File},
    io::Read,
    path::{Path, PathBuf},
};

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
    Unknown,
}

#[derive(Serialize, Type)]
pub enum OperatingSystem {
    Linux,
    Windows,
    Unknown,
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

pub fn is_unity_exe(game_exe_path: &Path) -> bool {
    get_data_path(game_exe_path).map_or(false, |data_path| {
        game_exe_path.is_file() && data_path.is_dir()
    })
}

pub fn get_unity_scripting_backend(game_exe_path: &Path) -> Result<UnityScriptingBackend, String> {
    game_exe_path.parent().map_or_else(
        || Err("Noooo".to_owned()),
        |game_folder| {
            if game_folder.join("GameAssembly.dll").is_file()
                || game_folder.join("GameAssembly.so").is_file()
            {
                Ok(UnityScriptingBackend::Il2Cpp)
            } else {
                Ok(UnityScriptingBackend::Mono)
            }
        },
    )
}

fn file_name_without_extension(file_path: &Path) -> Option<&str> {
    file_path.file_stem()?.to_str()
}

fn get_data_path(game_exe_path: &Path) -> Result<PathBuf, &'static str> {
    game_exe_path
        .parent()
        .map_or(Err("Failed to get parent directory"), |parent| {
            file_name_without_extension(game_exe_path).map_or(
                Err("Failed to get file name without extension"),
                |exe_name| Ok(parent.join(format!("{exe_name}_Data"))),
            )
        })
}

pub fn get_os_and_architecture(
    file_path: &Path,
) -> Result<(OperatingSystem, Architecture), String> {
    let file = File::open(file_path);

    file.map_or_else(
        |_| Err("Failed to open the file".to_owned()),
        |mut file| {
            let mut buffer = Vec::new();
            if file.read_to_end(&mut buffer).is_ok() {
                if let Ok(elf) = Elf::parse(&buffer) {
                    match elf.header.e_machine {
                        goblin::elf::header::EM_X86_64 => {
                            Ok((OperatingSystem::Linux, Architecture::X64))
                        }
                        goblin::elf::header::EM_386 => {
                            Ok((OperatingSystem::Linux, Architecture::X32))
                        }
                        _ => Ok((OperatingSystem::Linux, Architecture::Unknown)),
                    }
                } else if let Ok(pe) = PE::parse(&buffer) {
                    match pe.header.coff_header.machine {
                        goblin::pe::header::COFF_MACHINE_X86_64 => {
                            Ok((OperatingSystem::Windows, Architecture::X64))
                        }
                        goblin::pe::header::COFF_MACHINE_X86 => {
                            Ok((OperatingSystem::Windows, Architecture::X32))
                        }
                        _ => Ok((OperatingSystem::Windows, Architecture::Unknown)),
                    }
                } else {
                    Ok((OperatingSystem::Unknown, Architecture::Unknown))
                }
            } else {
                Err("Failed to read the file".to_owned())
            }
        },
    )
}

const ASSETS_WITH_VERSION: [&str; 3] = ["globalgamemanagers", "mainData", "data.unity3d"];

pub fn get_unity_version(game_exe_path: &Path) -> String {
    if let Ok(data_path) = get_data_path(game_exe_path) {
        for asset_name in &ASSETS_WITH_VERSION {
            let asset_path = data_path.join(asset_name);

            if let Ok(metadata) = metadata(&asset_path) {
                if metadata.is_file() {
                    if let Ok(version) = get_version_from_asset(&asset_path) {
                        return version;
                    }
                }
            }
        }
    }

    "Unknown".into()
}

fn get_version_from_asset(asset_path: &Path) -> Result<String, Box<dyn Error>> {
    let mut file = File::open(asset_path)?;
    let mut data = vec![0u8; 4096];

    let bytes_read = file.read(&mut data)?;
    if bytes_read == 0 {
        return Err("No data read from file".into());
    }

    let data_str = String::from_utf8_lossy(&data[..bytes_read]);
    let pattern = Regex::new(r"\d+\.\d+\.\d+[fp]\d+").unwrap();
    let match_result = pattern.find(&data_str);

    match_result.map_or_else(
        || Ok("No version found".to_string()),
        |matched| Ok(matched.as_str().to_string()),
    )
}
