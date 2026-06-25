use std::{
	fs,
	path::Path,
};

use lazy_regex::regex_find;
use log::error;

use super::game_engine::EngineVersionNumbers;
use crate::{
	game::DbGame,
	game_engines::game_engine::EngineVersion,
	result::LogErrExt,
};

pub fn is_godot_exe(exe_path: &Path) -> bool {
	exe_path.is_file()
		&& fs::read(exe_path).is_ok_and(|bytes| regex_find!(r#"(?i)godot"#B, &bytes).is_some())
}

fn parse_version(version_string: &str) -> Option<EngineVersion> {
	let parts: Vec<&str> = version_string
		.split(|c: char| !c.is_ascii_digit())
		.filter(|s| !s.is_empty())
		.collect();

	if parts.is_empty() {
		return None;
	}

	let major: u32 = parts[0].parse().ok_or_log(&format!(
		"Failed to parse major version number `{}` from version string `{version_string}`",
		parts[0]
	))?;

	let minor = if parts.len() >= 2 {
		Some(parts[1].parse().ok_or_log(&format!(
			"Failed to parse minor version number `{}` from version string `{version_string}`",
			parts[1]
		))?)
	} else {
		None
	};

	let patch = if parts.len() >= 3 {
		parts[2].parse().ok_or_log(&format!(
			"Failed to parse patch version number `{}` from version string `{version_string}`",
			parts[2]
		))
	} else {
		None
	};

	let display = match (minor, patch) {
		(Some(m), Some(p)) => format!("{major}.{m}.{p}"),
		(Some(m), None) => format!("{major}.{m}"),
		(None, _) => format!("{major}"),
	};

	Some(EngineVersion {
		display,
		numbers: EngineVersionNumbers {
			major,
			minor,
			patch,
		},
		suffix: None,
	})
}

fn get_version_from_exe(exe_path: &Path) -> Option<EngineVersion> {
	let file_bytes = match fs::read(exe_path) {
		Ok(bytes) => bytes,
		Err(err) => {
			error!(
				"Failed to read exe `{}`. Error: {}",
				exe_path.display(),
				err
			);
			return None;
		}
	};

	// Godot 4.x embeds "Godot Engine vX.Y.Z" or "Godot vX.Y.Z".
	if let Some(m) = regex_find!(
		r#"(?i)godot(?:[ _]engine)?[ _]v\d+(?:\.\d+)*"#B,
		&file_bytes
	) {
		let match_string = String::from_utf8_lossy(m.as_ref());
		return parse_version(&match_string);
	}

	// Godot 3.x only embeds the bare version like "3.3.1.stable.official".
	if let Some(m) = regex_find!(
		r#"\d+\.\d+\.\d+\.(?:stable|beta|alpha|rc|dev)\d*"#B,
		&file_bytes
	) {
		let match_string = String::from_utf8_lossy(m.as_ref());
		return parse_version(&match_string);
	}

	None
}

pub fn process_game(game: &mut DbGame) {
	if let Some(exe_path) = game.exe_path.as_ref() {
		if let Some(version) = get_version_from_exe(exe_path) {
			game.engine_version_major = Some(version.numbers.major);
			game.engine_version_minor = version.numbers.minor;
			game.engine_version_patch = version.numbers.patch;
			game.engine_version_display = Some(version.display);
		}
	}
}

#[cfg(test)]
mod tests {
	use std::path::Path;

	use super::*;

	#[test]
	fn test_moldrise_detection() {
		let path =
			Path::new("/mnt/big_nvme/SteamLibrary/steamapps/common/MOLDRISE Demo/MOLDRISE.exe");
		assert!(path.is_file(), "Test exe not found");
		assert!(is_godot_exe(path), "Failed to detect as Godot");
	}

	#[test]
	fn test_moldrise_version() {
		let path =
			Path::new("/mnt/big_nvme/SteamLibrary/steamapps/common/MOLDRISE Demo/MOLDRISE.exe");
		let version = get_version_from_exe(path);
		assert!(version.is_some(), "Failed to get version");
		let v = version.unwrap();
		assert_eq!(v.numbers.major, 4);
		assert_eq!(v.numbers.minor, Some(6));
		assert_eq!(v.numbers.patch, Some(3));
	}

	#[test]
	fn test_glongoboy_version() {
		let path = Path::new("/home/rai/Downloads/glongoboy/glongoboy.exe");
		assert!(path.is_file(), "Test exe not found");
		assert!(is_godot_exe(path), "Failed to detect as Godot");
		let version = get_version_from_exe(path);
		assert!(version.is_some(), "Failed to get version");
		let v = version.unwrap();
		assert_eq!(v.numbers.major, 4);
		assert_eq!(v.numbers.minor, Some(6));
		assert_eq!(v.numbers.patch, None);
	}

	#[test]
	fn test_spring_and_fall_version() {
		let path =
			Path::new("/home/rai/Downloads/spring-and-fall-windows-64bit/spring-and-fall.exe");
		assert!(path.is_file(), "Test exe not found");
		assert!(is_godot_exe(path), "Failed to detect as Godot");
		let version = get_version_from_exe(path);
		assert!(version.is_some(), "Failed to get version");
		let v = version.unwrap();
		assert_eq!(v.numbers.major, 3);
		assert_eq!(v.numbers.minor, Some(3));
		assert_eq!(v.numbers.patch, Some(1));
	}
}
