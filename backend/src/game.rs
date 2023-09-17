use crate::{appinfo, Result};
use anyhow::anyhow;
use appinfo::SteamLaunchOption;
use core::fmt;
use directories::ProjectDirs;
use goblin::elf::Elf;
use goblin::pe::PE;
use lazy_regex::regex_find;
use serde::Serialize;
use specta::Type;
use std::{
    collections::HashMap,
    fmt::Display,
    fs::{self, metadata, File},
    io::Read,
    path::{Path, PathBuf},
};

#[derive(Serialize, Type, Clone, PartialEq, Eq, Hash, Debug)]
pub enum UnityScriptingBackend {
    Il2Cpp,
    Mono,
}

// TODO clean this up, avoid repetition if possible.
impl Display for UnityScriptingBackend {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

#[derive(Serialize, Type, Clone, PartialEq, Eq, Hash, Debug)]
pub enum Architecture {
    Unknown,
    X64,
    X86,
}

// TODO clean this up, avoid repetition if possible.
impl Display for Architecture {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

#[derive(Serialize, Type, Clone, PartialEq, Eq, Hash, Debug)]
pub enum OperatingSystem {
    Unknown,
    Linux,
    Windows,
}

// TODO clean this up, avoid repetition if possible.
impl Display for OperatingSystem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

#[derive(Serialize, Type, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Game {
    pub id: String,
    pub name: String,
    pub discriminator: Option<String>,
    pub is_legacy: bool,
    pub mod_files_path: String,
    pub full_path: PathBuf,
    pub architecture: Architecture,
    pub scripting_backend: UnityScriptingBackend,
    pub unity_version: String,
    pub operating_system: OperatingSystem,
    pub steam_launch: Option<SteamLaunchOption>,
}

impl Game {
    pub fn new(
        id: String,
        name: String,
        discriminator: Option<String>,
        full_path: &Path,
        steam_launch: Option<&SteamLaunchOption>,
    ) -> Option<Self> {
        // Games exported by Unity always have one of these extensions.
        const VALID_EXTENSIONS: [&str; 3] = ["exe", "x86_64", "x86"];

        // We ignore games that don't have an extension.
        let extension = full_path.extension()?.to_str()?;

        if !VALID_EXTENSIONS.contains(&extension) {
            return None;
        }

        if extension == "x86" && full_path.with_extension("x86_64").is_file() {
            // If there's an x86_64 version, we ignore the x86 version.
            // I'm just gonna presume there are no x86 modders out there,
            // if someone cries about it I'll make this smarter.
            return None;
        }

        let (operating_system, architecture) = get_os_and_architecture(full_path).ok()?;

        if !is_unity_exe(full_path) {
            return None;
        }

        let unity_version = get_unity_version(full_path);

        Some(Self {
            architecture,
            full_path: full_path.to_owned(),
            id,
            is_legacy: is_legacy(&unity_version),
            operating_system,
            mod_files_path: String::new(),
            name,
            discriminator,
            scripting_backend: get_unity_scripting_backend(full_path).ok()?,
            steam_launch: steam_launch.cloned(),
            unity_version,
        })
    }

    pub fn open_game_folder(&self) -> Result {
        if let Some(parent) = self.full_path.parent() {
            Ok(open::that_detached(parent)?)
        } else {
            Err(anyhow!("Failed to find parent for game {}", self.name))
        }
    }

    pub fn get_installed_mods_folder(&self) -> Result<PathBuf> {
        let project_dirs = ProjectDirs::from("com", "raicuparta", "rai-pal")
            .ok_or_else(|| anyhow!("Failed to get user data folders"))?;

        let installed_mods_folder = project_dirs.data_dir().join("games").join(&self.id);
        fs::create_dir_all(&installed_mods_folder)?;

        Ok(installed_mods_folder)
    }

    pub fn open_mods_folder(&self) -> Result {
        open::that_detached(self.get_installed_mods_folder()?)
            .map_err(|err| anyhow!("Failed to open game mods folder: {err}"))
    }

