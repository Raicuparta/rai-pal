use std::{
	fs::File,
	io::{
		Read,
		Seek,
		SeekFrom,
	},
	path::Path,
};

use pelite::{
	image::{
		IMAGE_DATA_DIRECTORY,
		IMAGE_DIRECTORY_ENTRY_EXPORT,
		IMAGE_EXPORT_DIRECTORY,
	},
	pe64::headers::SectionHeaders,
};

use crate::result::LogErrExt;

// Size of IMAGE_SECTION_HEADER in bytes.
const SECTION_HEADER_SIZE: usize = 40;

// A parsed PE section header, with the fields we care about.
#[derive(Debug, Clone)]
pub struct SectionInfo {
	pub name: [u8; 8],
	pub virtual_size: u32,
	pub virtual_address: u32,
	pub size_of_raw_data: u32,
	pub pointer_to_raw_data: u32,
}

impl SectionInfo {
	pub fn name_str(&self) -> &str {
		let end = self.name.iter().position(|&b| b == 0).unwrap_or(8);
		std::str::from_utf8(&self.name[..end]).unwrap_or("")
	}
}

// All the PE metadata we need from the headers — sections and the export DLL name.
#[derive(Debug, Clone)]
pub struct PeMetadata {
	pub export_dll_name: String,
	pub sections: Vec<SectionInfo>,
}

impl PeMetadata {
	// Whether the export DLL name starts with "godot" (case-insensitive).
	pub fn is_godot_export(&self) -> bool {
		self.export_dll_name
			.to_ascii_lowercase()
			.starts_with("godot")
	}

	// Find a section by name (e.g. ".rdata"). Returns `None` if no section has
	// that name.
	pub fn find_section(&self, name: &str) -> Option<&SectionInfo> {
		self.sections.iter().find(|sec| sec.name_str() == name)
	}
}

// ---------------------------------------------------------------------------
// Slice-based helpers (used by the pelite-based path, kept for compatibility)
// ---------------------------------------------------------------------------

// Resolve an RVA to a file offset using VirtualSize-based bounds (not SizeOfRawData).
// This avoids pelite's `cmp::max(VirtualSize, SizeOfRawData)` which incorrectly maps
// RVAs into sections like Godot's `pck` section that have tiny VirtualSize but huge raw data.
fn rva_to_file_offset(sections: &SectionHeaders, rva: u32) -> Option<usize> {
	let sec = sections.by_rva(rva)?;
	let offset_within = rva.checked_sub(sec.VirtualAddress)?;
	let file_offset = sec.PointerToRawData as usize + offset_within as usize;
	Some(file_offset)
}

// Read a null-terminated string from `file_bytes` starting at `offset`.
fn read_cstr(file_bytes: &[u8], offset: usize) -> Option<&str> {
	let remaining = file_bytes.get(offset..)?;
	let len = remaining.iter().position(|&b| b == 0)?;
	std::str::from_utf8(&remaining[..len]).ok_or_log("Failed to read cstr")
}

// Try to read the export DLL name using correct RVA-to-file-offset mapping.
// Returns None if the export directory doesn't exist or can't be read.
pub fn try_read_export_dll_name<'a>(
	sections: &SectionHeaders,
	data_dir: &[IMAGE_DATA_DIRECTORY],
	file_bytes: &'a [u8],
) -> Option<&'a str> {
	let export_entry = data_dir.get(IMAGE_DIRECTORY_ENTRY_EXPORT)?;
	let export_rva = export_entry.VirtualAddress;
	if export_rva == 0 {
		return None;
	}

	let export_offset = rva_to_file_offset(sections, export_rva)?;

	// Read just the Name field from the export directory struct
	let exp_dir_end = export_offset.checked_add(std::mem::size_of::<IMAGE_EXPORT_DIRECTORY>())?;
	let exp_dir_bytes = file_bytes.get(export_offset..exp_dir_end)?;
	let name_offset_in_struct = std::mem::offset_of!(IMAGE_EXPORT_DIRECTORY, Name);
	let name_rva_bytes = exp_dir_bytes.get(name_offset_in_struct..name_offset_in_struct + 4)?;
	let Ok(name_rva_bytes): Result<[u8; 4], _> = name_rva_bytes.try_into() else {
		return None;
	};
	let name_rva = u32::from_le_bytes(name_rva_bytes);

	let name_offset = rva_to_file_offset(sections, name_rva)?;
	read_cstr(file_bytes, name_offset)
}

// ---------------------------------------------------------------------------
// File-based (surgical) helpers — read only what's needed from disk.
// ---------------------------------------------------------------------------

