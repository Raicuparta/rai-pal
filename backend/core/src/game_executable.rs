use std::{
	fs::File,
	io::{self, Read, Seek},
	path::{Path, PathBuf},
};

use rai_pal_proc_macros::{serializable_enum, serializable_struct};

use crate::{
	game_engines::{
		game_engine::GameEngine,
		unity::{self, UnityBackend},
		unreal,
	},
	paths::normalize_path,
	result::Result,
};

#[serializable_enum]
pub enum Architecture {
	X64,
	X86,
}

#[serializable_struct]
pub struct GameExecutable {
	pub path: PathBuf,
	pub engine: Option<GameEngine>,
	pub architecture: Option<Architecture>,
	pub unity_backend: Option<UnityBackend>,
}

pub fn get_architecture(exe_path: &Path) -> Result<Option<Architecture>> {
	// Open the file
	let mut file = File::open(exe_path)?;

	// Read only the first 2 bytes (DOS header signature)
	let mut buf2 = [0; 2];
	file.read_exact(&mut buf2)?;

	// Check if the file starts with "MZ" (DOS header signature)

	if &buf2 == b"MZ" {
		// Seek to the 60th byte to read the offset to the PE header
		file.seek(io::SeekFrom::Start(60))?;
		let mut buf4 = [0; 4];
		file.read_exact(&mut buf4)?;

		// Convert offset to PE header to u32
		let pe_offset = u32::from_le_bytes(buf4);

		// Seek to the PE header offset
		file.seek(io::SeekFrom::Start(u64::from(pe_offset)))?;

		file.read_exact(&mut buf4)?;

		// Check PE signature (should be "PE\0\0")

		// Read the machine type field at offset 4 bytes from the PE signature
		let mut machine_buf = [0; 2];
		file.read_exact(&mut machine_buf)?;

		// Convert machine type to u16
		let machine_type = u16::from_le_bytes(machine_buf);

		// Check machine type to determine 32-bit or 64-bit
		return match machine_type {
			0x014c => Ok(Some(Architecture::X86)),
			0x8664 => Ok(Some(Architecture::X64)),
			_ => Ok(None),
		};
	}

	Ok(None)
}

impl GameExecutable {
	pub fn new(path: &Path) -> Option<Self> {
		const VALID_EXTENSIONS: [&str; 3] = ["exe", "x86_64", "x86"];

		if !path.is_file() {
			return None;
		}

		// We ignore games that don't have an extension.
		let extension = path.extension()?.to_str()?;

		if !VALID_EXTENSIONS.contains(&extension.to_lowercase().as_str()) {
			return None;
		}

		if extension == "x86" && path.with_extension("x86_64").is_file() {
			// If there's an x86_64 version, we ignore the x86 version.
			// I'm just gonna presume there are no x86 modders out there,
			// if someone cries about it I'll make this smarter.
			return None;
		}

		let normalized_path = normalize_path(path);

		unity::get_executable(&normalized_path)
			.or_else(|| unreal::get_executable(&normalized_path))
			.or_else(|| {
				Some(Self {
					path: path.to_owned(),
					engine: None,
					architecture: None,
					unity_backend: None,
				})
			})
	}
}
