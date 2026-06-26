use std::path::Path;

use lazy_regex::regex_find;

use super::{
	game_engine::{
		EngineBrand,
		EngineVersion,
		EngineVersionNumbers,
	},
	pe_utils,
};
use crate::{
	game::DbGame,
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

fn check_exe(exe_path: &Path) -> Option<EngineVersion> {
	// Read only the PE headers and export DLL name from disk (a few KB).
	let mut file = std::fs::File::open(exe_path).ok()?;
	let metadata = pe_utils::read_pe_metadata(&mut file)?;

	// Fast filter: not a Godot export → return early, no full file read.
	if !metadata.is_godot_export() {
		return None;
	}

	// Looks like a Godot game — read only the .rdata section from the same file
	// handle and scan it for version strings.
	let rdata = metadata.find_section(".rdata")?;
	let section_bytes = pe_utils::read_section_bytes(&mut file, rdata)?;
	scan_for_version(&section_bytes)
}

pub fn process_game(game: &mut DbGame) -> bool {
	let Some(exe_path) = game.exe_path.as_ref() else {
		return false;
	};

	let Some(version) = check_exe(exe_path) else {
		return false;
	};

	game.engine_brand = Some(EngineBrand::Godot);
	game.engine_version_major = Some(version.numbers.major);
	game.engine_version_minor = version.numbers.minor;
	game.engine_version_patch = version.numbers.patch;
	game.engine_version_display = Some(version.display);

	true
}
