use std::{
	fs::{
		self,
	},
	path::Path,
};

use lazy_regex::{
	regex_captures,
	regex_find,
};
use log::error;
use pelite::{
	pe32::{
		Pe as Pe32,
		PeFile as PeFile32,
	},
	pe64::{
		Pe as Pe64,
		PeFile as PeFile64,
	},
};
use serde_json;

use super::game_engine::EngineVersionNumbers;
use crate::{
	architecture::{
		Architecture,
		get_architecture,
	},
	data_types::path_data::PathData,
	game::DbGame,
	game_engines::game_engine::EngineVersion,
	paths::glob_path,
};

fn get_version_from_metadata(
	file_bytes: &[u8],
	architecture: Architecture,
) -> Option<EngineVersion> {
	let version = if architecture == Architecture::X86 {
		PeFile32::from_bytes(&file_bytes)
			.ok()?
			.resources()
			.ok()?
			.version_info()
			.ok()?
			.fixed()?
	} else {
		PeFile64::from_bytes(&file_bytes)
			.ok()?
			.resources()
			.ok()?
			.version_info()
			.ok()?
			.fixed()?
	};

	let major = u32::from(version.dwFileVersion.Major);
	let minor = u32::from(version.dwFileVersion.Minor);
	let patch = u32::from(version.dwFileVersion.Patch);

	Some(EngineVersion {
		numbers: EngineVersionNumbers {
			major,
			minor: Some(minor),
			patch: Some(patch),
		},
		suffix: None,
		display: format!("{major}.{minor}.{patch}"),
	})
}

fn try_get_version_from_metadata(file_bytes: &[u8]) -> Option<EngineVersion> {
	// Try x64 first
	if let Some(version) = get_version_from_metadata(file_bytes, Architecture::X64) {
		return Some(version);
	}
	// Try x86
	get_version_from_metadata(file_bytes, Architecture::X86)
}

fn find_game_root(exe_path: &Path) -> &Path {
	let try_find = || -> Option<&Path> {
		let win_dir = exe_path.parent()?;
		if !is_valid_win_folder(win_dir) {
			return None;
		}
		let binaries_dir = win_dir.parent()?;
		if !binaries_dir.ends_with("Binaries") {
			return None;
		}
		let module_dir = binaries_dir.parent()?;
		let game_root = module_dir.parent()?;
		if game_root.join("Engine").is_dir() {
			return Some(game_root);
		}
		None
	};

	try_find().unwrap_or_else(|| exe_path.parent().unwrap_or(exe_path))
}

fn get_version_from_build_version_file(exe_path: &Path) -> Option<EngineVersion> {
	let game_root = find_game_root(exe_path);

	let build_version_path = glob_path(&game_root.join("**/Build.version"))
		.into_iter()
		.next()?;

	let content = fs::read_to_string(&build_version_path).ok()?;
	let value = serde_json::from_str::<serde_json::Value>(&content).ok()?;
	let (Some(major), Some(minor), Some(patch), Some(changelist)) = (
		value["MajorVersion"].as_u64(),
		value["MinorVersion"].as_u64(),
		value["PatchVersion"].as_u64(),
		value["Changelist"].as_u64(),
	) else {
		return None;
	};

	let display = format!("{major}.{minor}.{patch}.{changelist}");
	Some(EngineVersion {
		numbers: EngineVersionNumbers {
			major: u32::try_from(major).ok()?,
			minor: u32::try_from(minor).ok(),
			patch: u32::try_from(patch).ok(),
		},
		suffix: None,
		display,
	})
}

fn get_version_from_exe_path(exe_path: &Path) -> Option<EngineVersion> {
	match fs::read(exe_path) {
		Ok(file_bytes) => {
			// Try metadata first
			if let Some(version) = try_get_version_from_metadata(&file_bytes) {
				return Some(version);
			}
			// Then try parsing exe strings
			get_version_from_exe_parse(&file_bytes)
		}
		Err(err) => {
			error!(
				"Failed to read exe `{}`. Error: {}",
				exe_path.display(),
				err
			);
			None
		}
	}
}

fn parse_version(string: &str) -> Option<EngineVersion> {
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
		&string
	)?;

	Some(EngineVersion {
		numbers: EngineVersionNumbers {
			major: major.parse().unwrap_or(0),
			minor: minor.parse().ok(),
			patch: None,
		},
		suffix: None,
		display: format!("{major}{}", {
			if minor.is_empty() {
				String::new()
			} else {
				format!(".{minor}")
			}
		}),
	})
}

fn get_version_from_exe_parse(file_bytes: &[u8]) -> Option<EngineVersion> {
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

	// Parse again because the byte regex above can't extract the match groups.
	parse_version(&match_string)
}

fn is_valid_win_folder(path: &Path) -> bool {
	path.ends_with("Win64") || path.ends_with("Win32") || path.ends_with("WinGDK")
}

pub fn is_unreal_exe(game_path: &Path) -> bool {
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
		if is_valid_win_folder(parent)
			&& let Some(binaries) = parent.parent()
			&& binaries.ends_with("Binaries")
		{
			return true;
		}
	}

	false
}

pub fn process_game(game: &mut DbGame) {
	if let Some(PathData(game_path)) = game.exe_path.as_ref() {
		let game_dir = game_path.parent().unwrap_or(game_path);

		// 1. Try Build.version file (most reliable)
		if let Some(version) = get_version_from_build_version_file(game_path) {
			game.engine_version_major = Some(version.numbers.major);
			game.engine_version_minor = version.numbers.minor;
			game.engine_version_patch = version.numbers.patch;
			game.engine_version_display = Some(version.display);
			return;
		}

		// 2-3. Try specific known launcher/helper executables (CrashReportClient.exe, EpicWebHelper.exe)
		for filename in &["CrashReportClient.exe", "EpicWebHelper.exe"] {
			let exe_paths = glob_path(&game_dir.join(format!("**/{filename}")));
			if let Some(exe_path) = exe_paths.first()
				&& let Some(version) = get_version_from_exe_path(exe_path)
			{
				game.engine_version_major = Some(version.numbers.major);
				game.engine_version_minor = version.numbers.minor;
				game.engine_version_patch = version.numbers.patch;
				game.engine_version_display = Some(version.display);
				game.architecture = get_architecture(exe_path).unwrap_or(None);
				game.exe_path = Some(PathData(exe_path.clone()));
				return;
			}
		}

		// 4. Try *-Shipping.exe (build stamp and metadata)
		if let Some(shipping_exe_path) = glob_path(&game_dir.join("**/*-Shipping.exe"))
			.first()
			.cloned() && let Some(version) = get_version_from_exe_path(&shipping_exe_path)
		{
			game.engine_version_major = Some(version.numbers.major);
			game.engine_version_minor = version.numbers.minor;
			game.engine_version_patch = version.numbers.patch;
			game.engine_version_display = Some(version.display);
			game.architecture = get_architecture(&shipping_exe_path).unwrap_or(None);
			game.exe_path = Some(PathData(shipping_exe_path));
			return;
		}

		// 5-6. Try any .exe files
		for exe_path in glob_path(&game_dir.join("**/*.exe")) {
			if let Some(version) = get_version_from_exe_path(&exe_path) {
				game.engine_version_major = Some(version.numbers.major);
				game.engine_version_minor = version.numbers.minor;
				game.engine_version_patch = version.numbers.patch;
				game.engine_version_display = Some(version.display);
				game.architecture = get_architecture(&exe_path).unwrap_or(None);
				game.exe_path = Some(PathData(exe_path));
				return;
			}
		}
	}
}
