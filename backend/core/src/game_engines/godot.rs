use std::{
	fs,
	path::Path,
};

use lazy_regex::regex_find;
use log::error;
use pelite::{
	pe32::{
		Pe as _,
		PeFile as PeFile32,
	},
	pe64::{
		Pe as _,
		PeFile as PeFile64,
	},
};

use super::{
	game_engine::EngineVersionNumbers,
	pe_utils,
};
use crate::{
	game::DbGame,
	game_engines::game_engine::EngineVersion,
	result::LogErrExt,
};

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

/// Scan a byte slice for Godot version strings.
fn scan_for_version(bytes: &[u8]) -> Option<EngineVersion> {
	// Godot 4.x: "Godot Engine v" or "Godot v"
	if let Some(m) = regex_find!(r#"(?-u)(?i)Godot (?:Engine )?v\d[-.\d]*"#B, bytes) {
		return parse_version(&String::from_utf8_lossy(m));
	}

	// Godot 3.x: version markers like "3.3.1.stable" — require a digit
	// to anchor the match and avoid false positives like "alphaTest".
	if let Some(m) = regex_find!(
		r#"(?-u)(?i)\d[-.\d]*(?:\.stable|\.beta|\.alpha|\.rc|\.dev)"#B,
		bytes
	) && let Some(ver) = parse_version(&String::from_utf8_lossy(m))
	{
		return Some(ver);
	}

	None
}

/// Detects and versions a Godot executable.
///
/// Pipeline:
/// 1. Parse PE header (PE32+ then PE32).
/// 2. Fast filter: read export DLL name using correct RVA→file mapping.
///    If name doesn't start with "godot" → None.
///    If exports can't be read, fall through to section scanning.
/// 3. Scan only the `.rdata` section for version strings.
pub fn check_exe(exe_path: &Path) -> Option<EngineVersion> {
	let file_bytes = match fs::read(exe_path) {
		Ok(b) => b,
		Err(err) => {
			error!(
				"Failed to read exe `{}`. Error: {}",
				exe_path.display(),
				err
			);
			return None;
		}
	};

	#[allow(
		clippy::disallowed_methods,
		reason = "Errors from PeFile parsing are expected."
	)]
	{
		if let Ok(pe) = PeFile64::from_bytes(&file_bytes) {
			return check_pe64(&pe, &file_bytes);
		}
		if let Ok(pe) = PeFile32::from_bytes(&file_bytes) {
			return check_pe32(&pe, &file_bytes);
		}
	}

	None
}

fn check_pe64(pe: &PeFile64<'_>, file_bytes: &[u8]) -> Option<EngineVersion> {
	let sections = pe.section_headers();
	let data_dir = pe.data_directory();

	// Fast filter: if we can read the export DLL name and it doesn't start with "godot",
	// skip the expensive section scan.
	if let Some(name) = pe_utils::try_read_export_dll_name(sections, data_dir, file_bytes)
		&& !name.to_ascii_lowercase().starts_with("godot")
	{
		return None;
	}

	if let Some(sec) = pe.section_headers().by_name(".rdata") {
		return pe.get_section_bytes(sec).ok().and_then(scan_for_version);
	}
	None
}

fn check_pe32(pe: &PeFile32<'_>, file_bytes: &[u8]) -> Option<EngineVersion> {
	let sections = pe.section_headers();
	let data_dir = pe.data_directory();

	if let Some(name) = pe_utils::try_read_export_dll_name(sections, data_dir, file_bytes)
		&& !name.to_ascii_lowercase().starts_with("godot")
	{
		return None;
	}

	if let Some(sec) = pe.section_headers().by_name(".rdata") {
		return pe.get_section_bytes(sec).ok().and_then(scan_for_version);
	}
	None
}

pub fn process_game(game: &mut DbGame, version: &EngineVersion) {
	game.engine_version_major = Some(version.numbers.major);
	game.engine_version_minor = version.numbers.minor;
	game.engine_version_patch = version.numbers.patch;
	game.engine_version_display = Some(version.display.clone());
}

