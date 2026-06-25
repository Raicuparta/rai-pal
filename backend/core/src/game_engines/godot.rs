use std::{
	fs,
	io::{
		Read,
		Seek,
		SeekFrom,
	},
	path::Path,
};

use log::error;

use super::game_engine::EngineVersionNumbers;
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

/// Reads the exe and returns the engine version if it's a Godot executable.
///
/// Godot version strings live near the end of the file (in .rdata section),
/// so we only scan the last portion to minimize I/O and scan time.
pub fn check_exe(exe_path: &Path) -> Option<EngineVersion> {
	let metadata = match fs::metadata(exe_path) {
		Ok(m) => m,
		Err(err) => {
			error!("Failed to stat `{}`. Error: {}", exe_path.display(), err);
			return None;
		}
	};

	let file_len = metadata.len();
	if file_len == 0 {
		return None;
	}

	// Read only the last 50 MB (or the whole file if smaller).
	// Version strings and Godot markers consistently appear in the last ~30%.
	let tail_size = (50 * 1024 * 1024).min(file_len);
	let start_offset = file_len - tail_size;

	let mut file = match fs::File::open(exe_path) {
		Ok(f) => f,
		Err(err) => {
			error!(
				"Failed to open exe `{}`. Error: {}",
				exe_path.display(),
				err
			);
			return None;
		}
	};

	if file.seek(SeekFrom::Start(start_offset)).is_err() {
		// If seek fails, fall back to full read.
		let bytes = match fs::read(exe_path) {
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
		return scan_for_godot(&bytes);
	}

	let mut bytes = vec![0u8; tail_size as usize];
	if file.read_exact(&mut bytes).is_err() {
		return None;
	}

	scan_for_godot(&bytes)
}

fn scan_for_godot(bytes: &[u8]) -> Option<EngineVersion> {
	let len = bytes.len();

	// --- Scan for Godot 4.x: "Godot Engine v" or "Godot v" ---
	let mut i = 0;
	while i + 14 <= len {
		if bytes[i] != b'G' {
			i += 1;
			continue;
		}
		if bytes[i + 1] != b'o'
			|| bytes[i + 2] != b'd'
			|| bytes[i + 3] != b'o'
			|| bytes[i + 4] != b't'
		{
			i += 1;
			continue;
		}
		if bytes[i + 5] != b' ' {
			i += 1;
			continue;
		}
		let b6 = bytes[i + 6];
		if (b6 == b'e' || b6 == b'E')
			&& (bytes[i + 7] == b'n' || bytes[i + 7] == b'N')
			&& (bytes[i + 8] == b'g' || bytes[i + 8] == b'G')
			&& (bytes[i + 9] == b'i' || bytes[i + 9] == b'I')
			&& (bytes[i + 10] == b'n' || bytes[i + 10] == b'N')
			&& (bytes[i + 11] == b'e' || bytes[i + 11] == b'E')
			&& bytes[i + 12] == b' '
			&& (bytes[i + 13] == b'v' || bytes[i + 13] == b'V')
		{
			let end = (i + 80).min(len);
			let s = String::from_utf8_lossy(&bytes[i..end]);
			return parse_version(&s);
		}
		if b6 == b'v' || b6 == b'V' {
			let end = (i + 60).min(len);
			let s = String::from_utf8_lossy(&bytes[i..end]);
			return parse_version(&s);
		}
		i += 1;
	}

	// --- Scan for Godot 3.x version markers ---
	let markers: &[&[u8]] = &[b".stable", b".beta", b".alpha", b".rc", b".dev"];
	for marker in markers {
		let mut i = 0;
		let m = marker.len();
		let first = marker[0];
		while i + m <= len {
			if bytes[i] != first {
				i += 1;
				continue;
			}
			let mut matched = true;
			for j in 1..m {
				if bytes[i + j] != marker[j] {
					matched = false;
					break;
				}
			}
			if matched {
				let start = i.saturating_sub(20);
				let end = (i + m + 20).min(len);
				let s = String::from_utf8_lossy(&bytes[start..end]);
				if let Some(ver) = parse_version(&s) {
					return Some(ver);
				}
			}
			i += 1;
		}
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
	use std::path::Path;
	use std::time::Instant;

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
	fn bench_timing() {
		use std::fs;

		let path = Path::new("/mnt/big_nvme/SteamLibrary/steamapps/common/MOLDRISE Demo/MOLDRISE.exe");
		if !path.is_file() {
			return;
		}

		let file_size = fs::metadata(path).unwrap().len();
		println!("\nFile size: {} MB", file_size / 1024 / 1024);

		let t0 = Instant::now();
		let _ = check_exe(path);
		let t = t0.elapsed();
		println!("check_exe total: {:>8.2} ms", t.as_secs_f64() * 1000.0);
	}
}
