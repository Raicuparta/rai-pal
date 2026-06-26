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
	// skip the expensive section scan. If we can't read it at all, also skip — it's not
	// a Godot game.
	let name = pe_utils::try_read_export_dll_name(sections, data_dir, file_bytes)?;
	if !name.to_ascii_lowercase().starts_with("godot") {
		return None;
	}

	if let Some(sec) = pe.section_headers().by_name(".rdata") {
		return pe
			.get_section_bytes(sec)
			.ok_or_log("Failed to get PE section bytes")
			.and_then(scan_for_version);
	}
	None
}

fn check_pe32(pe: &PeFile32<'_>, file_bytes: &[u8]) -> Option<EngineVersion> {
	let sections = pe.section_headers();
	let data_dir = pe.data_directory();

	let name = pe_utils::try_read_export_dll_name(sections, data_dir, file_bytes)?;
	if !name.to_ascii_lowercase().starts_with("godot") {
		return None;
	}

	if let Some(sec) = pe.section_headers().by_name(".rdata") {
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