#[cfg(test)]
mod tests {
	use std::{
		path::Path,
		time::Instant,
	};

	use super::*;

	#[test]
	fn test_moldrise() {
		let path =
			Path::new("/mnt/big_nvme/SteamLibrary/steamapps/common/MOLDRISE Demo/MOLDRISE.exe");
		assert!(path.is_file(), "Test exe not found");
		let version = check_exe(path).expect("Failed to detect/version");
		assert_eq!(version.numbers.major, 4);
		assert_eq!(version.numbers.minor, Some(6));
		assert_eq!(version.numbers.patch, Some(3));
	}

	#[test]
	fn test_glongoboy() {
		let path = Path::new("/home/rai/Downloads/glongoboy/glongoboy.exe");
		assert!(path.is_file(), "Test exe not found");
		let version = check_exe(path).expect("Failed to detect/version");
		assert_eq!(version.numbers.major, 4);
		assert_eq!(version.numbers.minor, Some(6));
		assert_eq!(version.numbers.patch, None);
	}

	#[test]
	fn test_spring_and_fall() {
		let path =
			Path::new("/home/rai/Downloads/spring-and-fall-windows-64bit/spring-and-fall.exe");
		assert!(path.is_file(), "Test exe not found");
		let version = check_exe(path).expect("Failed to detect/version");
		assert_eq!(version.numbers.major, 3);
		assert_eq!(version.numbers.minor, Some(3));
		assert_eq!(version.numbers.patch, Some(1));
	}

	#[test]
	fn test_unity_not_godot() {
		let path = Path::new("/mnt/big_nvme/Unity/Hub/Editor/UnitySetup-4.6.1/Unity.exe");
		if !path.is_file() {
			return;
		}
		assert!(
			check_exe(path).is_none(),
			"Unity exe should NOT be detected as Godot"
		);
	}

	#[test]
	fn test_asteroid_arcade() {
		let path = Path::new(
			"/mnt/big_nvme/SteamLibrary/steamapps/common/Asteroid Arcade/AsteroidArcade.exe",
		);
		assert!(path.is_file(), "Test exe not found");
		let version = check_exe(path).expect("Failed to detect/version");
		assert_eq!(version.numbers.major, 4);
		assert_eq!(version.numbers.minor, Some(3));
		assert_eq!(version.numbers.patch, None);
	}

	#[test]
	fn bench_timing() {
		use std::fs;

		let path =
			Path::new("/mnt/big_nvme/SteamLibrary/steamapps/common/MOLDRISE Demo/MOLDRISE.exe");
		if !path.is_file() {
			return;
		}

		let file_size = fs::metadata(path).unwrap().len();
		println!("\nFile size: {} MB", file_size / 1024 / 1024);

		let t0 = Instant::now();
		let _ = check_exe(path);
		let t = t0.elapsed();
		println!("Godot 4.x check: {:>8.2} ms", t.as_secs_f64() * 1000.0);

		let path2 = Path::new("/usr/bin/bash");
		if path2.is_file() {
			let t0 = Instant::now();
			let result = check_exe(path2);
			let t = t0.elapsed();
			println!(
				"Non-PE check:     {:>8.2} ms (result={:?})",
				t.as_secs_f64() * 1000.0,
				result.is_some()
			);
		}

		// PE32 non-Godot file (Unity) — should be very fast.
		let path3 = Path::new("/mnt/big_nvme/Unity/Hub/Editor/UnitySetup-4.6.1/Unity.exe");
		if path3.is_file() {
			let t0 = Instant::now();
			let result = check_exe(path3);
			let t = t0.elapsed();
			println!(
				"PE32 non-Godot:   {:>8.2} ms (result={:?})",
				t.as_secs_f64() * 1000.0,
				result.is_some()
			);
		}
	}
}
