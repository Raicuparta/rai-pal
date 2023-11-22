use std::{
	fs,
	path::{
		Path,
		PathBuf,
	},
};

use goblin::{
	elf::Elf,
	pe::PE,
};

use crate::{
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

pub fn get_os_and_architecture(
	file_path: &Path,
) -> Result<(Option<OperatingSystem>, Option<Architecture>)> {
	fs::read(file_path).map(|file| {
		let elf_result = match Elf::parse(&file) {
			Ok(elf) => match elf.header.e_machine {
				goblin::elf::header::EM_X86_64 => {
					Ok((Some(OperatingSystem::Linux), Some(Architecture::X64)))
				}
				goblin::elf::header::EM_386 => {
					Ok((Some(OperatingSystem::Linux), Some(Architecture::X86)))
				}
				_ => Ok((Some(OperatingSystem::Linux), None)),
			},
			Err(err) => Err(err),
		};

		if elf_result.is_ok() {
			return Ok(elf_result?);
		}

		let pe_result = match PE::parse(&file) {
			Ok(pe) => match pe.header.coff_header.machine {
				goblin::pe::header::COFF_MACHINE_X86_64 => {
					Ok((Some(OperatingSystem::Windows), Some(Architecture::X64)))
				}
				goblin::pe::header::COFF_MACHINE_X86 => {
					Ok((Some(OperatingSystem::Windows), Some(Architecture::X86)))
				}
				_ => Ok((Some(OperatingSystem::Windows), None)),
			},
			Err(err) => Err(err),
		};

		if pe_result.is_ok() {
			return Ok(pe_result?);
		}

		eprintln!("Failed to parse exe as ELF or PE");
		if let Err(err) = elf_result {
			eprintln!("ELF error: {err}");
		}
		if let Err(err) = pe_result {
			eprintln!("PE error: {err}");
		}

		Ok((None, None))
	})?
}

impl GameExecutable {
	pub fn new(path: &Path) -> Self {
		let normalized_path = normalize_path(path);

		unity::get_engine(&normalized_path)
			.or_else(|| unreal::get_engine(&normalized_path))
			.unwrap_or_else(|| {
				let (operating_system, architecture) =
					get_os_and_architecture(&normalized_path).unwrap_or((None, None));

				Self {
					engine: None,
					architecture,
					operating_system,
					path: normalized_path,
					scripting_backend: None,
				}
			})
	}
}
