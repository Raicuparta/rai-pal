//! Unity Game Metadata Extraction Sequence:
//! 
//! When a game is identified as a Unity game, the following sequence occurs to extract its metadata.
//! The flow involves resolving potential launcher misdirections before extracting the actual data.
//! 
//! ```text
//! [Identify Executable] (e.g. Launcher.exe or Game.exe)
//!       |
//!       v
//! [Resolve Data Path]
//!  |-- Try: {exe_stem}_Data (e.g. Launcher_Data)
//!  |-- Try (Fallback): Any *_Data with Unity assets (e.g. Game_Data)
//!       |
//!       v
//! [Resolve True Executable]
//!  |-- If fallback used: Update exe_path to match data folder (Game.exe)
//!  |-- Else: Keep original exe_path
//!       |
//!       +--> [Engine Version]
//!       |     |-- Regex scan globalgamemanagers, mainData, etc.
//!       |     |-- Extract Major.Minor.Patch[Suffix]
//!       |
//!       +--> [Scripting Backend]
//!       |     |-- Look for GameAssembly.dll
//!       |     |-- Result: Il2Cpp or Mono
//!       |
//!       +--> [Architecture]
//!             |-- Read PE headers / Check for UnityCrashHandler64.exe
//! ```

use std::{
	fs::{self, File},
	io::Read,
	path::{Path, PathBuf},
};

use lazy_regex::regex_captures;
use log::error;
use rai_pal_proc_macros::serializable_enum;

use super::game_engine::EngineVersionNumbers;
use crate::{
	architecture::{Architecture, get_architecture},
	data_types::path_data::PathData,
	game::DbGame,
	game_engines::game_engine::EngineVersion,
	paths::{self, glob_path},
	result::{Error, Result},
};

#[serializable_enum]
pub enum UnityBackend {
	Il2Cpp,
	Mono,
}

pub fn parse_version(string: &str) -> Option<EngineVersion> {
	// Parses Unity version strings, such as "2021.3.45f2", "2021.3.45f1c1" or just "2021.3.45".
	// Group 1: Major version (\d+)
	// Group 2: Minor version (\d+)
	// Group 3: Patch version (\d+)
	// Group 4: Optional suffix (e.g., "f2" or "f1c1", matching [a-z0-9]+)
	let (full, major, minor, patch, suffix) =
		regex_captures!(r"(\d+)\.(\d+)\.(\d+)(?:[.\s]*)?([a-z0-9]+)?", &string)?;

	Some(EngineVersion {
		display: full.to_string(),
		numbers: EngineVersionNumbers {
			major: major.parse().unwrap_or(0),
			minor: minor.parse().ok(),
			patch: patch.parse().ok(),
		},
		suffix: if suffix.is_empty() { None } else { Some(suffix.to_string()) },
	})
}

fn get_version_from_asset(asset_path: &Path) -> Result<EngineVersion> {
	let mut file = File::open(asset_path)?;
	let mut data = vec![0u8; 65536];

	let bytes_read = file.read(&mut data)?;
	if bytes_read == 0 {
		return Err(Error::EmptyFile(asset_path.to_path_buf()));
	}

	let data_str = String::from_utf8_lossy(&data[..bytes_read]);

	let version = parse_version(&data_str).ok_or(Error::FailedToParseUnityVersionAsset(
		asset_path.to_path_buf(),
	))?;

	log::debug!(
		"Found Unity version {} in asset {}",
		version.display,
		asset_path.display()
	);

	Ok(version)
}

fn get_version(data_path: &Path) -> Option<EngineVersion> {
	const ASSETS_WITH_VERSION: [&str; 3] = ["globalgamemanagers", "mainData", "data.unity3d"];

	log::debug!("Checking Unity data path for version: {}", data_path.display());
	for asset_name in &ASSETS_WITH_VERSION {
		let asset_path = data_path.join(asset_name);

		log::debug!("Checking asset path: {}", asset_path.display());

		if let Ok(metadata) = fs::metadata(&asset_path)
			&& metadata.is_file()
		{
			log::debug!("Found asset file: {}", asset_path.display());
			match get_version_from_asset(&asset_path) {
				Ok(version) => {
					return Some(version);
				}
				Err(err) => error!("Failed to get Unity version from {}: {}", asset_path.display(), err),
			}
		}
	}

	None
}

