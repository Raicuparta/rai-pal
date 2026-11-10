use fs_err::File;
use std::{io::Read, path::Path};

use rai_pal_proc_macros::serializable_enum;

use crate::result::LogErrExt;

#[serializable_enum]
pub enum OperatingSystem {
	Windows,
	Linux,
}

impl OperatingSystem {
	pub const ALL: [Self; 2] = [Self::Windows, Self::Linux];

	pub const fn get_current() -> Self {
		if cfg!(target_os = "windows") {
			Self::Windows
		} else {
			Self::Linux
		}
		// There are no other operating systems in the Universe.
	}
}

/// Infer the operating system an executable targets by reading its file header.
/// Windows executables are PE files (`MZ` magic), Linux executables are ELF files.
pub fn get_os_from_path(exe_path: &Path) -> Option<OperatingSystem> {
	let mut file = File::open(exe_path).ok_or_log(&format!(
		"Failed to open executable to detect OS: {}",
		exe_path.display()
	))?;

	let mut magic = [0u8; 4];
	if let Some(bytes_read) = file
		.read(&mut magic)
		.ok_or_log("Failed to read executable header to detect OS")
		&& bytes_read < 2
	{
		return None;
	}

	match magic {
		[0x4D, 0x5A, ..] => Some(OperatingSystem::Windows),
		[0x7F, b'E', b'L', b'F'] => Some(OperatingSystem::Linux),
		_ => None,
	}
}
