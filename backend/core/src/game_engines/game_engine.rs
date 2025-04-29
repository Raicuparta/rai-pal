use std::{cmp::Ordering, path::Path};

use rai_pal_proc_macros::{serializable_enum, serializable_struct};

use super::{unity, unreal};

#[serializable_enum]
pub enum EngineBrand {
	Unity,
	Unreal,
	Godot,
	GameMaker,
}

#[serializable_struct]
pub struct GameEngine {
	pub brand: EngineBrand,
	pub version: Option<EngineVersion>,
}

#[serializable_struct]
pub struct EngineVersionNumbers {
	pub major: u32,
	pub minor: Option<u32>,
	pub patch: Option<u32>,
}

#[serializable_struct]
pub struct EngineVersion {
	pub numbers: EngineVersionNumbers,
	pub suffix: Option<String>,
	pub display: String,
}

impl Eq for EngineVersionNumbers {}

impl PartialEq for EngineVersionNumbers {
	fn eq(&self, other: &Self) -> bool {
		self.major == other.major && self.minor == other.minor && self.patch == other.patch
	}
}

impl Ord for EngineVersionNumbers {
	fn cmp(&self, other: &Self) -> Ordering {
		match self.major.cmp(&other.major) {
			Ordering::Equal => match self.minor.cmp(&other.minor) {
				Ordering::Equal => self.patch.cmp(&other.patch),
				other => other,
			},
			other => other,
		}
	}
}

impl PartialOrd for EngineVersionNumbers {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

impl Eq for GameEngine {}

impl PartialEq for GameEngine {
	fn eq(&self, other: &Self) -> bool {
		self.brand == other.brand
			&& self
				.version
				.as_ref()
				.map(|version| version.display.to_lowercase())
				== other
					.version
					.as_ref()
					.map(|version| version.display.to_lowercase())
	}
}

impl Ord for GameEngine {
	fn cmp(&self, other: &Self) -> Ordering {
		match self.brand.cmp(&other.brand) {
			Ordering::Equal => self
				.version
				.as_ref()
				.map(|version| &version.numbers)
				.cmp(&other.version.as_ref().map(|version| &version.numbers)),
			other => other,
		}
	}
}

impl PartialOrd for GameEngine {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

pub fn get_exe_engine(exe_path: &Path) -> Option<EngineBrand> {
	if unity::is_unity_exe(exe_path) {
		Some(EngineBrand::Unity)
	} else if unreal::is_unreal_exe(exe_path) {
		Some(EngineBrand::Unreal)
	} else {
		None
	}
}
