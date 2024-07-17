use std::{
	fs::{self, File},
	io::{self, Read, Seek},
	path::{Path, PathBuf},
};

use goblin::{elf::Elf, pe::PE};
use log::error;
use rai_pal_proc_macros::serializable_struct;

use crate::{
	game_engines::{
		game_engine::GameEngine,
		unity::{self, UnityScriptingBackend},
		unreal,
	},
	paths::normalize_path,
	result::Result,
	serializable_enum,
};

serializable_enum!(Architecture { X64, X86 });

serializable_enum!(OperatingSystem { Linux, Windows });

#[serializable_struct]
pub struct GameExecutable {
	pub path: PathBuf,
	pub name: String,
	pub engine: Option<GameEngine>,
	pub architecture: Option<Architecture>,
	pub operating_system: Option<OperatingSystem>,
	pub scripting_backend: Option<UnityScriptingBackend>,
}

fn is64(file_path: &Path) -> Result<bool> {
	// Open the file
	let mut file = File::open(file_path)?;

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
		file.seek(io::SeekFrom::Start(pe_offset as u64))?;

		file.read_exact(&mut buf4)?;

		// Check PE signature (should be "PE\0\0")

		// Read the machine type field at offset 4 bytes from the PE signature
		let mut machine_buf = [0; 2];
		file.read_exact(&mut machine_buf)?;

		// Convert machine type to u16
		let machine_type = u16::from_le_bytes(machine_buf);

		// Check machine type to determine 32-bit or 64-bit
		return match machine_type {
			0x014c => Ok(false),
			0x8664 => Ok(true),
			_ => Ok(false),
		};
	} else {
		println!("Not an executable file");
	}

	Ok(false)
}

pub fn read_windows_binary(file: &[u8]) -> Result<(Option<OperatingSystem>, Option<Architecture>)> {
	match PE::parse(file)?.header.coff_header.machine {
		goblin::pe::header::COFF_MACHINE_X86_64 => {
			Ok((Some(OperatingSystem::Windows), Some(Architecture::X64)))
		}
		goblin::pe::header::COFF_MACHINE_X86 => {
			Ok((Some(OperatingSystem::Windows), Some(Architecture::X86)))
		}
		_ => Ok((Some(OperatingSystem::Windows), None)),
	}
}

pub fn read_linux_binary(file: &[u8]) -> Result<(Option<OperatingSystem>, Option<Architecture>)> {
	match Elf::parse(file)?.header.e_machine {
		goblin::elf::header::EM_X86_64 => {
			Ok((Some(OperatingSystem::Linux), Some(Architecture::X64)))
		}
		goblin::elf::header::EM_386 => Ok((Some(OperatingSystem::Linux), Some(Architecture::X86))),
		_ => Ok((Some(OperatingSystem::Linux), None)),
	}
}

pub fn get_os_and_architecture(
	file_path: &Path,
) -> Result<(Option<OperatingSystem>, Option<Architecture>)> {
	// read only header, don't open whole file
	let buf: &mut [u8] = &mut [0; 128];
	fs::File::open(file_path)?.read_exact(buf)?;

	#[cfg(target_os = "linux")]
	{
		let elf_result = read_linux_binary(buf);
		if elf_result.is_ok() {
			return elf_result;
		}
		if let Err(err) = elf_result {
			error!("ELF error: {err}");
		}
	}

	if let Ok(is_windows_64_bit_exe) = is64(file_path) {
		return Ok((
			Some(OperatingSystem::Windows),
			if is_windows_64_bit_exe {
				Some(Architecture::X64)
			} else {
				Some(Architecture::X86)
			},
		));
	}

	let pe_result = read_windows_binary(buf);
	if pe_result.is_ok() {
		return pe_result;
	}

	error!("Failed to parse exe as ELF or PE");
	if let Err(err) = pe_result {
		error!("PE error: {err}");
	}

	Ok((None, None))
}

impl GameExecutable {
	pub fn new(path: &Path) -> Option<Self> {
		let normalized_path = normalize_path(path);

		unity::get_executable(&normalized_path).or_else(|| unreal::get_executable(&normalized_path))
	}
}
