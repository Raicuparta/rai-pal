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
	game_engines::game_engine::{EngineBrand, EngineVersion, GameEngine},
	game_executable::{get_architecture, Architecture, GameExecutable},
	paths::{self, glob_path},
	result::{Error, Result},
};

#[serializable_enum]
pub enum UnityScriptingBackend {
	Il2Cpp,
	Mono,
}

pub fn parse_version(string: &str) -> Option<EngineVersion> {
	let (full, major, minor, patch, suffix) = regex_captures!(
		r#"(?x)
			# Version number as "major.minor.patch".
			(\d+)\.(\d+)\.(\d+)

			# Suffix, like "f1" or "p3".
			([fp]\d+)
		"#,
		&string
	)?;

	Some(EngineVersion {
		display: full.to_string(),
		numbers: EngineVersionNumbers {
			major: major.parse().unwrap_or(0),
			minor: minor.parse().ok(),
			patch: patch.parse().ok(),
		},
		suffix: Some(suffix.to_string()),
	})
}

fn get_version_from_asset(asset_path: &Path) -> Result<EngineVersion> {
	let mut file = File::open(asset_path)?;
	let mut data = vec![0u8; 4096];

	let bytes_read = file.read(&mut data)?;
	if bytes_read == 0 {
		return Err(Error::EmptyFile(asset_path.to_path_buf()));
	}

	let data_str = String::from_utf8_lossy(&data[..bytes_read]);

	parse_version(&data_str).ok_or(Error::FailedToParseUnityVersionAsset(
		asset_path.to_path_buf(),
	))
}

fn get_version(game_exe_path: &Path) -> Option<EngineVersion> {
	const ASSETS_WITH_VERSION: [&str; 3] = ["globalgamemanagers", "mainData", "data.unity3d"];

	if let Ok(data_path) = get_unity_data_path(game_exe_path) {
		for asset_name in &ASSETS_WITH_VERSION {
			let asset_path = data_path.join(asset_name);

			if let Ok(metadata) = fs::metadata(&asset_path) {
				if metadata.is_file() {
					match get_version_from_asset(&asset_path) {
						Ok(version) => {
							return Some(version);
						}
						Err(err) => error!("Failed to get Unity version: {err}"),
					}
				}
			}
		}
	}

	None
}

fn get_unity_data_path(game_exe_path: &Path) -> Result<PathBuf> {
	let parent = paths::path_parent(game_exe_path)?;
	let file_stem = paths::file_name_without_extension(game_exe_path)?;

	Ok(parent.join(format!("{file_stem}_Data")))
}

fn get_scripting_backend(path: &Path) -> Option<UnityScriptingBackend> {
	match paths::path_parent(path) {
		Ok(game_folder) => {
			if game_folder.join("GameAssembly.dll").is_file()
				|| game_folder.join("GameAssembly.so").is_file()
			{
				Some(UnityScriptingBackend::Il2Cpp)
			} else {
				Some(UnityScriptingBackend::Mono)
			}
		}
		Err(err) => {
			error!("Failed to get Unity scripting backend: {err}");
			None
		}
	}
}

fn is_unity_exe(game_path: &Path) -> bool {
	game_path.is_file() && get_unity_data_path(game_path).is_ok_and(|data_path| data_path.is_dir())
}

// If we can't figure out the architecture by reading the executable,
// we try to guess it using Unity-specific stuff. This isn't perfect though.
fn get_alt_architecture(game_path: &Path) -> Option<Architecture> {
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
		if let Some(first_dll) = glob_path(&game_folder.join("*.dll")).first() {
			if let Ok(Some(architecture)) = get_architecture(first_dll) {
				return Some(architecture);
			}
		}

		// If there are no top-level dlls, we try the Unity plugin dlls.
		// On an unmodified game, the plugin dlls would probably be a good bet for this.
		// But it's common to drop all kinds of dlls in the plugins folder,
		// so I'm leaving it for last. (mostly because my own UUVR mod drops both the
		// x86 and x64 dlls in the folder so Unity picks the right one)
		if let Ok(unity_data_path) = get_unity_data_path(game_path) {
			if let Some(plugin_dll) =
				glob_path(&unity_data_path.join("Plugins").join("**").join("*.dll")).first()
			{
				if let Ok(Some(architecture)) = get_architecture(plugin_dll) {
					return Some(architecture);
				}
			}
		}
	}

	None
}

pub fn get_executable(game_path: &Path) -> Option<GameExecutable> {
	if is_unity_exe(game_path) {
		let architecture = get_architecture(game_path)
			.unwrap_or(None)
			.or_else(|| get_alt_architecture(game_path));

		Some(GameExecutable {
			path: game_path.to_path_buf(),
			name: game_path.file_name()?.to_string_lossy().to_string(),
			architecture,
			scripting_backend: get_scripting_backend(game_path),
			engine: Some(GameEngine {
				brand: EngineBrand::Unity,
				version: get_version(game_path),
			}),
		})
	} else {
		None
	}
}
