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

/// Reads and scans the exe, returning the version if it's a Godot executable.
///
/// Uses a two-tier approach:
/// 1. Fast probe: read 1 MB at ~80% of the file, check for "Godot" or ".stable".
///    For non-Godot games this takes ~1ms and returns early.
/// 2. Full scan: only if the probe hits, read the last 50 MB and extract the version.
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

	// Tier 1: fast probe at ~80% of the file (where version strings reside).
	if !probe_for_godot(exe_path, file_len) {
		// If seek-based probe fails, fall back to checking if the file
		// contains "Godot" anywhere by reading the tail.
		if let Some(bytes) = read_tail(exe_path, file_len, 50 * 1024 * 1024) {
			if !contains_marker(&bytes) {
				return None;
			}
		} else {
			return None;
		}
	}

	// Tier 2: full version extraction.
	let bytes = read_tail(exe_path, file_len, 50 * 1024 * 1024)?;
	scan_for_version(&bytes)
}

/// Read 1 MB at 80% offset and check for Godot markers.
fn probe_for_godot(exe_path: &Path, file_len: u64) -> bool {
	let probe_size = 1 * 1024 * 1024; // 1 MB
	let offset = (file_len * 80 / 100).saturating_sub(probe_size as u64 / 2);

	let mut file = match fs::File::open(exe_path) {
		Ok(f) => f,
		Err(_) => return true, // Can't read — assume it might be Godot, let Tier 2 decide.
	};

	if file.seek(SeekFrom::Start(offset)).is_err() {
		return true; // Can't seek — assume it might be Godot.
	}

	let mut buf = vec![0u8; probe_size];
	let n = file.read(&mut buf).unwrap_or(0);
	if n == 0 {
		return true; // Can't read — assume it might be Godot.
	}

	contains_marker(&buf[..n])
}

/// Fast scan for Godot-identifying markers in a byte slice.
fn contains_marker(bytes: &[u8]) -> bool {
	let len = bytes.len();
	let mut i = 0;
	// Check for "Godot" (5 bytes, case-sensitive).
	while i + 5 <= len {
		if bytes[i] == b'G'
			&& bytes[i + 1] == b'o'
			&& bytes[i + 2] == b'd'
			&& bytes[i + 3] == b'o'
			&& bytes[i + 4] == b't'
		{
			return true;
		}
		i += 1;
	}
	// Check for ".stable" (7 bytes, Godot 3.x marker).
	i = 0;
	while i + 7 <= len {
		if bytes[i] == b'.'
			&& bytes[i + 1] == b's'
			&& bytes[i + 2] == b't'
			&& bytes[i + 3] == b'a'
			&& bytes[i + 4] == b'b'
			&& bytes[i + 5] == b'l'
			&& bytes[i + 6] == b'e'
		{
			return true;
		}
		i += 1;
	}
	false
}

fn read_tail(exe_path: &Path, file_len: u64, max_size: u64) -> Option<Vec<u8>> {
	let tail_size = max_size.min(file_len);
	let start_offset = file_len - tail_size;

	let mut file = fs::File::open(exe_path).ok()?;
	if file.seek(SeekFrom::Start(start_offset)).is_err() {
		// Fall back to full read.
		return fs::read(exe_path).ok();
	}

	let mut bytes = vec![0u8; tail_size as usize];
	file.read_exact(&mut bytes).ok()?;
	Some(bytes)
}

fn scan_for_version(bytes: &[u8]) -> Option<EngineVersion> {
	let len = bytes.len();

	// Scan for Godot 4.x: "Godot Engine v" or "Godot v".
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

	// Scan for Godot 3.x version markers.
	for marker in &[&b".stable"[..], b".beta", b".alpha", b".rc", b".dev"] {
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
		println!("Godot 4.x check: {:>8.2} ms", t.as_secs_f64() * 1000.0);

		// Test a non-Godot file
		let path2 = Path::new("/usr/bin/bash");
		if path2.is_file() {
			let t0 = Instant::now();
			let result = check_exe(path2);
			let t = t0.elapsed();
			println!("Non-Godot check: {:>8.2} ms (result={:?})", t.as_secs_f64() * 1000.0, result.is_some());
		}

		// Test a small file (where probe reads whole file)
		let path3 = Path::new("/usr/bin/ls");
		if path3.is_file() {
			let t0 = Instant::now();
			let result = check_exe(path3);
			let t = t0.elapsed();
			println!("Small non-Godot:  {:>8.2} ms (result={:?})", t.as_secs_f64() * 1000.0, result.is_some());
		}
	}
}