    pub fn start(&self) -> Result {
        self.steam_launch
            .as_ref()
            .map_or_else(
                || open::that_detached(&self.full_path),
                |steam_launch| {
                    open::that_detached(format!(
                        "steam://launch/{}/{}",
                        steam_launch.app_id,
                        steam_launch.app_type.as_deref().unwrap_or("")
                    ))
                },
            )
            .map_err(|err| anyhow!("Failed to run game: {err}"))
    }
}

fn is_unity_exe(game_exe_path: &Path) -> bool {
    get_unity_data_path(game_exe_path).map_or(false, |data_path| {
        game_exe_path.is_file() && data_path.is_dir()
    })
}

fn get_unity_scripting_backend(game_exe_path: &Path) -> Result<UnityScriptingBackend> {
    game_exe_path.parent().map_or_else(
        || {
            Err(anyhow!(
                "Failed to get game exe parent while determining Unity scripting backend."
            ))
        },
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

fn get_unity_data_path(game_exe_path: &Path) -> Result<PathBuf> {
    game_exe_path
        .parent()
        .map_or(Err(anyhow!("Failed to get parent directory")), |parent| {
            file_name_without_extension(game_exe_path).map_or(
                Err(anyhow!("Failed to get file name without extension")),
                |exe_name| Ok(parent.join(format!("{exe_name}_Data"))),
            )
        })
}

fn get_os_and_architecture(file_path: &Path) -> Result<(OperatingSystem, Architecture)> {
    fs::read(file_path)
        .map(|file| {
            let elf_result = match Elf::parse(&file) {
                Ok(elf) => match elf.header.e_machine {
                    goblin::elf::header::EM_X86_64 => {
                        Ok((OperatingSystem::Linux, Architecture::X64))
                    }
                    goblin::elf::header::EM_386 => Ok((OperatingSystem::Linux, Architecture::X86)),
                    _ => Ok((OperatingSystem::Linux, Architecture::Unknown)),
                },
                Err(err) => Err(anyhow!("Failed to parse as ELF: {}", err)),
            };

            if elf_result.is_ok() {
                return elf_result;
            }

            let pe_result = match PE::parse(&file) {
                Ok(pe) => match pe.header.coff_header.machine {
                    goblin::pe::header::COFF_MACHINE_X86_64 => {
                        Ok((OperatingSystem::Windows, Architecture::X64))
                    }
                    goblin::pe::header::COFF_MACHINE_X86 => {
                        Ok((OperatingSystem::Windows, Architecture::X86))
                    }
                    _ => Ok((OperatingSystem::Windows, Architecture::Unknown)),
                },
                Err(err) => Err(anyhow!("Failed to parse as PE: {}", err)),
            };

            if pe_result.is_ok() {
                return pe_result;
            }

            println!("Failed to parse exe as ELF or PE");
            if let Err(err) = elf_result {
                println!("ELF error: {err}");
            }
            if let Err(err) = pe_result {
                println!("PE error: {err}");
            }

            Ok((OperatingSystem::Unknown, Architecture::Unknown))
        })?
        .map_err(|err| anyhow!("Failed to read the file: {err}"))
}

fn get_unity_version(game_exe_path: &Path) -> String {
    const ASSETS_WITH_VERSION: [&str; 3] = ["globalgamemanagers", "mainData", "data.unity3d"];

    if let Ok(data_path) = get_unity_data_path(game_exe_path) {
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

fn is_legacy(unity_version: &str) -> bool {
    unity_version
        .split('.')
        .map(|part| part.parse::<u32>().unwrap_or(0))
        .collect::<Vec<u32>>()
        .as_slice()
        < [5, 5].as_slice()
}

fn get_version_from_asset(asset_path: &Path) -> Result<String> {
    let mut file = File::open(asset_path)?;
    let mut data = vec![0u8; 4096];

    let bytes_read = file.read(&mut data)?;
    if bytes_read == 0 {
        return Err(anyhow!("No data read from file"));
    }

    let data_str = String::from_utf8_lossy(&data[..bytes_read]);
    let match_result = regex_find!(r"\d+\.\d+\.\d+[fp]\d+", &data_str);

    match_result.map_or_else(
        || Ok("No version found".to_string()),
        |matched| Ok(matched.to_string()),
    )
}

pub type Map = HashMap<String, Game>;