// Read the PE metadata from an open file: section headers and export DLL name.
// The file position after this call is unspecified (caller should seek).
pub fn read_pe_metadata(file: &mut File) -> Option<PeMetadata> {
	// -- 1. DOS header --
	let mut dos_hdr = [0u8; 64];
	file.read_exact(&mut dos_hdr).ok()?;
	if dos_hdr[..2] != [b'M', b'Z'] {
		return None;
	}
	let e_lfanew = u32::from_le_bytes(dos_hdr[0x3C..0x40].try_into().ok()?);

	// -- 2. PE signature + COFF file header (24 bytes total) --
	file.seek(SeekFrom::Start(e_lfanew as u64)).ok()?;
	let mut pe_hdr = [0u8; 24];
	file.read_exact(&mut pe_hdr).ok()?;

	if pe_hdr[..4] != [b'P', b'E', b'\0', b'\0'] {
		return None;
	}

	let num_sections = u16::from_le_bytes(pe_hdr[6..8].try_into().ok()?);
	let size_of_opt_header = u16::from_le_bytes(pe_hdr[20..22].try_into().ok()?);

	// -- 3. Optional header (contains data directories) --
	let mut opt_header = vec![0u8; size_of_opt_header as usize];
	file.read_exact(&mut opt_header).ok()?;

	let magic = u16::from_le_bytes(opt_header[..2].try_into().ok()?);

	// Offset of NumberOfRvaAndSizes within the optional header differs between
	// PE32 (0x10b) and PE32+ (0x20b).
	let num_rva_sizes_offset = match magic {
		0x10b => 92,  // PE32
		0x20b => 108, // PE32+
		_ => return None,
	};

	let num_rva_sizes = u32::from_le_bytes(
		opt_header[num_rva_sizes_offset..num_rva_sizes_offset + 4]
			.try_into()
			.ok()?,
	);
	let data_dir_offset = num_rva_sizes_offset + 4;

	// Export directory is entry 0 in the data directory array.
	let export_entry_offset = data_dir_offset;
	if export_entry_offset + 8 > opt_header.len() || num_rva_sizes == 0 {
		return None;
	}
	let export_rva = u32::from_le_bytes(
		opt_header[export_entry_offset..export_entry_offset + 4]
			.try_into()
			.ok()?,
	);
	if export_rva == 0 {
		return None;
	}

	// -- 4. Section headers (40 bytes each) --
	let section_hdrs_size = num_sections as usize * SECTION_HEADER_SIZE;
	let mut section_hdrs_raw = vec![0u8; section_hdrs_size];
	file.read_exact(&mut section_hdrs_raw).ok()?;

	let sections = parse_section_headers(&section_hdrs_raw, num_sections);

	// -- 5. Read the export directory from the file at the correct offset --
	let export_file_offset = rva_to_file_offset_raw(&section_hdrs_raw, num_sections, export_rva)?;
	file.seek(SeekFrom::Start(export_file_offset as u64)).ok()?;

	let exp_dir_size = std::mem::size_of::<IMAGE_EXPORT_DIRECTORY>();
	let mut exp_dir_buf = vec![0u8; exp_dir_size];
	file.read_exact(&mut exp_dir_buf).ok()?;

	// Name field is at offset 12 in IMAGE_EXPORT_DIRECTORY (a u32 RVA).
	let name_rva = u32::from_le_bytes(exp_dir_buf[12..16].try_into().ok()?);

	// -- 6. Read the export name string --
	let name_file_offset = rva_to_file_offset_raw(&section_hdrs_raw, num_sections, name_rva)?;
	file.seek(SeekFrom::Start(name_file_offset as u64)).ok()?;

	let mut name_bytes = Vec::new();
	let mut byte = [0u8; 1];
	loop {
		file.read_exact(&mut byte).ok()?;
		if byte[0] == 0 {
			break;
		}
		name_bytes.push(byte[0]);
	}

	let export_dll_name = String::from_utf8(name_bytes).ok()?;

	Some(PeMetadata {
		export_dll_name,
		sections,
	})
}

// Read the raw bytes of a section from the file (just its `SizeOfRawData`).
// The file position after this call is unspecified.
pub fn read_section_bytes(file: &mut File, section: &SectionInfo) -> Option<Vec<u8>> {
	file.seek(SeekFrom::Start(section.pointer_to_raw_data as u64))
		.ok()?;
	let mut buf = vec![0u8; section.size_of_raw_data as usize];
	file.read_exact(&mut buf).ok()?;
	Some(buf)
}

// Read the export DLL name from a PE file path without loading the entire file
// into memory.
pub fn try_read_export_dll_name_from_file(exe_path: &Path) -> Option<String> {
	let mut file = File::open(exe_path).ok()?;
	read_pe_metadata(&mut file).map(|m| m.export_dll_name)
}

// Read the export DLL name from an already-opened PE file.
pub fn try_read_export_dll_name_from_file_impl(file: &mut File) -> Option<String> {
	read_pe_metadata(file).map(|m| m.export_dll_name)
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

fn parse_section_headers(raw: &[u8], num_sections: u16) -> Vec<SectionInfo> {
	(0..num_sections as usize)
		.filter_map(|i| {
			let off = i * SECTION_HEADER_SIZE;
			if off + 40 > raw.len() {
				return None;
			}
			let mut name = [0u8; 8];
			name.copy_from_slice(&raw[off..off + 8]);
			Some(SectionInfo {
				name,
				virtual_size: u32::from_le_bytes(raw[off + 8..off + 12].try_into().ok()?),
				virtual_address: u32::from_le_bytes(raw[off + 12..off + 16].try_into().ok()?),
				size_of_raw_data: u32::from_le_bytes(raw[off + 16..off + 20].try_into().ok()?),
				pointer_to_raw_data: u32::from_le_bytes(raw[off + 20..off + 24].try_into().ok()?),
			})
		})
		.collect()
}

// RVA → file offset using a raw byte slice of section headers (40 bytes each).
// Uses VirtualSize for bounds (same logic as `rva_to_file_offset`).
fn rva_to_file_offset_raw(section_hdrs: &[u8], num_sections: u16, rva: u32) -> Option<usize> {
	for i in 0..num_sections as usize {
		let off = i * SECTION_HEADER_SIZE;
		if off + 24 > section_hdrs.len() {
			return None;
		}
		let virtual_size = u32::from_le_bytes(section_hdrs[off + 8..off + 12].try_into().ok()?);
		let virtual_address = u32::from_le_bytes(section_hdrs[off + 12..off + 16].try_into().ok()?);
		let pointer_to_raw_data =
			u32::from_le_bytes(section_hdrs[off + 20..off + 24].try_into().ok()?);

		if rva >= virtual_address && rva < virtual_address + virtual_size {
			let offset_within = rva - virtual_address;
			return Some(pointer_to_raw_data as usize + offset_within as usize);
		}
	}
	None
}
