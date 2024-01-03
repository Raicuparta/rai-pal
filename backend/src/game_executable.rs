use std::{
	fs::{
		self,
		File,
	},
	io::Read,
	path::{
		Path,
		PathBuf,
	},
	time::Instant,
};

use goblin::{
	elf::Elf,
	pe::PE,
};
use log::error;

use crate::{
	debug::LoggableInstant,
	game_engines::{
		game_engine::GameEngine,
		unity::{
			self,
			UnityScriptingBackend,
		},
		unreal,
	},
	paths::normalize_path,
	result::Result,
	serializable_enum,
	serializable_struct,
};

serializable_enum!(Architecture { X64, X86 });

serializable_enum!(OperatingSystem { Linux, Windows });

serializable_struct!(GameExecutable {
	pub path: PathBuf,
	pub engine: Option<GameEngine>,
	pub architecture: Option<Architecture>,
  pub operating_system: Option<OperatingSystem>,
	pub scripting_backend: Option<UnityScriptingBackend>
});

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
	fs::read(file_path).map(|file| {
		let elf_result = read_linux_binary(&file);
		if elf_result.is_ok() {
			return elf_result;
		}

		let pe_result = read_windows_binary(&file);
		if pe_result.is_ok() {
			return pe_result;
		}

		error!("Failed to parse exe as ELF or PE");
		if let Err(err) = elf_result {
			error!("ELF error: {err}");
		}
		if let Err(err) = pe_result {
			error!("PE error: {err}");
		}

		Ok((None, None))
	})?
}

impl GameExecutable {
	pub fn new(path: &Path) -> Option<Self> {
		let normalized_path = normalize_path(path);

		unity::get_executable(&normalized_path).or_else(|| unreal::get_executable(&normalized_path))
	}
}
