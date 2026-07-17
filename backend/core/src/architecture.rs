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

	let Ok(pe) = PeFile::from_bytes(&mmap) else {
		return Ok(None);
	};

	Ok(Some(match pe {
		Wrap::T64(_) => Architecture::X64,
		Wrap::T32(_) => Architecture::X86,
	}))
}
