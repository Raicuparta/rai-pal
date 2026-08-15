use std::path::Path;

use pelite::{
	PeFile,
	Wrap,
};
use rai_pal_proc_macros::serializable_enum;

use crate::{
	game_engines::mmap_safe,
	result::Result,
};

#[serializable_enum]
pub enum Architecture {
	X64,
	X86,
}

pub fn get_architecture(exe_path: &Path) -> Result<Option<Architecture>> {
	let mmap = mmap_safe::map_readonly(exe_path)?;

	if mmap.starts_with(b"\x7FELF") {
		return Ok(get_elf_architecture(&mmap));
	}

	let Ok(pe) = PeFile::from_bytes(&mmap) else {
		return Ok(None);
	};

	Ok(Some(match pe {
		Wrap::T64(_) => Architecture::X64,
		Wrap::T32(_) => Architecture::X86,
	}))
}

fn get_elf_architecture(mmap: &[u8]) -> Option<Architecture> {
	let machine_bytes = mmap.get(18..20)?.try_into().ok()?;
	let machine = u16::from_le_bytes(machine_bytes);

	match machine {
		// EM_386
		3 => Some(Architecture::X86),
		// EM_X86_64
		62 => Some(Architecture::X64),
		_ => None,
	}
}
