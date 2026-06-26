use pelite::{
	image::{
		IMAGE_DATA_DIRECTORY,
		IMAGE_DIRECTORY_ENTRY_EXPORT,
		IMAGE_EXPORT_DIRECTORY,
	},
	pe64::headers::SectionHeaders,
};

use crate::result::LogErrExt;

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

	let exp_dir_end = export_offset.checked_add(std::mem::size_of::<IMAGE_EXPORT_DIRECTORY>())?;
	let exp_dir_bytes = file_bytes.get(export_offset..exp_dir_end)?;
	let exp_dir = unsafe { &*(exp_dir_bytes.as_ptr() as *const IMAGE_EXPORT_DIRECTORY) };

	let name_offset = rva_to_file_offset(sections, exp_dir.Name)?;
	read_cstr(file_bytes, name_offset)
}
