use std::{
	fs::{self},
	path::{Path, PathBuf},
};

use lazy_regex::{regex_captures, regex_find};
use pelite::pe64::{Pe, PeFile};

use crate::{
	game_engines::game_engine::{GameEngine, GameEngineBrand, GameEngineVersion},
	paths::glob_path,
};

fn get_version_from_metadata(file_bytes: &[u8]) -> Option<GameEngineVersion> {
	let file = PeFile::from_bytes(&file_bytes).ok()?;
	// Get the version resource
	let version = file.resources().ok()?.version_info().ok()?.fixed()?;
	// Get the fixed file info			// Print the file version
	println!(
		"File version: {}.{}.{}",
		version.dwFileVersion.Major, version.dwFileVersion.Minor, version.dwFileVersion.Patch
	);

	let major = u32::from(version.dwFileVersion.Major);
	let minor = u32::from(version.dwFileVersion.Minor);
	let patch = u32::from(version.dwFileVersion.Patch);

	Some(GameEngineVersion {
		major,
		minor,
		patch,
		suffix: None,
		display: format!("{major}.{minor}.{patch}"),
	})
}

fn get_version_from_exe_parse(file_bytes: &[u8]) -> Option<GameEngineVersion> {
	let file = PeFile::from_bytes(&file_bytes).ok()?;
	// Get the version resource
	let version = file.resources().ok()?.version_info().ok()?.fixed()?;
	// Get the fixed file info			// Print the file version
	println!(
		"File version: {}.{}.{}",
		version.dwFileVersion.Major, version.dwFileVersion.Minor, version.dwFileVersion.Patch
	);

	// Looking for strings like "+UE4+release-4.25", or just "+UE4" if the full version isn't found.
	// The extra \x00 are because the strings are unicode.
	let match_result = regex_find!(
		r#"(?x)
			# Case insensitive.
			(?i)

			# Starts with "+UE".
			\+\x00U\x00E\x00
			
			# Major version number.
			[45]\x00

			# Optional block with full version number.
			(?:
				# Skip over some characters, usually something like "+release-",
				# but changes between different games.
				.{0,100}?

				# Full version as "major.minor".
				[45]\x00\.\x00(\d\x00)+
			)?
		"#B,
		file_bytes
	);
	// I also noticed the game ABZU has the version in the exe as "4.12.5-0+UE4".
	// But I don't know if any other games do that, so I didn't try to match it.

	let match_string = String::from_utf16_lossy(
		&match_result?
			.chunks(2)
			.map(|e| u16::from_le_bytes(e.try_into().unwrap_or_default()))
			.collect::<Vec<_>>(),
	);

	// Regex again because the byte regex above can't extract the match groups.
	// Can either be major.minor, or just major.
	let (_, major, minor) = regex_captures!(
		r#"(?x)
			# Case insensitive.
			(?i)
		
			# Starts with "+UE".
			\+UE
			
			# Capture major version number.
			([45])
			
			# Capture optional block with full version number.
			(?:
				# Skip over some characters, usually something like "+release-".
				.*?
				
				# Full version as "major.minor".
				# Capture minor only (major already captured above).
				[45]\.(\d+)
			)?
		"#,
		&match_string
	)?;

	Some(GameEngineVersion {
		major: major.parse().unwrap_or(0),
		minor: minor.parse().unwrap_or(0),
		patch: 0,
		suffix: None,
		display: format!("{major}.{}", {
			if minor.is_empty() {
				// If we couldn't figure out the minor version,
				// we just put an x in there, like "4.x".
				"x"
			} else {
				minor
			}
		}),
	})
}

fn get_version(game_exe_path: &Path) -> Option<GameEngineVersion> {
	let actual_binary = get_actual_unreal_binary(game_exe_path);
	match fs::read(actual_binary) {
		Ok(file_bytes) => {
			return get_version_from_metadata(&file_bytes)
				.or_else(|| get_version_from_exe_parse(&file_bytes));
		}
		Err(err) => {
			println!("Failed to read game exe: {err}");
		}
	}

	None
}

pub fn get_actual_unreal_binary(game_exe_path: &Path) -> PathBuf {
	if let Some(parent) = game_exe_path.parent() {
		if parent.ends_with("Win64") || parent.ends_with("Win32") {
			return game_exe_path.to_path_buf();
		}

		let paths = glob_path(
			&parent
				.join("*")
				.join("Binaries")
				.join("Win[63][42]")
				.join("*.exe"),
		);

		if let Ok(mut paths) = paths {
			let path = paths.find(|path_result| {
				path_result
					.as_ref()
					.map_or(false, |path| !path.starts_with(parent.join("Engine")))
			});

			if let Some(Ok(path)) = path {
				return path;
			}
		}
	}

	game_exe_path.to_path_buf()
}

fn is_unreal_exe(game_path: &Path) -> bool {
	const VALID_FOLDER_NAMES: [&str; 3] = ["Win64", "Win32", "ThirdParty"];

	if let Some(parent) = game_path.parent() {
		// For cases where the registered exe points to a launcher at the root level:
		if VALID_FOLDER_NAMES.iter().any(|folder_name| {
			parent
				.join("Engine")
				.join("Binaries")
				.join(folder_name)
				.is_dir()
		}) {
			return true;
		}

		// For cases where the registered exe points directly to the shipping binary:
		if parent.ends_with("Win64") || parent.ends_with("Win32") {
			if let Some(binaries) = parent.parent() {
				if binaries.ends_with("Binaries") {
					return true;
				}
			}
		}
	}

	false
}

pub fn get_engine(game_path: &Path) -> Option<GameEngine> {
	if is_unreal_exe(game_path) {
		Some(GameEngine {
			brand: GameEngineBrand::Unreal,
			version: get_version(game_path),
		})
	} else {
		None
	}
}
