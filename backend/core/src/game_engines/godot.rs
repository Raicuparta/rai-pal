use std::path::Path;

use lazy_regex::regex_find;
use pelite::PeFile;

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
	open_better::open_detached_better,
	path_extensions::PathExt,
	result::{
		LogErrExt,
		Result,
	},
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
	// Memory-map the file. The OS loads pages lazily, so only the headers
	// (a few KB) are actually fetched from disk for the fast filter. If the
	// game turns out to be a Godot game, the .rdata pages are loaded on
	// demand during the section scan.
	let mmap = crate::game_engines::mmap_safe::map_readonly(exe_path)
		.ok_or_log("Failed to memory map Godot exe")?;

	let pe = PeFile::from_bytes(&mmap).ok_or_log("Failed to parse PE file")?;
	let sections = pe.section_headers();
	let data_dir = pe.data_directory();

	let name = pe_utils::try_read_export_dll_name(sections.image(), data_dir, &mmap)?;
	if !name.to_ascii_lowercase().starts_with("godot") {
		return None;
	}

	if let Some(sec) = sections.by_name(".rdata") {
		return pe
			.get_section_bytes(sec)
			.ok_or_log("Failed to get PE section bytes")
			.and_then(scan_for_version);
	}
	None
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

pub fn open_data_folder(game: &DbGame) -> Result {
	let app_data = game.get_roaming_app_data_slow()?;
	let default_godot_data = game.get_roaming_app_data_slow()?.join("Godot/app_userdata");

	let target = if default_godot_data.exists() {
		let potential_game_folder =
			default_godot_data.join(game.try_get_exe_path()?.file_name_without_extension()?);

		if potential_game_folder.exists() {
			potential_game_folder
		} else {
			default_godot_data
		}
	} else {
		app_data
	};

	open_detached_better(target)
}
