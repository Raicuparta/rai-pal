use std::{
	fs::{self},
	path::{Path, PathBuf},
};

use lazy_regex::{regex_captures, regex_find};
use log::error;
use pelite::{
	pe32::{Pe as Pe32, PeFile as PeFile32},
	pe64::{Pe as Pe64, PeFile as PeFile64},
};

use super::game_engine::EngineVersionNumbers;
use crate::{
	architecture::{Architecture, get_architecture},
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

fn get_version(path: &Path, architecture: Architecture) -> Option<EngineVersion> {
	match fs::read(path) {
		Ok(file_bytes) => {
			return get_version_from_metadata(&file_bytes, architecture)
				.or_else(|| get_version_from_exe_parse(&file_bytes));
		}
		Err(err) => {
			error!(
				"Failed to read game exe `{}`. Error: {}",
				path.display(),
				err
			);
		}
	}

	None
}

// The shipping exe is usually in a Win* folder.
fn is_valid_win_folder(path: &Path) -> bool {
	path.ends_with("Win64") || path.ends_with("Win32") || path.ends_with("WinGDK")
}

// Some games have multiple exes in the same folder.
// The exe names can be anything, but it's common to have a launcher exe,
// next to a *-Shipping.exe, which is usually the one we want.
fn is_shipping_exe(path: &Path) -> bool {
	path.file_name()
		.and_then(|file_name| file_name.to_str())
		.is_some_and(|file_name| file_name.ends_with("Shipping.exe"))
}

// Unreal games often ship with extra launcher exes that we don't care about.
// We need the actual exe built by Unreal Engine to be able to find the engine version.
// Usually, the exe we want would have a name like Game-Name-Win64-Shipping.exe, but not always.
// Unfortunately there are no precise rules for this, so there's a lot of guesswork involved.
fn get_shipping_exe(game_exe_path: &Path) -> PathBuf {
	if let Some(parent) = game_exe_path.parent() {
		if is_valid_win_folder(parent) {
			if is_shipping_exe(game_exe_path) {
				// Case where given exe is the shipping exe.
				return game_exe_path.to_path_buf();
			}

			if let Some(sibling_shipping_exe) =
				glob_path(&parent.join("*Shipping.exe")).first().cloned()
			{
				// Case where given exe is a sibling of the shipping exe.
				return sibling_shipping_exe;
			}

			// Case where the given exe isn't a shipping exe,
			// but doesn't have a shipping exe sibling.
			// We just presume the given exe is good enough.
			return game_exe_path.to_path_buf();
		}

		// From here, we start presuming that the given exe is a launcher at the root level,
		// and we need to dig down to find the shipping exe.
		let globbed_paths = glob_path(
			&parent
				// This portion of the path would usually be the game's name, but no way to guess that.
				// We know it's not "Engine", but can't exclude with the rust glob crate (we filter it below).
				.join("*")
				.join("Binaries")
				.join("Win{64,32,GDK}")
				// The file name may or may not end with Shipping.exe, so we don't test for that yet.
				.join("*.exe"),
		);

		let mut suitable_paths = globbed_paths.iter().filter(|path| {
			// The Engine folder can have similar structure, but it's not the one we want.
			!path.starts_with(parent.join("Engine"))
		});

		let first_path = suitable_paths.next();
		if let Some(best_path) = suitable_paths
			// Exe that looks like a shipping exe takes priority.
			.find(|path| is_shipping_exe(path))
			.or(first_path)
		{
			return best_path.clone();
		}
	}

	// If nothing works, just fall back to the given exe.
	game_exe_path.to_path_buf()
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
		if is_valid_win_folder(parent) {
			if let Some(binaries) = parent.parent() {
				if binaries.ends_with("Binaries") {
					return true;
				}
			}
		}
	}

	false
}

pub fn process_game(game: &mut DbGame) {
	if let Some(PathData(launch_path)) = game.exe_path.as_ref() {
		let shipping_exe_path = get_shipping_exe(launch_path);
		game.architecture = get_architecture(&shipping_exe_path).unwrap_or(None);

		if let Some(version) = get_version(
			&shipping_exe_path,
			game.architecture.unwrap_or(Architecture::X64),
		) {
			game.engine_version_major = Some(version.numbers.major);
			game.engine_version_minor = version.numbers.minor;
			game.engine_version_patch = version.numbers.patch;
			game.engine_version_display = Some(version.display);
		}

		game.exe_path = Some(PathData(shipping_exe_path));
	}
}
