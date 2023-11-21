use std::{
	fs::{
		self,
		File,
	},
	io::Read,
	path::{
		Path,
		PathBuf,
	},
};

use lazy_regex::regex_captures;

use crate::{
	game_engines::game_engine::{
		GameEngine,
		GameEngineBrand,
		GameEngineVersion,
	},
	game_executable::{
		get_os_and_architecture,
		GameExecutable,
	},
	paths,
	result::{
		Error,
		Result,
	},
	serializable_enum,
};

serializable_enum!(UnityScriptingBackend { Il2Cpp, Mono });

fn get_version_from_asset(asset_path: &Path) -> Result<GameEngineVersion> {
	let mut file = File::open(asset_path)?;
	let mut data = vec![0u8; 4096];

	let bytes_read = file.read(&mut data)?;
	if bytes_read == 0 {
		return Err(Error::EmptyFile(asset_path.to_path_buf()));
	}

	let data_str = String::from_utf8_lossy(&data[..bytes_read]);
	let (full, major, minor, patch, suffix) = regex_captures!(
		r#"(?x)
			# Version number as "major.minor.patch".
			(\d+)\.(\d+)\.(\d+)

			# Suffix, like "f1" or "p3".
			([fp]\d+)
		"#,
		&data_str
	)
	.ok_or(Error::FailedToParseUnityVersionAsset(
		asset_path.to_path_buf(),
	))?;

	Ok(GameEngineVersion {
		display: full.to_string(),
		major: major.parse().unwrap_or(0),
		minor: minor.parse().unwrap_or(0),
		patch: patch.parse().unwrap_or(0),
		suffix: Some(suffix.to_string()),
	})
}

fn get_version(game_exe_path: &Path) -> Option<GameEngineVersion> {
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
						Err(err) => eprintln!("Failed to get Unity version: {err}"),
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
			eprintln!("Failed to get Unity scripting backend: {err}");
			None
		}
	}
}

fn is_unity_exe(game_path: &Path) -> bool {
	game_path.is_file()
		&& get_unity_data_path(game_path).map_or(false, |data_path| data_path.is_dir())
}

pub fn get_engine(game_path: &Path) -> Option<GameExecutable> {
	if is_unity_exe(game_path) {
		let (operating_system, architecture) =
			get_os_and_architecture(game_path).unwrap_or((None, None));

		Some(GameExecutable {
			path: game_path.to_path_buf(),
			architecture,
			operating_system,
			scripting_backend: get_scripting_backend(game_path),
			engine: Some(GameEngine {
				brand: GameEngineBrand::Unity,
				version: get_version(game_path),
			}),
		})
	} else {
		None
	}
}
