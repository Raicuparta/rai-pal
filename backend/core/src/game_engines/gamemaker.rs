use std::path::Path;

use pelite::PeFile;

use super::game_engine::{
	EngineBrand,
	EngineVersion,
	EngineVersionNumbers,
};
use crate::{
	game::DbGame,
	result::LogErrExt,
};

fn read_gen8_version(form_offset: usize, data: &[u8]) -> Option<EngineVersion> {
	if data.len() < form_offset + 76 {
		return None;
	}

	let slice = &data[form_offset..];

	if slice.get(0..4) != Some(b"FORM") || slice.get(8..12) != Some(b"GEN8") {
		return None;
	}

	let major = u32::from_le_bytes(
		slice[60..64]
			.try_into()
			.ok_or_log("Failed to read GameMaker major version")?,
	);

	// GEN8 stores the data format version, not the IDE version.
	// For GM:S 1.x the version is 1.0.0.xxxx (build varies).
	// For GM:S 2.x the version is consistently 2.0.0.0.
	// Only the major digit is meaningful.
	let display = major.to_string();

	Some(EngineVersion {
		display,
		numbers: EngineVersionNumbers {
			major,
			minor: None,
			patch: None,
		},
		suffix: None,
	})
}

fn try_read_version_from_pe_sections(overlay_start: usize, data: &[u8]) -> Option<EngineVersion> {
	let pe_data = &data[..overlay_start.min(data.len())];

	for i in 0..pe_data.len().saturating_sub(76) {
		if let Some(version) = read_gen8_version(i, pe_data) {
			return Some(version);
		}
	}

	None
}

fn try_read_version_from_overlay(overlay_start: usize, data: &[u8]) -> Option<EngineVersion> {
	if data.len() < overlay_start + 76 {
		return None;
	}

	let overlay = &data[overlay_start..];
	read_gen8_version(0, overlay)
}

fn try_read_version_from_data_win(exe_path: &Path) -> Option<EngineVersion> {
	let data_win_path = exe_path.parent()?.join("data.win");
	if !data_win_path.is_file() {
		return None;
	}

	let mmap = crate::game_engines::mmap_safe::map_readonly(&data_win_path)
		.ok_or_log("Failed to memory map data.win")?;

	read_gen8_version(0, &mmap)
}

fn is_gamemaker_studio_1(data: &[u8]) -> bool {
	const MARKERS: &[&[u8]] = &[b"GML_STUB_ASSERT", b"Initializing GML"];
	MARKERS
		.iter()
		.any(|marker| data.windows(marker.len()).any(|w| w == *marker))
}

fn is_gamemaker_legacy(data: &[u8]) -> bool {
	const MARKERS: &[&[u8]] = &[b"GameMaker", b"YoYo Games", b"gamemaker"];
	MARKERS
		.iter()
		.any(|marker| data.windows(marker.len()).any(|w| w == *marker))
}

fn has_gamemaker_metadata(data: &[u8]) -> bool {
	is_gamemaker_studio_1(data) || is_gamemaker_legacy(data)
}

#[allow(dead_code)]
fn check_exe(exe_path: &Path) -> Option<EngineVersion> {
	let mmap = crate::game_engines::mmap_safe::map_readonly(exe_path)
		.ok_or_log("Failed to memory map GameMaker exe")?;

	let pe = PeFile::from_bytes(&mmap).ok_or_log("Failed to parse PE file")?;

	let mut end_of_sections = 0;
	for section in pe.section_headers() {
		let section_end = section.PointerToRawData as usize + section.SizeOfRawData as usize;
		if section_end > end_of_sections {
			end_of_sections = section_end;
		}
	}

	let overlay_start = end_of_sections.min(mmap.len());

	if let Some(version) = try_read_version_from_overlay(overlay_start, &mmap) {
		return Some(version);
	}

	if let Some(version) = try_read_version_from_data_win(exe_path) {
		return Some(version);
	}

	if let Some(version) = try_read_version_from_pe_sections(overlay_start, &mmap) {
		return Some(version);
	}

	None
}

pub fn process_game(game: &mut DbGame) -> bool {
	let Some(exe_path) = game.exe_path.as_ref() else {
		return false;
	};

	let Ok(mmap) = crate::game_engines::mmap_safe::map_readonly(exe_path) else {
		return false;
	};

	let Ok(pe) = PeFile::from_bytes(&mmap) else {
		return false;
	};

	let mut end_of_sections = 0;
	for section in pe.section_headers() {
		let section_end = section.PointerToRawData as usize + section.SizeOfRawData as usize;
		if section_end > end_of_sections {
			end_of_sections = section_end;
		}
	}

	let overlay_start = end_of_sections.min(mmap.len());

	let version = try_read_version_from_overlay(overlay_start, &mmap)
		.or_else(|| try_read_version_from_data_win(exe_path))
		.or_else(|| try_read_version_from_pe_sections(overlay_start, &mmap));

	let pe_data = &mmap[..overlay_start];

	if version.is_none() && !has_gamemaker_metadata(pe_data) {
		return false;
	}

	game.engine_brand = Some(EngineBrand::GameMaker);

	if let Some(version) = version {
		game.engine_version_major = Some(version.numbers.major);
		game.engine_version_minor = version.numbers.minor;
		game.engine_version_patch = version.numbers.patch;
		game.engine_version_display = Some(version.display);
	} else if is_gamemaker_studio_1(pe_data) || is_gamemaker_legacy(pe_data) {
		let display = "1".to_string();
		game.engine_version_major = Some(1);
		game.engine_version_display = Some(display);
	}

	true
}