fn get_unity_data_path(game_exe_path: &Path) -> Result<PathBuf> {
	let parent = paths::path_parent(game_exe_path)?;
	let file_stem = paths::file_name_without_extension(game_exe_path)?;

	let default_path = parent.join(format!("{file_stem}_Data"));
	if default_path.is_dir() {
		return Ok(default_path);
	}

	// Fallback for games with launchers
	// Search for any directory ending in _Data that looks like a Unity data folder
	if let Ok(entries) = fs::read_dir(&parent) {
		for entry in entries.flatten() {
			let path = entry.path();
			if path.is_dir() {
				if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
					if file_name.ends_with("_Data") {
						if path.join("globalgamemanagers").is_file() 
							|| path.join("mainData").is_file() 
							|| path.join("data.unity3d").is_file() 
						{
							log::debug!("Found Unity data path fallback for {}: {}", file_stem, path.display());
							return Ok(path);
						}
					}
				}
			}
		}
	}

	Ok(default_path)
}

fn get_unity_backend(path: &Path) -> Option<UnityBackend> {
	match paths::path_parent(path) {
		Ok(game_folder) => {
			if game_folder.join("GameAssembly.dll").is_file()
				|| game_folder.join("GameAssembly.so").is_file()
			{
				Some(UnityBackend::Il2Cpp)
			} else {
				Some(UnityBackend::Mono)
			}
		}
		Err(err) => {
			error!("Failed to get Unity scripting backend: {err}");
			None
		}
	}
}

pub fn is_unity_exe(game_path: &Path) -> bool {
	game_path.is_file() && get_unity_data_path(game_path).is_ok_and(|data_path| data_path.is_dir())
}

// If we can't figure out the architecture by reading the executable,
// we try to guess it using Unity-specific stuff. This isn't perfect though.
fn get_alt_architecture(game_path: &Path, data_path_opt: Option<&Path>) -> Option<Architecture> {
	if let Some(game_folder) = game_path.parent() {
		// If the x64 crash handler is present, then it's an x64 game.
		// But it not being present can mean it's just an older Unity version.
		if game_folder.join("UnityCrashHandler64.exe").is_file() {
			return Some(Architecture::X64);
		}

		// Then we try to scan the first dll file we find at the top level.
		// This would usually be UnityPlayer.dll, steam_api.dll, etc.
		// Here the guessing can go wrong, since it's possible a top level dll is actual x86,
		// when the actual game is x64.
		if let Some(first_dll) = glob_path(&game_folder.join("*.dll")).first()
			&& let Ok(Some(architecture)) = get_architecture(first_dll)
		{
			return Some(architecture);
		}

		// If there are no top-level dlls, we try the Unity plugin dlls.
		// On an unmodified game, the plugin dlls would probably be a good bet for this.
		// But it's common to drop all kinds of dlls in the plugins folder,
		// so I'm leaving it for last. (mostly because my own UUVR mod drops both the
		// x86 and x64 dlls in the folder so Unity picks the right one)
		if let Some(unity_data_path) = data_path_opt
			&& let Some(plugin_dll) =
				glob_path(&unity_data_path.join("Plugins").join("**").join("*.dll")).first()
			&& let Ok(Some(architecture)) = get_architecture(plugin_dll)
		{
			return Some(architecture);
		}
	}

	None
}

pub fn process_game(game: &mut DbGame) {
	if let Some(PathData(exe_path)) = game.exe_path.clone() {
		let mut actual_exe_path = exe_path.clone();
		let mut resolved_data_path = None;

		// If the original exe is a launcher, get_unity_data_path will fallback to the real data folder
		if let Ok(data_path) = get_unity_data_path(&exe_path) {
			resolved_data_path = Some(data_path.clone());
			if let Some(file_name) = data_path.file_name().and_then(|n| n.to_str()) {
				let expected_exe_stem = file_name.trim_end_matches("_Data");
				if let Some(parent) = data_path.parent() {
					let mut possible_exe = parent.join(expected_exe_stem);
					if let Some(ext) = exe_path.extension() {
						possible_exe.set_extension(ext);
					}
					
					if possible_exe.is_file() && possible_exe != exe_path {
						log::debug!("Updating game exe_path from launcher {} to {}", exe_path.display(), possible_exe.display());
						actual_exe_path = possible_exe;
						game.exe_path = Some(PathData(actual_exe_path.clone()));
					}
				}
			}
		}

		game.unity_backend = get_unity_backend(&actual_exe_path);
		
		if let Some(data_path) = resolved_data_path.as_ref() {
			if let Some(version) = get_version(data_path) {
				game.engine_version_major = Some(version.numbers.major);
				game.engine_version_minor = version.numbers.minor;
				game.engine_version_patch = version.numbers.patch;
				game.engine_version_display = Some(version.display);
			}
		}

		game.architecture = get_architecture(&actual_exe_path)
			.unwrap_or_else(|err| {
				log::error!(
					"Failed to get exe architecture for {}: {}",
					actual_exe_path.display(),
					err
				);
				None
			})
			.or_else(|| get_alt_architecture(&actual_exe_path, resolved_data_path.as_deref()));
	}
}
